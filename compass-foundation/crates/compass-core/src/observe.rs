//! Linux observation of a live tree. All path resolution is relative to the
//! caller's pinned root descriptor; symlinks are never traversed.
//!
//! `openat2` is required (Linux 5.6+). There is intentionally no check-then-open
//! fallback. Metadata checks detect common races, not an atomic tree snapshot.

use crate::model::{Coverage, Entry, Issue, Policy, Snapshot, Stats, Workspace, path_key};
use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::{CStr, CString};
use std::fs::{File, Metadata};
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::fs::MetadataExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const BUFFER_BYTES: usize = 64 * 1024;
const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
const RESOLVE_NO_SYMLINKS: u64 = 0x04;
const RESOLVE_BENEATH: u64 = 0x08;

#[repr(C)]
struct OpenHow {
    flags: u64,
    mode: u64,
    resolve: u64,
}

/// Open a path from the authorized root in one kernel resolution operation.
/// O_PATH|O_NOFOLLOW may return the final link itself, never its target.
fn open_beneath(root: RawFd, path: &[u8], flags: i32) -> io::Result<File> {
    let path = CString::new(if path.is_empty() { b"." } else { path })
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in path"))?;
    let how = OpenHow {
        flags: (flags | libc::O_CLOEXEC | libc::O_NOFOLLOW) as u64,
        mode: 0,
        resolve: RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS,
    };
    // SAFETY: path and how remain valid for the syscall. On success, ownership
    // of the fresh descriptor passes exactly once to File.
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            root,
            path.as_ptr(),
            &how,
            std::mem::size_of::<OpenHow>(),
        )
    };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd as RawFd) })
    }
}

fn metadata_beneath(root: RawFd, path: &[u8]) -> io::Result<Metadata> {
    open_beneath(root, path, libc::O_PATH)?.metadata()
}

fn same_object(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev() && a.ino() == b.ino() && a.mode() & libc::S_IFMT == b.mode() & libc::S_IFMT
}

fn unchanged(a: &Metadata, b: &Metadata) -> bool {
    same_object(a, b)
        && a.size() == b.size()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
        && a.mode() == b.mode()
}

fn kind(metadata: &Metadata) -> &'static str {
    let t = metadata.file_type();
    if t.is_file() {
        "file"
    } else if t.is_dir() {
        "directory"
    } else if t.is_symlink() {
        "symlink"
    } else {
        "other"
    }
}

fn failure_state(error: &io::Error) -> &'static str {
    match error.raw_os_error() {
        Some(
            libc::ENOENT | libc::ENOTDIR | libc::ELOOP | libc::EXDEV | libc::ESTALE | libc::EAGAIN,
        ) => "unstable",
        _ => "unreadable",
    }
}

fn blank_entry(path: &[u8], kind: &str, state: &str, reason: &str) -> Entry {
    Entry {
        path_key: path_key(path),
        kind: kind.into(),
        state: state.into(),
        reason: Some(reason.into()),
        content_sha256: None,
        size_bytes: None,
        executable: None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    BeforeMetadata,
    BeforeReadOpen,
    AfterReadChunk,
    BeforeDirectoryOpen,
    AfterDirectoryList,
    BeforeDirectoryFinish,
}

struct Event<'a> {
    phase: Phase,
    path: &'a [u8],
    attempt: u64,
}

trait Hooks {
    fn event(&mut self, event: Event<'_>) -> io::Result<()>;
}

impl<F: FnMut(Event<'_>) -> io::Result<()>> Hooks for F {
    fn event(&mut self, event: Event<'_>) -> io::Result<()> {
        self(event)
    }
}

struct NoHooks;

impl Hooks for NoHooks {
    fn event(&mut self, event: Event<'_>) -> io::Result<()> {
        // Keep hook payloads exercised in production without logging paths or
        // content. Unit tests replace this zero-cost implementation.
        let _ = (event.phase, event.path, event.attempt);
        Ok(())
    }
}

struct Limits {
    file: Option<u64>,
    total: Option<u64>,
    entries: Option<u64>,
    duration: Option<u64>,
}

impl Limits {
    fn from_policy(policy: &Policy) -> Result<Self> {
        fn parse(value: &Option<String>) -> Result<Option<u64>> {
            value
                .as_ref()
                .map(|v| v.parse().map_err(|_| Error::invalid("limit exceeds u64")))
                .transpose()
        }
        Ok(Self {
            file: parse(&policy.max_file_bytes)?,
            total: parse(&policy.max_total_bytes_read)?,
            entries: parse(&policy.max_entries)?,
            duration: parse(&policy.max_duration_ms)?,
        })
    }
}

struct Directory {
    path: Vec<u8>,
    file: File,
    before: Metadata,
    names: Vec<Vec<u8>>,
}

enum Task {
    Entry { path: Vec<u8>, parent: Vec<u8> },
    FinishDirectory(Directory),
}

struct Listing {
    names: Vec<Vec<u8>>,
    incomplete: Option<&'static str>,
}

struct Observer<'a, H> {
    root: RawFd,
    policy: &'a Policy,
    cancel: &'a AtomicBool,
    hooks: H,
    capture: Option<&'a mut dyn ContentCapture>,
    limits: Limits,
    excludes: Vec<Vec<u8>>,
    started: Instant,
    entries: BTreeMap<Vec<u8>, Entry>,
    coverage: BTreeMap<Vec<u8>, Coverage>,
    issues: BTreeMap<Vec<u8>, Vec<Issue>>,
    visited: u64,
    bytes_read: u64,
    read_attempts: u64,
    stop: Option<&'static str>,
}

