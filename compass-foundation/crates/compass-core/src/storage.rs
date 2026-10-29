//! Local identity and immutable publication. Directory descriptors prevent link substitution.
use crate::{
    Error, Result, codec,
    model::{Policy, Snapshot, Workspace},
    observe,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rustix::fs::{self as rfs, FlockOperation, Mode, OFlags, RenameFlags, ResolveFlags};
use serde::Serialize;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

const RESOLVE: ResolveFlags = ResolveFlags::BENEATH
    .union(ResolveFlags::NO_SYMLINKS)
    .union(ResolveFlags::NO_MAGICLINKS);

fn io(context: &str, error: rustix::io::Errno) -> Error {
    Error::io(context, error.into())
}

fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::cancelled())
    } else {
        Ok(())
    }
}

fn open_at(parent: &File, name: &str, flags: OFlags, mode: Mode) -> Result<File> {
    rfs::openat2(parent, name, flags | OFlags::CLOEXEC, mode, RESOLVE)
        .map(File::from)
        .map_err(|e| io(&format!("open {name}"), e))
}

fn make_directory(parent: &File, name: &str) -> Result<()> {
    match rfs::mkdirat(parent, name, Mode::from_raw_mode(0o700)) {
        Ok(()) | Err(rustix::io::Errno::EXIST) => Ok(()),
        Err(e) => Err(io(&format!("create directory {name}"), e)),
    }
}

struct Store {
    root: File,
    state: File,
    _lock: File,
    root_path: PathBuf,
}

impl Store {
    fn open(path: &Path, create: bool, cancel: &AtomicBool) -> Result<Self> {
        check_cancel(cancel)?;
        let root_path = path
            .canonicalize()
            .map_err(|e| Error::io("resolve root", e))?;
        let root = rfs::open(
            &root_path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map(File::from)
        .map_err(|e| io("open root", e))?;
        if create {
            make_directory(&root, ".compass")?;
        }
        let state = open_at(
            &root,
            ".compass",
            OFlags::RDONLY | OFlags::DIRECTORY,
            Mode::empty(),
        )?;
        let lock = open_at(
            &state,
            "lock",
            OFlags::RDWR | OFlags::CREATE | OFlags::NONBLOCK,
            Mode::from_raw_mode(0o600),
        )?;
        if !lock
            .metadata()
            .map_err(|e| Error::io("inspect lock", e))?
            .is_file()
        {
            return Err(Error::corrupt("workspace lock is not a regular file"));
        }
        loop {
            check_cancel(cancel)?;
            match rfs::flock(&lock, FlockOperation::NonBlockingLockExclusive) {
                Ok(()) => break,
                Err(rustix::io::Errno::WOULDBLOCK) => std::thread::sleep(Duration::from_millis(20)),
                Err(rustix::io::Errno::INTR) => continue,
                Err(e) => return Err(io("lock workspace", e)),
            }
        }
        Ok(Self {
            root,
            state,
            _lock: lock,
            root_path,
        })
    }

    fn read_optional(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let fd = match rfs::openat2(
            &self.state,
            name,
            OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
            RESOLVE,
        ) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(e) => return Err(io(&format!("open {name}"), e)),
        };
        let mut file = File::from(fd);
        let meta = file
            .metadata()
            .map_err(|e| Error::io("inspect state record", e))?;
        if !meta.is_file() || meta.len() > 1024 * 1024 {
            return Err(Error::corrupt(
                "identity/policy must be a regular file at most 1 MiB",
            ));
        }
        let mut data = Vec::new();
        (&mut file)
            .take(1024 * 1024 + 1)
            .read_to_end(&mut data)
            .map_err(|e| Error::io("read state record", e))?;
        if data.len() > 1024 * 1024 {
            return Err(Error::corrupt("state record exceeded 1 MiB"));
        }
        Ok(Some(data))
    }

    fn config(&self) -> Result<(Workspace, Policy)> {
        let workspace: Workspace =
            codec::decode(&self.read_optional("workspace.json")?.ok_or_else(|| {
                Error::invalid("workspace is not initialized; run compass init")
            })?)?;
        workspace.validate()?;
        let bytes = self
            .read_optional("policy.json")?
            .ok_or_else(|| Error::corrupt("initialized workspace has no policy.json"))?;
        let policy: Policy =
            codec::decode(&bytes).map_err(|e| Error::invalid(format!("invalid policy: {e}")))?;
        policy
            .validate()
            .map_err(|e| Error::invalid(format!("invalid policy: {e}")))?;
        Ok((workspace, policy))
    }
}

