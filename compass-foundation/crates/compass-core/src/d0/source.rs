//! Linux capability-based opens. No stat/open fallback, retries or persistent cache.
use super::{
    resources::Context,
    types::{Error, Result},
};
use crate::model::digest;
use serde_json::{Value, json};
use std::{
    ffi::CString,
    fs::{File, Metadata},
    io::{self, Read},
    os::{
        fd::{AsRawFd, FromRawFd, RawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Path, PathBuf},
};

#[repr(C)]
struct OpenHow {
    flags: u64,
    mode: u64,
    resolve: u64,
}
fn open(root: RawFd, path: &[u8], flags: i32) -> io::Result<File> {
    let name = CString::new(path).map_err(|_| io::Error::from_raw_os_error(libc::EINVAL))?;
    let how = OpenHow {
        flags: (flags
            | libc::O_CLOEXEC
            | if flags & libc::O_PATH == 0 {
                libc::O_NOCTTY | libc::O_NONBLOCK
            } else {
                0
            }) as u64,
        mode: 0,
        resolve: 0x08 | 0x04 | 0x02,
    };
    // SAFETY: NUL-terminated path and initialized open_how; returned fd is owned once.
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            root,
            name.as_ptr(),
            &how,
            std::mem::size_of::<OpenHow>(),
        )
    };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd as i32) })
    }
}
fn identity(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev() && a.ino() == b.ino() && a.mode() & libc::S_IFMT == b.mode() & libc::S_IFMT
}
fn unchanged(a: &Metadata, b: &Metadata) -> bool {
    identity(a, b)
        && a.len() == b.len()
        && a.mode() == b.mode()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}