/// Receives the exact stream hashed by the secure observer. Each attempt begins
/// anew; only stable, accepted revisions receive `finish`. Failed attempts must
/// remain private temporary data and are discarded by the capture implementation.
pub trait ContentCapture {
    fn begin(&mut self, path: &[u8]) -> Result<()>;
    fn chunk(&mut self, bytes: &[u8]) -> Result<()>;
    fn finish(&mut self, entry: &Entry) -> Result<()>;
}

pub fn observe_captured(
    root: &File,
    workspace: &Workspace,
    policy: &Policy,
    cancel: &AtomicBool,
    capture: &mut dyn ContentCapture,
) -> Result<Snapshot> {
    observe_inner(root, workspace, policy, cancel, NoHooks, Some(capture))
}

/// Observe an initialized workspace without reading or executing excluded
/// content. Cancellation is an error, so callers must not publish a manifest.
pub fn observe(
    root: &File,
    workspace: &Workspace,
    policy: &Policy,
    cancel: &AtomicBool,
) -> Result<Snapshot> {
    observe_with_hooks(root, workspace, policy, cancel, NoHooks)
}

fn observe_with_hooks<H: Hooks>(
    root: &File,
    workspace: &Workspace,
    policy: &Policy,
    cancel: &AtomicBool,
    hooks: H,
) -> Result<Snapshot> {
    observe_inner(root, workspace, policy, cancel, hooks, None)
}

fn observe_inner<'a, H: Hooks>(
    root: &File,
    workspace: &Workspace,
    policy: &'a Policy,
    cancel: &'a AtomicBool,
    hooks: H,
    capture: Option<&'a mut dyn ContentCapture>,
) -> Result<Snapshot> {
    workspace.validate()?;
    policy.validate()?;
    let root_metadata = root.metadata().map_err(|e| Error::io("root metadata", e))?;
    if !root_metadata.is_dir() {
        return Err(Error::invalid("workspace root must be a directory"));
    }
    // Probe openat2 before work: unsupported kernels must fail explicitly,
    // rather than misreporting every entry as inaccessible.
    open_beneath(root.as_raw_fd(), b"", libc::O_PATH | libc::O_DIRECTORY)
        .map_err(|e| Error::io("secure root resolution (openat2 required)", e))?;
    let started_at = timestamp()?;
    let mut observer = Observer {
        root: root.as_raw_fd(),
        policy,
        cancel,
        hooks,
        capture,
        limits: Limits::from_policy(policy)?,
        excludes: policy
            .exclude_paths
            .iter()
            .map(|p| crate::model::decode_path_key(p))
            .collect::<Result<_>>()?,
        started: Instant::now(),
        entries: BTreeMap::new(),
        coverage: BTreeMap::new(),
        issues: BTreeMap::new(),
        visited: 0,
        bytes_read: 0,
        read_attempts: 0,
        stop: None,
    };
    observer.check_cancel()?;
    let mut tasks = Vec::new();
    observer.enter_directory(Vec::new(), &root_metadata, &mut tasks)?;
    while let Some(task) = tasks.pop() {
        observer.check_cancel()?;
        match task {
            Task::Entry { path, parent } => observer.visit_entry(path, parent, &mut tasks)?,
            Task::FinishDirectory(directory) => observer.finish_directory(directory)?,
        }
    }
    observer.check_cancel()?;
    let partial = observer.coverage.values().any(|c| c.state != "complete")
        || observer
            .entries
            .values()
            .any(|e| e.state != "observed" && e.state != "excluded");
    let stats = Stats {
        observed_files: observer
            .entries
            .values()
            .filter(|e| e.state == "observed")
            .count()
            .to_string(),
        excluded_entries: observer
            .entries
            .values()
            .filter(|e| e.state == "excluded")
            .count()
            .to_string(),
        unknown_entries: observer
            .entries
            .values()
            .filter(|e| e.state != "observed" && e.state != "excluded")
            .count()
            .to_string(),
        bytes_read: observer.bytes_read.to_string(),
        read_attempts: observer.read_attempts.to_string(),
        duration_ms: observer.started.elapsed().as_millis().to_string(),
    };
    let mut snapshot = Snapshot {
        schema_version: "1".into(),
        contract_version: "0.1.0".into(),
        producer_version: env!("CARGO_PKG_VERSION").into(),
        producer_build: crate::BUILD_ID.into(),
        workspace_id: workspace.workspace_id.clone(),
        snapshot_id: uuid::Uuid::new_v4().to_string(),
        path_namespace: "linux_bytes_v1".into(),
        policy: policy.clone(),
        policy_id: policy.id()?,
        started_at,
        finished_at: timestamp()?,
        consistency: "observation_window".into(),
        quality: if partial { "partial" } else { "complete" }.into(),
        entries: observer.entries.into_values().collect(),
        directory_coverage: observer.coverage.into_values().collect(),
        issues: observer.issues.into_values().flatten().collect(),
        state_digest: None,
        stats,
    };
    if !partial {
        snapshot.state_digest = Some(snapshot.compute_state_digest()?);
    }
    snapshot.validate()?;
    Ok(snapshot)
}