fn publish(dir: &File, name: &str, bytes: &[u8], cancel: &AtomicBool) -> Result<()> {
    publish_with_hook(dir, name, bytes, cancel, || {})
}

fn publish_with_hook<F: FnOnce()>(
    dir: &File,
    name: &str,
    bytes: &[u8],
    cancel: &AtomicBool,
    before_publish: F,
) -> Result<()> {
    check_cancel(cancel)?;
    let temporary = format!(".tmp-{}", Uuid::new_v4());
    let mut file = open_at(
        dir,
        &temporary,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL,
        Mode::from_raw_mode(0o600),
    )?;
    let result = (|| {
        for chunk in bytes.chunks(64 * 1024) {
            check_cancel(cancel)?;
            file.write_all(chunk)
                .map_err(|e| Error::io("write temporary record", e))?;
        }
        file.sync_all()
            .map_err(|e| Error::io("sync temporary record", e))?;
        before_publish();
        check_cancel(cancel)?;
        rfs::renameat_with(dir, temporary.as_str(), dir, name, RenameFlags::NOREPLACE)
            .map_err(|e| io("publish without replacement", e))?;
        dir.sync_all()
            .map_err(|e| Error::io("sync record directory", e))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = rfs::unlinkat(dir, temporary.as_str(), rfs::AtFlags::empty());
    }
    result
}

#[derive(Serialize)]
pub struct InitReceipt {
    pub workspace_id: String,
    pub workspace_path: String,
    pub workspace_path_base64: String,
    pub policy_path: String,
    pub policy_path_base64: String,
}

// Display paths may replace non-UTF-8 bytes. Native absolute paths travel
// separately; unlike manifest path_key, these are locations, not identities.
fn encoded_path(path: &Path) -> String {
    URL_SAFE_NO_PAD.encode(path.as_os_str().as_bytes())
}

pub fn init(root: &Path, cancel: &AtomicBool) -> Result<InitReceipt> {
    let store = Store::open(root, true, cancel)?;
    let existing_workspace = store.read_optional("workspace.json")?;
    let existing_policy = store.read_optional("policy.json")?;
    let policy = match existing_policy {
        Some(bytes) => {
            let policy: Policy = codec::decode(&bytes)
                .map_err(|e| Error::invalid(format!("invalid policy: {e}")))?;
            policy
                .validate()
                .map_err(|e| Error::invalid(format!("invalid policy: {e}")))?;
            policy
        }
        None if existing_workspace.is_some() => {
            return Err(Error::corrupt("initialized workspace has no policy.json"));
        }
        None => {
            let policy = Policy::default();
            policy.validate()?;
            publish(
                &store.state,
                "policy.json",
                &codec::canonical(&policy)?,
                cancel,
            )?;
            policy
        }
    };
    policy.validate()?;
    let workspace = if let Some(bytes) = existing_workspace {
        let workspace: Workspace = codec::decode(&bytes)?;
        workspace.validate()?;
        workspace
    } else {
        let workspace = Workspace {
            schema_version: "1".into(),
            contract_version: "0.1.0".into(),
            workspace_id: Uuid::new_v4().to_string(),
            path_namespace: "linux_bytes_v1".into(),
            created_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .map_err(|e| Error::corrupt(e.to_string()))?,
        };
        workspace.validate()?;
        publish(
            &store.state,
            "workspace.json",
            &codec::canonical(&workspace)?,
            cancel,
        )?;
        workspace
    };
    make_directory(&store.state, "snapshots")?;
    open_at(
        &store.state,
        "snapshots",
        OFlags::RDONLY | OFlags::DIRECTORY,
        Mode::empty(),
    )?;
    store
        .state
        .sync_all()
        .map_err(|e| Error::io("sync initialization", e))?;
    store
        .root
        .sync_all()
        .map_err(|e| Error::io("sync workspace root", e))?;
    let workspace_path = store.root_path.join(".compass/workspace.json");
    let policy_path = store.root_path.join(".compass/policy.json");
    Ok(InitReceipt {
        workspace_id: workspace.workspace_id,
        workspace_path: workspace_path.to_string_lossy().into_owned(),
        workspace_path_base64: encoded_path(&workspace_path),
        policy_path: policy_path.to_string_lossy().into_owned(),
        policy_path_base64: encoded_path(&policy_path),
    })
}

#[derive(Serialize)]
pub struct SnapshotReceipt {
    pub snapshot_id: String,
    pub snapshot_path: String,
    pub snapshot_path_base64: String,
    pub quality: String,
    pub state_digest: Option<String>,
}

pub fn snapshot(root: &Path, cancel: &AtomicBool) -> Result<SnapshotReceipt> {
    let store = Store::open(root, false, cancel)?;
    let (workspace, policy) = store.config()?;
    let directory = open_at(
        &store.state,
        "snapshots",
        OFlags::RDONLY | OFlags::DIRECTORY,
        Mode::empty(),
    )?;
    let snapshot = observe::observe(&store.root, &workspace, &policy, cancel)?;
    snapshot.validate()?;
    let filename = format!("{}.json", snapshot.snapshot_id);
    publish(&directory, &filename, &codec::canonical(&snapshot)?, cancel)?;
    let snapshot_path = store.root_path.join(".compass/snapshots").join(filename);
    Ok(SnapshotReceipt {
        snapshot_path: snapshot_path.to_string_lossy().into_owned(),
        snapshot_path_base64: encoded_path(&snapshot_path),
        snapshot_id: snapshot.snapshot_id,
        quality: snapshot.quality,
        state_digest: snapshot.state_digest,
    })
}

pub fn load_snapshot(path: &Path) -> Result<Snapshot> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY)
        .open(path)
        .map_err(|e| Error::io("open snapshot", e))?;
    if !file
        .metadata()
        .map_err(|e| Error::io("inspect snapshot", e))?
        .is_file()
    {
        return Err(Error::corrupt("snapshot must be a regular file"));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| Error::io("read snapshot", e))?;
    let snapshot: Snapshot = codec::decode(&bytes)?;
    snapshot
        .validate()
        .map_err(|e| Error::corrupt(format!("invalid snapshot: {e}")))?;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn receipts_preserve_native_root_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join(std::ffi::OsStr::from_bytes(b"root\xff"));
        std::fs::create_dir(&root).unwrap();
        let cancel = AtomicBool::new(false);
        let initialized = init(&root, &cancel).unwrap();
        let snapshot = snapshot(&root, &cancel).unwrap();
        for (encoded, suffix) in [
            (
                initialized.workspace_path_base64,
                ".compass/workspace.json".to_owned(),
            ),
            (
                initialized.policy_path_base64,
                ".compass/policy.json".to_owned(),
            ),
            (
                snapshot.snapshot_path_base64,
                format!(".compass/snapshots/{}.json", snapshot.snapshot_id),
            ),
        ] {
            let decoded = URL_SAFE_NO_PAD.decode(encoded).unwrap();
            let expected = root.join(suffix);
            assert_eq!(decoded, expected.as_os_str().as_bytes());
            assert!(std::fs::read(std::ffi::OsStr::from_bytes(&decoded)).is_ok());
        }
    }

    #[test]
    fn cancellation_before_publication_leaves_no_final_or_temporary() {
        let temp = tempfile::tempdir().unwrap();
        let dir = File::open(temp.path()).unwrap();
        let cancel = AtomicBool::new(false);
        let error = publish_with_hook(&dir, "record.json", b"{}", &cancel, || {
            cancel.store(true, Ordering::Relaxed)
        })
        .unwrap_err();
        assert_eq!(error.exit_code, 130);
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn existing_record_is_never_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let dir = File::open(temp.path()).unwrap();
        let cancel = AtomicBool::new(false);
        publish(&dir, "record.json", b"first", &cancel).unwrap();
        assert!(publish(&dir, "record.json", b"second", &cancel).is_err());
        assert_eq!(
            std::fs::read(temp.path().join("record.json")).unwrap(),
            b"first"
        );
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn state_directory_link_is_rejected_without_external_writes() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), root.path().join(".compass")).unwrap();
        assert!(init(root.path(), &AtomicBool::new(false)).is_err());
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }

    #[test]
    fn repeated_and_concurrent_init_preserves_identity() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().to_owned();
        let second_path = path.clone();
        let thread =
            std::thread::spawn(move || init(&second_path, &AtomicBool::new(false)).unwrap());
        let one = init(&path, &AtomicBool::new(false)).unwrap();
        let two = thread.join().unwrap();
        let three = init(&path, &AtomicBool::new(false)).unwrap();
        assert_eq!(one.workspace_id, two.workspace_id);
        assert_eq!(two.workspace_id, three.workspace_id);
    }
}