pub(crate) fn local_error(e: &io::Error) -> &'static str {
    match e.raw_os_error() {
        Some(libc::ELOOP | libc::ENOTDIR | libc::EXDEV | libc::ENXIO) => "source_invalid_kind",
        Some(libc::ENOENT) => "object_missing",
        _ => "source_unreadable",
    }
}
fn capability(e: &io::Error) -> bool {
    matches!(
        e.raw_os_error(),
        Some(libc::ENOSYS | libc::EINVAL | libc::E2BIG)
    )
}
struct DirectoryBinding {
    name: Vec<u8>,
    handle: File,
    before: Metadata,
}
impl DirectoryBinding {
    fn stable(&self, origin: RawFd) -> bool {
        self.handle
            .metadata()
            .is_ok_and(|m| identity(&m, &self.before))
            && open(origin, &self.name, libc::O_PATH | libc::O_DIRECTORY)
                .and_then(|f| f.metadata())
                .is_ok_and(|m| identity(&m, &self.before))
    }
}
pub(crate) struct Root {
    dir: File,
    origin: File,
    path: Vec<u8>,
    ancestors: Vec<DirectoryBinding>,
    pub identity: (u64, u64),
}
impl Root {
    pub fn at(path: &Path) -> Result<Self> {
        let origin = File::open("/").map_err(|_| Error("operational_error"))?;
        let absolute = std::path::absolute(path).map_err(|_| Error("source_invalid"))?;
        let bytes = absolute.as_os_str().as_bytes();
        let relative = bytes.strip_prefix(b"/").unwrap_or(bytes);
        let mut ancestors = vec![];
        for (i, b) in relative.iter().enumerate() {
            if *b == b'/' && i > 0 {
                let name = relative[..i].to_vec();
                let handle = open(origin.as_raw_fd(), &name, libc::O_PATH | libc::O_DIRECTORY)
                    .map_err(|e| {
                        Error(if capability(&e) {
                            "operational_error"
                        } else {
                            "source_invalid"
                        })
                    })?;
                let before = handle.metadata().map_err(|_| Error("source_invalid"))?;
                ancestors.push(DirectoryBinding {
                    name,
                    handle,
                    before,
                });
            }
        }
        let dir = open(
            origin.as_raw_fd(),
            if relative.is_empty() { b"." } else { relative },
            libc::O_RDONLY | libc::O_DIRECTORY,
        )
        .map_err(|e| {
            Error(if capability(&e) {
                "operational_error"
            } else {
                "source_invalid"
            })
        })?;
        let meta = dir.metadata().map_err(|_| Error("source_invalid"))?;
        Ok(Self {
            dir,
            origin,
            path: if relative.is_empty() {
                b".".to_vec()
            } else {
                relative.to_vec()
            },
            identity: (meta.dev(), meta.ino()),
            ancestors,
        })
    }
    pub fn read_input(
        &self,
        name: &[u8],
        limit: u64,
        ctx: &mut Context<'_>,
        invalid: &'static str,
    ) -> Result<Vec<u8>> {
        let mut file = self.file(name).map_err(|e| {
            Error(if capability(&e) {
                "operational_error"
            } else {
                invalid
            })
        })?;
        ctx.check("input_opened")?;
        let (data, eof, failed) = file.read(limit + 1, ctx, false)?;
        if !file.stable(self) || failed {
            return Err(Error(invalid));
        }
        if !eof || data.len() as u64 > limit {
            return Err(Error("input_limit"));
        }
        Ok(data)
    }
    fn stable(&self) -> bool {
        open(
            self.origin.as_raw_fd(),
            &self.path,
            libc::O_PATH | libc::O_DIRECTORY,
        )
        .and_then(|f| f.metadata())
        .is_ok_and(|m| (m.dev(), m.ino()) == self.identity)
            && self
                .ancestors
                .iter()
                .all(|d| d.stable(self.origin.as_raw_fd()))
    }
    fn file(&self, name: &[u8]) -> io::Result<BoundFile> {
        let mut dirs = vec![];
        for (i, b) in name.iter().enumerate() {
            if *b == b'/' {
                let part = name[..i].to_vec();
                let file = open(
                    self.dir.as_raw_fd(),
                    &part,
                    libc::O_PATH | libc::O_DIRECTORY,
                )?;
                let before = file.metadata()?;
                dirs.push(DirectoryBinding {
                    name: part,
                    handle: file,
                    before,
                });
            }
        }
        let file = open(self.dir.as_raw_fd(), name, libc::O_RDONLY)?;
        let before = file.metadata()?;
        if !before.is_file() {
            return Err(io::Error::from_raw_os_error(libc::ENXIO));
        }
        Ok(BoundFile {
            file,
            before,
            dirs,
            name: name.to_vec(),
        })
    }
}
struct BoundFile {
    file: File,
    before: Metadata,
    dirs: Vec<DirectoryBinding>,
    name: Vec<u8>,
}
impl BoundFile {
    fn stable(&self, root: &Root) -> bool {
        root.stable()
            && self
                .file
                .metadata()
                .is_ok_and(|m| unchanged(&m, &self.before))
            && open(root.dir.as_raw_fd(), &self.name, libc::O_PATH)
                .and_then(|f| f.metadata())
                .is_ok_and(|m| identity(&m, &self.before))
            && self.dirs.iter().all(|d| d.stable(root.dir.as_raw_fd()))
    }
    fn read(
        &mut self,
        cap: u64,
        ctx: &mut Context<'_>,
        count: bool,
    ) -> Result<(Vec<u8>, bool, bool)> {
        let mut bytes = vec![];
        let mut chunk = [0u8; 65536];
        let mut eof = false;
        let mut failed = false;
        while (bytes.len() as u64) < cap {
            ctx.check("read_chunk")?;
            let n = ((cap - bytes.len() as u64) as usize).min(chunk.len());
            match self.file.read(&mut chunk[..n]) {
                Ok(0) => {
                    eof = true;
                    break;
                }
                Ok(n) => {
                    if count {
                        ctx.read += n as u128;
                    }
                    bytes.extend_from_slice(&chunk[..n]);
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                    ctx.check("read_interrupted")?;
                }
                Err(_) => {
                    failed = true;
                    break;
                }
            }
        }
        ctx.check("read_finished")?;
        Ok((bytes, eof, failed))
    }
}
pub(crate) fn raw_file(
    path: &Path,
    limit: u64,
    ctx: &mut Context<'_>,
    invalid: &'static str,
) -> Result<Vec<u8>> {
    let root = Root::at(Path::new("/"))?;
    let absolute: PathBuf = std::path::absolute(path).map_err(|_| Error(invalid))?;
    let name = absolute.as_os_str().as_bytes().strip_prefix(b"/").unwrap();
    let mut file = root.file(name).map_err(|e| {
        Error(if capability(&e) {
            "operational_error"
        } else {
            invalid
        })
    })?;
    ctx.check("input_opened")?;
    let (data, eof, failed) = file.read(limit + 1, ctx, false)?;
    if !file.stable(&root) || failed {
        return Err(Error(invalid));
    }
    if !eof || data.len() as u64 > limit {
        return Err(Error("input_limit"));
    }
    Ok(data)
}
pub(crate) struct Source {
    pub root: Root,
    pub descriptor: Value,
    pub raw: Vec<u8>,
}
impl Source {
    // Each supplied root reference has its own pinned component chain, even when
    // the object bytes and physical package root can be shared with another side.
    pub fn reference_stable(&self) -> bool {
        self.root.stable()
    }
    pub fn shared(&self, other: &Self, rev: &Value, other_rev: &Value) -> bool {
        self.root.identity == other.root.identity && self.raw == other.raw && rev == other_rev
    }
    pub fn object(
        &self,
        revision: &Value,
        ctx: &mut Context<'_>,
        check: &mut Value,
    ) -> Result<Vec<u8>> {
        let size = revision["size_bytes"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        if !ctx.reserve_bytes(size) {
            check["reasons"] = json!(["total_bytes_limit"]);
            return Ok(vec![]);
        }
        let name = format!("objects/{}", revision["content_sha256"].as_str().unwrap());
        let mut file = match self.root.file(name.as_bytes()) {
            Ok(file) => file,
            Err(e) => {
                if capability(&e) {
                    return Err(Error("operational_error"));
                }
                check["state"] = json!("failed");
                check["reasons"] = json!([local_error(&e)]);
                return Ok(vec![]);
            }
        };
        ctx.check("object_opened")?;
        let (bytes, eof, failed) = file.read(size + 1, ctx, true)?;
        let mut reasons = vec![];
        if failed {
            reasons.push("source_unreadable");
        } else if eof {
            let actual =
                json!({"content_sha256":digest(&bytes),"size_bytes":bytes.len().to_string()});
            let hash = actual["content_sha256"] == revision["content_sha256"];
            let sized = bytes.len() as u64 == size;
            check["actual_revision"] = actual;
            check["hash_check"] = json!(hash);
            check["size_check"] = json!(sized);
            if !hash {
                reasons.push("hash_mismatch");
            }
            if !sized {
                reasons.push("size_mismatch");
            }
        } else {
            check["size_check"] = json!(false);
            reasons.push("size_mismatch");
        }
        ctx.check("object_before_stability")?;
        if !file.stable(&self.root) {
            reasons.push("source_unstable");
        }
        reasons.sort();
        reasons.dedup();
        check["state"] = json!(if reasons.is_empty() {
            "verified"
        } else {
            "failed"
        });
        check["reasons"] = json!(reasons);
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::Limits;
    use super::*;
    use std::{
        fs,
        os::unix::{fs::symlink, net::UnixListener},
        sync::{Arc, Barrier, atomic::AtomicBool},
        thread,
    };
    fn setup(data: &[u8]) -> (tempfile::TempDir, Source, Value, Value) {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("objects")).unwrap();
        let rev = json!({"content_sha256":digest(data),"size_bytes":data.len().to_string()});
        fs::write(temp.path().join("objects").join(digest(data)), data).unwrap();
        let source = Source {
            root: Root::at(temp.path()).unwrap(),
            descriptor: json!({}),
            raw: vec![],
        };
        let check = json!({"state":"not_attempted","reasons":[],"actual_revision":null,"hash_check":null,"size_check":null});
        (temp, source, rev, check)
    }
    #[test]
    fn object_eof_prefix_empty_and_short() {
        for (actual, eof, hash, size) in [
            (b"abc".as_slice(), true, true, true),
            (b"ab", true, false, false),
            (b"abcd", false, false, false),
            (b"", true, false, false),
        ] {
            let (temp, source, rev, mut check) = setup(b"abc");
            fs::write(temp.path().join("objects").join(digest(b"abc")), actual).unwrap();
            let cancel = AtomicBool::new(false);
            let mut ctx = Context::new(Limits::default(), &cancel);
            source.object(&rev, &mut ctx, &mut check).unwrap();
            assert_eq!(
                check["hash_check"],
                if eof { json!(hash) } else { Value::Null }
            );
            assert_eq!(check["size_check"], json!(size));
            assert_eq!(check["actual_revision"].is_null(), !eof);
            assert_eq!(ctx.reserved, 4);
            assert_eq!(ctx.read, actual.len() as u128);
        }
        let (_temp, s, rev, mut check) = setup(b"");
        let c = AtomicBool::new(false);
        let mut ctx = Context::new(Limits::default(), &c);
        s.object(&rev, &mut ctx, &mut check).unwrap();
        assert_eq!(check["state"], "verified");
        assert_eq!(ctx.reserved, 1);
        assert_eq!(ctx.read, 0);
    }
    #[test]
    fn prohibited_kinds_missing_and_no_external_bytes() {
        for kind in [
            "regular",
            "missing",
            "root_link",
            "intermediate",
            "pending",
            "symlink",
            "fifo",
            "socket",
            "directory",
        ] {
            let (temp, mut source, rev, mut check) = setup(b"abc");
            let objects = temp.path().join("objects");
            let name = digest(b"abc");
            let path = objects.join(&name);
            let outside = tempfile::tempdir().unwrap();
            fs::write(outside.path().join(&name), b"abc").unwrap();
            match kind {
                "regular" => {}
                "missing" => fs::remove_file(&path).unwrap(),
                "root_link" => {
                    let link = temp.path().join("root-link");
                    symlink(temp.path(), &link).unwrap();
                    assert!(matches!(Root::at(&link), Err(Error("source_invalid"))));
                    continue;
                }
                "intermediate" | "pending" => {
                    fs::rename(&objects, temp.path().join("old")).unwrap();
                    symlink(
                        if kind == "pending" {
                            outside.path().join("absent")
                        } else {
                            outside.path().to_path_buf()
                        },
                        &objects,
                    )
                    .unwrap();
                }
                _ => {
                    fs::remove_file(&path).unwrap();
                    match kind {
                        "symlink" => symlink(outside.path().join(&name), &path).unwrap(),
                        "fifo" => {
                            let p = CString::new(path.as_os_str().as_bytes()).unwrap();
                            assert_eq!(unsafe { libc::mkfifo(p.as_ptr(), 0o600) }, 0);
                        }
                        "socket" => {
                            let short = temp.path().join("s");
                            let socket = UnixListener::bind(&short).unwrap();
                            fs::rename(short, &path).unwrap();
                            drop(socket);
                        }
                        "directory" => fs::create_dir(&path).unwrap(),
                        _ => unreachable!(),
                    }
                }
            }
            // Pinning precedes the adversarial object intervention in real operation too.
            source.descriptor = json!({});
            let cancel = AtomicBool::new(false);
            let mut ctx = Context::new(Limits::default(), &cancel);
            source.object(&rev, &mut ctx, &mut check).unwrap();
            assert_eq!(ctx.reserved, 4);
            if kind == "regular" {
                assert_eq!(check["state"], "verified");
                assert_eq!(ctx.read, 3);
            } else {
                assert_eq!(ctx.read, 0);
                assert!(check["actual_revision"].is_null());
                assert_eq!(
                    check["reasons"],
                    json!([if kind == "missing" {
                        "object_missing"
                    } else {
                        "source_invalid_kind"
                    }])
                );
            }
        }
    }
    #[test]
    fn synchronized_object_name_component_and_post_eof_replacement() {
        for mode in ["object", "name", "component", "post_eof"] {
            let (temp, source, rev, mut check) = setup(b"abc");
            let base = temp.path().to_owned();
            let gate = Arc::new(Barrier::new(2));
            let other = gate.clone();
            let worker = thread::spawn(move || {
                other.wait();
                let object = base.join("objects").join(digest(b"abc"));
                match mode {
                    "object" | "post_eof" => {
                        let new = base.join("replacement");
                        fs::write(&new, b"abc").unwrap();
                        fs::rename(new, object).unwrap();
                    }
                    "name" => {
                        fs::rename(object, base.join("moved")).unwrap();
                    }
                    "component" => {
                        fs::rename(base.join("objects"), base.join("old")).unwrap();
                        fs::create_dir(base.join("objects")).unwrap();
                        fs::hard_link(base.join("old").join(digest(b"abc")), object).unwrap();
                    }
                    _ => unreachable!(),
                };
                other.wait();
            });
            let cancel = AtomicBool::new(false);
            let mut ctx = Context::new(Limits::default(), &cancel);
            let mut fired = false;
            ctx.hook = Some(Box::new(move |event| {
                if !fired
                    && event
                        == if mode == "post_eof" {
                            "object_before_stability"
                        } else {
                            "object_opened"
                        }
                {
                    fired = true;
                    gate.wait();
                    gate.wait();
                }
                false
            }));
            source.object(&rev, &mut ctx, &mut check).unwrap();
            worker.join().unwrap();
            assert_eq!(check["reasons"], json!(["source_unstable"]));
            assert_eq!(check["state"], "failed");
            assert_eq!(check["hash_check"], true);
            assert_eq!(check["size_check"], true);
            assert_eq!(ctx.reserved, 4);
            assert_eq!(ctx.read, 3);
        }
    }
    #[test]
    fn synchronized_package_ancestor_replacement_with_same_pinned_root() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("parent");
        let package = parent.join("package");
        fs::create_dir_all(package.join("objects")).unwrap();
        fs::write(package.join("objects").join(digest(b"abc")), b"abc").unwrap();
        let source = Source {
            root: Root::at(&package).unwrap(),
            descriptor: json!({}),
            raw: vec![],
        };
        let rev = json!({"content_sha256":digest(b"abc"),"size_bytes":"3"});
        let mut check = json!({"state":"not_attempted","actual_revision":null,"hash_check":null,"size_check":null,"reasons":[]});
        let gate = Arc::new(Barrier::new(2));
        let other = gate.clone();
        let old = temp.path().join("old");
        let worker = thread::spawn(move || {
            other.wait();
            fs::rename(&parent, &old).unwrap();
            fs::create_dir(&parent).unwrap();
            fs::rename(old.join("package"), parent.join("package")).unwrap();
            other.wait();
        });
        let flag = AtomicBool::new(false);
        let mut ctx = Context::new(Limits::default(), &flag);
        let mut fired = false;
        ctx.hook = Some(Box::new(move |event| {
            if event == "object_opened" && !fired {
                fired = true;
                gate.wait();
                gate.wait();
            }
            false
        }));
        source.object(&rev, &mut ctx, &mut check).unwrap();
        worker.join().unwrap();
        assert_eq!(check["hash_check"], true);
        assert_eq!(check["reasons"], json!(["source_unstable"]));
    }

    #[test]
    fn reservation_never_refunded_and_chunk_checkpoint() {
        let (temp, s, rev, mut check) = setup(b"abc");
        fs::remove_file(temp.path().join("objects").join(digest(b"abc"))).unwrap();
        let c = AtomicBool::new(false);
        let mut limits = Limits::default();
        limits.values.insert("max_total_object_bytes".into(), 4);
        let mut ctx = Context::new(limits, &c);
        s.object(&rev, &mut ctx, &mut check).unwrap();
        assert_eq!(ctx.reserved, 4);
        assert!(!ctx.reserve_bytes(0));
        let (_temp, s, rev, mut check) = setup(&vec![b'a'; 65537]);
        let flag = AtomicBool::new(false);
        let mut ctx = Context::new(Limits::default(), &flag);
        let mut chunks = 0;
        ctx.hook = Some(Box::new(|event| {
            if event == "read_chunk" {
                chunks += 1;
            }
            false
        }));
        s.object(&rev, &mut ctx, &mut check).unwrap();
        drop(ctx);
        assert_eq!(chunks, 3);
    }
}