fn timestamp() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| Error::invalid("could not format observation time"))
}

impl<H: Hooks> Observer<'_, H> {
    fn check_cancel(&self) -> Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(Error::cancelled())
        } else {
            Ok(())
        }
    }

    fn time_limit(&self) -> bool {
        self.limits
            .duration
            .is_some_and(|ms| self.started.elapsed().as_millis() >= ms as u128)
    }

    fn hook(&mut self, phase: Phase, path: &[u8], attempt: u64) -> io::Result<()> {
        self.hooks.event(Event {
            phase,
            path,
            attempt,
        })
    }

    fn cover(&mut self, path: &[u8], state: &str, reason: Option<&str>) {
        self.coverage.insert(
            path.to_vec(),
            Coverage {
                path_key: path_key(path),
                state: state.into(),
                reason: reason.map(str::to_owned),
            },
        );
    }

    fn issue(&mut self, path: &[u8], code: &str, message: &str) {
        let issues = self.issues.entry(path.to_vec()).or_default();
        issues.push(Issue {
            path_key: path_key(path),
            ordinal: (issues.len() + 1).to_string(),
            code: code.into(),
            message: message.into(),
        });
    }

    fn excluded_by_policy(&self, path: &[u8], entry_kind: &str) -> bool {
        if self.excludes.iter().any(|excluded| {
            path == excluded
                || (path.starts_with(excluded) && path.get(excluded.len()) == Some(&b'/'))
        }) {
            return true;
        }
        let name = path.rsplit(|b| *b == b'/').next().unwrap_or(path);
        if entry_kind == "directory" {
            self.policy
                .exclude_directory_names
                .iter()
                .any(|p| p.as_bytes() == name)
        } else if entry_kind == "file" {
            self.policy
                .exclude_file_names
                .iter()
                .any(|p| p.as_bytes() == name)
                || self
                    .policy
                    .exclude_file_prefixes
                    .iter()
                    .any(|p| name.starts_with(p.as_bytes()))
        } else {
            false
        }
    }

    fn visit_entry(&mut self, path: Vec<u8>, parent: Vec<u8>, tasks: &mut Vec<Task>) -> Result<()> {
        if self.time_limit() {
            self.stop = Some("max_duration_ms");
        }
        if self
            .limits
            .entries
            .is_some_and(|limit| self.visited >= limit)
        {
            self.stop = Some("max_entries");
        }
        if let Some(reason) = self.stop {
            self.cover(&parent, "not_visited", Some(reason));
            return Ok(());
        }
        self.visited += 1;
        let metadata = self
            .hook(Phase::BeforeMetadata, &path, 0)
            .and_then(|_| metadata_beneath(self.root, &path));
        let metadata = match metadata {
            Ok(metadata) => metadata,
            Err(error) => {
                self.entries.insert(
                    path.clone(),
                    blank_entry(
                        &path,
                        "unknown",
                        failure_state(&error),
                        "metadata_unavailable",
                    ),
                );
                return Ok(());
            }
        };
        let entry_kind = kind(&metadata);
        let exclusion = if entry_kind == "symlink" {
            Some("symlink_not_followed")
        } else if entry_kind == "other" {
            Some("unsupported_kind")
        } else if self.excluded_by_policy(&path, entry_kind) {
            Some("policy_excluded")
        } else {
            None
        };
        if let Some(reason) = exclusion {
            self.entries.insert(
                path.clone(),
                blank_entry(&path, entry_kind, "excluded", reason),
            );
        } else if entry_kind == "directory" {
            self.enter_directory(path, &metadata, tasks)?;
        } else {
            let entry = self.read_file(&path)?;
            self.entries.insert(path, entry);
        }
        Ok(())
    }

    fn enter_directory(
        &mut self,
        path: Vec<u8>,
        expected: &Metadata,
        tasks: &mut Vec<Task>,
    ) -> Result<()> {
        if self.time_limit() {
            self.stop = Some("max_duration_ms");
        }
        if let Some(reason) = self.stop {
            self.cover(&path, "not_visited", Some(reason));
            return Ok(());
        }
        let file = self
            .hook(Phase::BeforeDirectoryOpen, &path, 0)
            .and_then(|_| open_beneath(self.root, &path, libc::O_RDONLY | libc::O_DIRECTORY));
        let file = match file {
            Ok(file) => file,
            Err(error) => {
                self.cover(&path, failure_state(&error), Some("directory_open_failed"));
                return Ok(());
            }
        };
        let before = match file.metadata() {
            Ok(metadata) if same_object(expected, &metadata) => metadata,
            _ => {
                self.cover(&path, "unstable", Some("directory_replaced"));
                return Ok(());
            }
        };
        let listing = self.list_directory(&file);
        self.check_cancel()?;
        let listing = match listing {
            Ok(listing) => listing,
            Err(_) => {
                self.cover(&path, "unreadable", Some("directory_enumeration_failed"));
                return Ok(());
            }
        };
        if let Some(reason) = listing.incomplete {
            self.cover(&path, "not_visited", Some(reason));
            if reason == "max_duration_ms" {
                self.stop = Some(reason);
            }
        } else {
            self.cover(&path, "complete", None);
        }
        if self.hook(Phase::AfterDirectoryList, &path, 0).is_err() {
            self.cover(&path, "unreadable", Some("directory_enumeration_failed"));
            return Ok(());
        }
        let children = listing
            .names
            .iter()
            .rev()
            .map(|name| {
                let mut child = path.clone();
                if !child.is_empty() {
                    child.push(b'/');
                }
                child.extend(name);
                Task::Entry {
                    path: child,
                    parent: path.clone(),
                }
            })
            .collect::<Vec<_>>();
        tasks.push(Task::FinishDirectory(Directory {
            path,
            file,
            before,
            names: listing.names,
        }));
        tasks.extend(children);
        Ok(())
    }

    fn finish_directory(&mut self, directory: Directory) -> Result<()> {
        let Directory {
            path,
            file,
            before,
            names,
        } = directory;
        if self.time_limit() {
            self.stop = Some("max_duration_ms");
        }
        if let Some(reason) = self.stop {
            self.cover(&path, "not_visited", Some(reason));
            return Ok(());
        }
        if self.hook(Phase::BeforeDirectoryFinish, &path, 0).is_err() {
            self.cover(&path, "unreadable", Some("directory_recheck_failed"));
            return Ok(());
        }
        let after_listing = self.list_directory(&file);
        self.check_cancel()?;
        let after_listing = match after_listing {
            Ok(listing) => listing,
            Err(_) => {
                self.cover(&path, "unreadable", Some("directory_recheck_failed"));
                return Ok(());
            }
        };
        if let Some(reason) = after_listing.incomplete {
            self.cover(&path, "not_visited", Some(reason));
            return Ok(());
        }
        let stable = file
            .metadata()
            .ok()
            .zip(metadata_beneath(self.root, &path).ok())
            .is_some_and(|(after, named)| unchanged(&before, &after) && unchanged(&after, &named));
        if !stable || names != after_listing.names {
            self.cover(&path, "unstable", Some("directory_changed"));
        }
        Ok(())
    }

    fn list_directory(&self, file: &File) -> io::Result<Listing> {
        // A new open description avoids sharing offsets with earlier scans.
        let scan = open_beneath(file.as_raw_fd(), b"", libc::O_RDONLY | libc::O_DIRECTORY)?;
        let fd = scan.into_raw_fd();
        // SAFETY: fd is a valid, owned directory descriptor. fdopendir owns it
        // only on success; the failure path closes it explicitly.
        let dir = unsafe { libc::fdopendir(fd) };
        if dir.is_null() {
            let error = io::Error::last_os_error();
            unsafe {
                libc::close(fd);
            }
            return Err(error);
        }
        struct Dir(*mut libc::DIR);
        impl Drop for Dir {
            fn drop(&mut self) {
                unsafe {
                    libc::closedir(self.0);
                }
            }
        }
        let dir = Dir(dir);
        let mut names = Vec::new();
        let mut incomplete = None;
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
            }
            if self.time_limit() {
                incomplete = Some("max_duration_ms");
                break;
            }
            // SAFETY: this thread exclusively owns DIR. readdir's returned
            // name remains valid until the next call and is copied immediately.
            unsafe {
                *libc::__errno_location() = 0;
            }
            let entry = unsafe { libc::readdir(dir.0) };
            if entry.is_null() {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(0) {
                    return Err(error);
                }
                break;
            }
            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
            if name == b"." || name == b".." {
                continue;
            }
            if self
                .limits
                .entries
                .is_some_and(|limit| names.len() as u64 >= limit)
            {
                incomplete = Some("max_entries");
                break;
            }
            names.push(name.to_vec());
        }
        names.sort();
        if names.windows(2).any(|w| w[0] == w[1]) {
            return Err(io::Error::other("directory changed during enumeration"));
        }
        Ok(Listing { names, incomplete })
    }

    fn read_file(&mut self, path: &[u8]) -> Result<Entry> {
        for attempt in 1..=2 {
            self.check_cancel()?;
            self.read_attempts += 1;
            let result = self.read_attempt(path, attempt)?;
            if result.state == "unstable" && attempt == 1 {
                self.issue(
                    path,
                    "retry_after_instability",
                    "First attempt changed; retrying once.",
                );
            } else {
                return Ok(result);
            }
        }
        unreachable!("second attempt always returns")
    }

    fn read_attempt(&mut self, path: &[u8], attempt: u64) -> Result<Entry> {
        let before_named = match metadata_beneath(self.root, path) {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => return Ok(blank_entry(path, "file", "unstable", "file_type_changed")),
            Err(error) => {
                return Ok(blank_entry(
                    path,
                    "file",
                    failure_state(&error),
                    "file_metadata_failed",
                ));
            }
        };
        if self
            .limits
            .file
            .is_some_and(|limit| before_named.size() > limit)
        {
            return Ok(blank_entry(
                path,
                "file",
                "limit_exceeded",
                "max_file_bytes",
            ));
        }
        let file = self
            .hook(Phase::BeforeReadOpen, path, attempt)
            .and_then(|_| {
                open_beneath(
                    self.root,
                    path,
                    libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOCTTY,
                )
            });
        let mut file = match file {
            Ok(file) => file,
            Err(error) => {
                return Ok(blank_entry(
                    path,
                    "file",
                    failure_state(&error),
                    "file_open_failed",
                ));
            }
        };
        let before = match file.metadata() {
            Ok(metadata) if metadata.is_file() && unchanged(&before_named, &metadata) => metadata,
            _ => {
                return Ok(blank_entry(
                    path,
                    "file",
                    "unstable",
                    "file_replaced_before_read",
                ));
            }
        };
        if let Some(capture) = self.capture.as_mut() {
            capture.begin(path)?;
        }
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; BUFFER_BYTES];
        let mut count = 0u64;
        while count < before.size() {
            self.check_cancel()?;
            if self.time_limit() {
                self.stop = Some("max_duration_ms");
                return Ok(blank_entry(
                    path,
                    "file",
                    "limit_exceeded",
                    "max_duration_ms",
                ));
            }
            let remaining = before.size() - count;
            let budget = self
                .limits
                .total
                .map_or(u64::MAX, |limit| limit.saturating_sub(self.bytes_read));
            if budget == 0 {
                self.stop = Some("max_total_bytes_read");
                return Ok(blank_entry(
                    path,
                    "file",
                    "limit_exceeded",
                    "max_total_bytes_read",
                ));
            }
            let request = remaining.min(budget).min(BUFFER_BYTES as u64) as usize;
            let read = match file.read(&mut buffer[..request]) {
                Ok(0) => return Ok(blank_entry(path, "file", "unstable", "file_short_read")),
                Ok(read) => read,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    return Ok(blank_entry(
                        path,
                        "file",
                        failure_state(&error),
                        "file_read_failed",
                    ));
                }
            };
            count += read as u64;
            self.bytes_read = self
                .bytes_read
                .checked_add(read as u64)
                .ok_or_else(|| Error::invalid("byte counter overflow"))?;
            hasher.update(&buffer[..read]);
            if let Some(capture) = self.capture.as_mut() {
                capture.chunk(&buffer[..read])?;
            }
            if self.hook(Phase::AfterReadChunk, path, attempt).is_err() {
                return Ok(blank_entry(path, "file", "unreadable", "file_read_failed"));
            }
        }
        self.check_cancel()?;
        let stable = file
            .metadata()
            .ok()
            .zip(metadata_beneath(self.root, path).ok())
            .is_some_and(|(after, named)| {
                unchanged(&before, &after) && unchanged(&after, &named) && count == after.size()
            });
        if !stable {
            return Ok(blank_entry(
                path,
                "file",
                "unstable",
                "file_changed_during_read",
            ));
        }
        let entry = Entry {
            path_key: path_key(path),
            kind: "file".into(),
            state: "observed".into(),
            reason: None,
            content_sha256: Some(format!("{:x}", hasher.finalize())),
            size_bytes: Some(count.to_string()),
            executable: Some(before.mode() & 0o111 != 0),
        };
        if let Some(capture) = self.capture.as_mut() {
            capture.finish(&entry)?;
        }
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::Path;
    use tempfile::TempDir;

    fn workspace() -> Workspace {
        Workspace {
            schema_version: "1".into(),
            contract_version: "0.1.0".into(),
            workspace_id: "10000000-0000-4000-8000-000000000001".into(),
            path_namespace: "linux_bytes_v1".into(),
            created_at: "2026-10-14T00:00:00Z".into(),
        }
    }

    fn run(root: &Path, policy: &Policy) -> Snapshot {
        observe(
            &File::open(root).unwrap(),
            &workspace(),
            policy,
            &AtomicBool::new(false),
        )
        .unwrap()
    }

    fn run_hook<H: Hooks>(root: &Path, policy: &Policy, hook: H) -> Snapshot {
        observe_with_hooks(
            &File::open(root).unwrap(),
            &workspace(),
            policy,
            &AtomicBool::new(false),
            hook,
        )
        .unwrap()
    }

    fn entry<'a>(snapshot: &'a Snapshot, path: &[u8]) -> &'a Entry {
        snapshot
            .entries
            .iter()
            .find(|e| e.path_key == path_key(path))
            .unwrap()
    }

    fn coverage<'a>(snapshot: &'a Snapshot, path: &[u8]) -> &'a Coverage {
        snapshot
            .directory_coverage
            .iter()
            .find(|e| e.path_key == path_key(path))
            .unwrap()
    }

    #[test]
    fn captured_stream_only_finishes_stable_attempt_and_excludes_secrets() {
        #[derive(Default)]
        struct Capture {
            pending: Vec<u8>,
            accepted: Vec<(String, Vec<u8>)>,
            attempts: usize,
        }
        impl ContentCapture for Capture {
            fn begin(&mut self, _: &[u8]) -> Result<()> {
                self.pending.clear();
                self.attempts += 1;
                Ok(())
            }
            fn chunk(&mut self, b: &[u8]) -> Result<()> {
                self.pending.extend_from_slice(b);
                Ok(())
            }
            fn finish(&mut self, e: &Entry) -> Result<()> {
                assert_eq!(
                    e.content_sha256.as_deref(),
                    Some(crate::model::digest(&self.pending).as_str())
                );
                self.accepted
                    .push((e.path_key.clone(), self.pending.clone()));
                Ok(())
            }
        }
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("file"), b"old").unwrap();
        fs::write(root.path().join(".env"), b"never capture").unwrap();
        let mut capture = Capture::default();
        let snapshot = observe_inner(
            &File::open(root.path()).unwrap(),
            &workspace(),
            &Policy::default(),
            &AtomicBool::new(false),
            |event: Event<'_>| {
                if event.phase == Phase::AfterReadChunk
                    && event.path == b"file"
                    && event.attempt == 1
                {
                    fs::write(root.path().join("file"), b"new state")?;
                }
                Ok(())
            },
            Some(&mut capture),
        )
        .unwrap();
        assert_eq!(snapshot.quality, "complete");
        assert_eq!(capture.attempts, 2);
        assert_eq!(
            capture.accepted,
            vec![(path_key(b"file"), b"new state".to_vec())]
        );
    }

    #[test]
    fn known_hash_vectors_and_repeated_observation() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("abc"), b"abc").unwrap();
        fs::write(root.path().join("empty"), b"").unwrap();
        fs::create_dir(root.path().join("empty-dir")).unwrap();
        let one = run(root.path(), &Policy::default());
        let two = run(root.path(), &Policy::default());
        assert_eq!(one.quality, "complete");
        assert_eq!(
            entry(&one, b"abc").content_sha256.as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
        assert_eq!(
            entry(&one, b"empty").content_sha256.as_deref(),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
        assert_eq!(entry(&one, b"empty").size_bytes.as_deref(), Some("0"));
        assert_eq!(coverage(&one, b"empty-dir").state, "complete");
        assert_eq!(one.stats.bytes_read, "3");
        assert_eq!(one.stats.observed_files, "2");
        assert_ne!(one.snapshot_id, two.snapshot_id);
        assert_eq!(one.state_digest, two.state_digest);
    }

    #[test]
    fn exclusions_links_and_fifo_are_not_read() {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        fs::write(outside.path().join("secret"), b"external").unwrap();
        fs::write(root.path().join("regular"), b"data").unwrap();
        for name in [".git", ".compass", ".aws", ".ssh"] {
            fs::create_dir(root.path().join(name)).unwrap();
            fs::write(root.path().join(name).join("hidden"), b"secret").unwrap();
        }
        fs::create_dir(root.path().join("nested")).unwrap();
        fs::write(root.path().join("nested/.env"), b"secret").unwrap();
        fs::write(root.path().join(".env.prod"), b"secret").unwrap();
        symlink(outside.path(), root.path().join("external")).unwrap();
        symlink("regular", root.path().join("internal")).unwrap();
        symlink("cycle", root.path().join("cycle")).unwrap();
        let fifo = CString::new(root.path().join("fifo").as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        let snapshot = run(root.path(), &Policy::default());
        assert_eq!(snapshot.quality, "complete");
        assert_eq!(snapshot.stats.bytes_read, "4");
        assert_eq!(snapshot.stats.observed_files, "1");
        assert_eq!(
            entry(&snapshot, b"external").reason.as_deref(),
            Some("symlink_not_followed")
        );
        assert_eq!(
            entry(&snapshot, b"fifo").reason.as_deref(),
            Some("unsupported_kind")
        );
        assert_eq!(
            entry(&snapshot, b"nested/.env").reason.as_deref(),
            Some("policy_excluded")
        );
        assert!(!snapshot.entries.iter().any(|e| {
            crate::model::decode_path_key(&e.path_key)
                .unwrap()
                .ends_with(b"secret")
        }));
    }

    #[test]
    fn native_names_hardlinks_binary_and_explicit_subtree_exclusions() {
        let root = TempDir::new().unwrap();
        let native = OsString::from_vec(vec![b'n', 0xff]);
        fs::write(root.path().join(&native), [0, 255, 13, 10]).unwrap();
        fs::hard_link(root.path().join(&native), root.path().join("N")).unwrap();
        fs::write(root.path().join("n"), b"x").unwrap();
        fs::create_dir(root.path().join("skip")).unwrap();
        fs::write(root.path().join("skip/file"), b"hidden").unwrap();
        fs::write(root.path().join("skipped"), b"eligible").unwrap();
        let policy = Policy {
            exclude_paths: vec![path_key(b"skip")],
            ..Policy::default()
        };
        let snapshot = run(root.path(), &policy);
        assert_eq!(snapshot.quality, "complete");
        assert_eq!(snapshot.stats.observed_files, "4");
        assert_eq!(
            entry(&snapshot, &[b'n', 255]).content_sha256,
            entry(&snapshot, b"N").content_sha256
        );
        assert_ne!(
            entry(&snapshot, b"N").path_key,
            entry(&snapshot, b"n").path_key
        );
        assert_eq!(entry(&snapshot, b"skip").state, "excluded");
        assert_eq!(entry(&snapshot, b"skipped").state, "observed");
    }

    #[test]
    fn modifications_during_both_attempts_never_accept_a_revision() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("file"), b"first").unwrap();
        let mut interventions = 0;
        let snapshot = run_hook(root.path(), &Policy::default(), |event: Event<'_>| {
            if event.phase == Phase::AfterReadChunk && event.path == b"file" {
                interventions += 1;
                fs::write(
                    root.path().join("file"),
                    if event.attempt == 1 {
                        b"second".as_slice()
                    } else {
                        b"third-state".as_slice()
                    },
                )?;
            }
            Ok(())
        });
        assert_eq!(interventions, 2);
        assert_eq!(snapshot.quality, "partial");
        assert_eq!(entry(&snapshot, b"file").state, "unstable");
        assert_eq!(entry(&snapshot, b"file").content_sha256, None);
        assert_eq!(snapshot.stats.read_attempts, "2");
        assert_eq!(snapshot.stats.bytes_read, "11");
        assert_eq!(snapshot.issues[0].code, "retry_after_instability");
        assert_eq!(snapshot.state_digest, None);
    }

    #[test]
    fn successful_retry_records_only_second_revision_and_all_read_bytes() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("file"), b"old").unwrap();
        let snapshot = run_hook(root.path(), &Policy::default(), |event: Event<'_>| {
            if event.phase == Phase::AfterReadChunk && event.path == b"file" && event.attempt == 1 {
                fs::write(root.path().join("file"), b"new state")?;
            }
            Ok(())
        });
        assert_eq!(snapshot.quality, "complete");
        assert_eq!(
            entry(&snapshot, b"file").content_sha256,
            Some(crate::model::digest(b"new state"))
        );
        assert_eq!(snapshot.stats.bytes_read, "12");
        assert_eq!(snapshot.stats.read_attempts, "2");
        assert_eq!(snapshot.issues.len(), 1);
    }

    #[test]
    fn symlink_substitution_before_open_never_reads_external_target() {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        fs::create_dir(root.path().join("dir")).unwrap();
        fs::write(root.path().join("dir/file"), b"inside").unwrap();
        fs::write(outside.path().join("file"), b"external secret").unwrap();
        let snapshot = run_hook(root.path(), &Policy::default(), |event: Event<'_>| {
            if event.phase == Phase::BeforeReadOpen
                && event.path == b"dir/file"
                && event.attempt == 1
            {
                fs::rename(root.path().join("dir"), root.path().join("moved"))?;
                symlink(outside.path(), root.path().join("dir"))?;
            }
            Ok(())
        });
        assert_eq!(snapshot.quality, "partial");
        assert_eq!(snapshot.stats.bytes_read, "0");
        assert_eq!(entry(&snapshot, b"dir/file").state, "unstable");
        assert_eq!(entry(&snapshot, b"dir/file").content_sha256, None);
        assert_eq!(coverage(&snapshot, b"dir").state, "unstable");
    }

    #[test]
    fn file_replaced_by_fifo_is_rejected_without_blocking_or_reading() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("file"), b"inside").unwrap();
        let snapshot = run_hook(root.path(), &Policy::default(), |event: Event<'_>| {
            if event.phase == Phase::BeforeReadOpen && event.path == b"file" && event.attempt == 1 {
                fs::remove_file(root.path().join("file"))?;
                let fifo =
                    CString::new(root.path().join("file").as_os_str().as_encoded_bytes()).unwrap();
                if unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) } != 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        });
        assert_eq!(snapshot.quality, "partial");
        assert_eq!(snapshot.stats.bytes_read, "0");
        assert_eq!(entry(&snapshot, b"file").state, "unstable");
    }

    #[test]
    fn streaming_preserves_binary_bytes_and_mtime_is_not_content_identity() {
        let root = TempDir::new().unwrap();
        let bytes: Vec<u8> = (0..BUFFER_BYTES * 3 + 7).map(|i| (i % 256) as u8).collect();
        let path = root.path().join("file");
        fs::write(&path, &bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let first = run(root.path(), &Policy::default());
        assert_eq!(
            entry(&first, b"file").content_sha256,
            Some(crate::model::digest(&bytes))
        );
        assert_eq!(first.stats.bytes_read, bytes.len().to_string());
        assert_eq!(entry(&first, b"file").executable, Some(false));
        File::open(&path)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(std::time::UNIX_EPOCH))
            .unwrap();
        let touched = run(root.path(), &Policy::default());
        assert_eq!(first.state_digest, touched.state_digest);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        let executable = run(root.path(), &Policy::default());
        assert_eq!(entry(&executable, b"file").executable, Some(true));
        assert_ne!(touched.state_digest, executable.state_digest);
    }

    #[test]
    fn inaccessible_file_and_directory_preserve_unknown_regions() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("file"), b"data").unwrap();
        fs::create_dir(root.path().join("dir")).unwrap();
        fs::write(root.path().join("dir/hidden"), b"data").unwrap();
        let snapshot = run_hook(root.path(), &Policy::default(), |event: Event<'_>| {
            if (event.phase == Phase::BeforeReadOpen && event.path == b"file")
                || (event.phase == Phase::BeforeDirectoryOpen && event.path == b"dir")
            {
                return Err(io::Error::from_raw_os_error(libc::EACCES));
            }
            Ok(())
        });
        assert_eq!(snapshot.quality, "partial");
        assert_eq!(entry(&snapshot, b"file").state, "unreadable");
        assert_eq!(coverage(&snapshot, b"dir").state, "unreadable");
        assert_eq!(snapshot.stats.bytes_read, "0");
        assert_eq!(snapshot.entries.len(), 1);
    }

    #[test]
    fn actual_linux_permission_denial_is_reported() {
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("file"), b"data").unwrap();
        fs::create_dir(root.path().join("dir")).unwrap();
        fs::write(root.path().join("dir/hidden"), b"data").unwrap();
        fs::set_permissions(root.path().join("file"), fs::Permissions::from_mode(0o0)).unwrap();
        fs::set_permissions(root.path().join("dir"), fs::Permissions::from_mode(0o0)).unwrap();
        let snapshot = run(root.path(), &Policy::default());
        fs::set_permissions(root.path().join("dir"), fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(snapshot.quality, "partial");
        assert_eq!(entry(&snapshot, b"file").state, "unreadable");
        assert_eq!(coverage(&snapshot, b"dir").state, "unreadable");
    }

    #[test]
    fn directory_reenumeration_detects_new_names() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join("dir")).unwrap();
        let snapshot = run_hook(root.path(), &Policy::default(), |event: Event<'_>| {
            if event.phase == Phase::BeforeDirectoryFinish && event.path == b"dir" {
                fs::write(root.path().join("dir/new"), b"new")?;
            }
            Ok(())
        });
        assert_eq!(snapshot.quality, "partial");
        assert_eq!(coverage(&snapshot, b"dir").state, "unstable");
        assert_eq!(snapshot.state_digest, None);
    }

    #[test]
    fn file_total_entry_and_duration_limits_are_explicit() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("a"), b"abc").unwrap();
        fs::write(root.path().join("b"), b"def").unwrap();
        let per_file = run(
            root.path(),
            &Policy {
                max_file_bytes: Some("2".into()),
                ..Policy::default()
            },
        );
        assert_eq!(
            entry(&per_file, b"a").reason.as_deref(),
            Some("max_file_bytes")
        );
        assert_eq!(per_file.stats.bytes_read, "0");
        assert_eq!(per_file.quality, "partial");
        let total = run(
            root.path(),
            &Policy {
                max_total_bytes_read: Some("4".into()),
                ..Policy::default()
            },
        );
        assert_eq!(total.stats.bytes_read, "4");
        assert_eq!(entry(&total, b"a").state, "observed");
        assert_eq!(
            entry(&total, b"b").reason.as_deref(),
            Some("max_total_bytes_read")
        );
        assert_eq!(coverage(&total, b"").state, "not_visited");
        let entries = run(
            root.path(),
            &Policy {
                max_entries: Some("1".into()),
                ..Policy::default()
            },
        );
        assert_eq!(entries.entries.len(), 1);
        assert_eq!(entries.quality, "partial");
        assert_eq!(
            coverage(&entries, b"").reason.as_deref(),
            Some("max_entries")
        );
        let timed = run_hook(
            root.path(),
            &Policy {
                max_duration_ms: Some("5".into()),
                ..Policy::default()
            },
            |event: Event<'_>| {
                if event.phase == Phase::AfterDirectoryList && event.path.is_empty() {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(())
            },
        );
        assert_eq!(timed.quality, "partial");
        assert_eq!(
            coverage(&timed, b"").reason.as_deref(),
            Some("max_duration_ms")
        );
    }

    #[test]
    fn exact_byte_budget_accepts_complete_file_and_empty_tree_is_complete() {
        let root = TempDir::new().unwrap();
        let empty = run(root.path(), &Policy::default());
        assert_eq!(empty.quality, "complete");
        assert_eq!(empty.stats.observed_files, "0");
        fs::write(root.path().join("file"), b"abc").unwrap();
        let bounded = run(
            root.path(),
            &Policy {
                max_total_bytes_read: Some("3".into()),
                ..Policy::default()
            },
        );
        assert_eq!(bounded.quality, "complete");
        assert_eq!(bounded.stats.bytes_read, "3");
    }

    #[test]
    fn cancelled_observation_returns_no_snapshot() {
        let root = TempDir::new().unwrap();
        let cancel = AtomicBool::new(true);
        let error = observe(
            &File::open(root.path()).unwrap(),
            &workspace(),
            &Policy::default(),
            &cancel,
        )
        .unwrap_err();
        assert_eq!(error.exit_code, 130);
        cancel.store(false, Ordering::Relaxed);
        fs::write(root.path().join("file"), vec![42; BUFFER_BYTES * 3]).unwrap();
        let error = observe_with_hooks(
            &File::open(root.path()).unwrap(),
            &workspace(),
            &Policy::default(),
            &cancel,
            |event: Event<'_>| {
                if event.phase == Phase::AfterReadChunk {
                    cancel.store(true, Ordering::Relaxed);
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(error.exit_code, 130);
    }

    #[test]
    fn pinned_root_survives_path_rename_without_following_replacement() {
        let base = TempDir::new().unwrap();
        fs::create_dir(base.path().join("root")).unwrap();
        fs::create_dir(base.path().join("other")).unwrap();
        fs::write(base.path().join("root/file"), b"authorized").unwrap();
        fs::write(base.path().join("other/file"), b"external").unwrap();
        let root = File::open(base.path().join("root")).unwrap();
        fs::rename(base.path().join("root"), base.path().join("moved")).unwrap();
        symlink("other", base.path().join("root")).unwrap();
        let snapshot = observe(
            &root,
            &workspace(),
            &Policy::default(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(
            entry(&snapshot, b"file").content_sha256,
            Some(crate::model::digest(b"authorized"))
        );
        assert_eq!(snapshot.quality, "complete");
    }
}
