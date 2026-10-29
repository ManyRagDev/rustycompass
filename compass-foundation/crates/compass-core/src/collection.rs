//! Explicit local collections: immutable generations, captured CAS objects and
//! a persistent SQLite FTS5 index. The live tree is only read by the observer.
use crate::{
    Error, Result, codec,
    model::{self, Entry, Policy, Snapshot, Workspace},
    observe::{self, ContentCapture},
    storage,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rusqlite::{Connection, OpenFlags, params};
use rustix::fs::{self as rfs, FlockOperation, Mode, OFlags, ResolveFlags};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::{
            ffi::{OsStrExt, OsStringExt},
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct PrepareRequest {
    pub root: PathBuf,
    pub store: PathBuf,
    pub max_disk_bytes: u64,
}
#[derive(Clone, Serialize, Deserialize)]
struct Config {
    version: u32,
    root_base64: String,
    max_disk_bytes: u64,
    workspace: Workspace,
    policy: Policy,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceDescriptor {
    source_schema_version: String,
    source_contract_version: String,
    workspace_id: String,
    snapshot_id: String,
    policy_id: String,
    snapshot_file_sha256: String,
}
pub struct VerifiedDocument {
    pub path_key: String,
    pub content_sha256: String,
    pub bytes: Vec<u8>,
}
pub struct Collection {
    store: PathBuf,
    generation: String,
    generation_dir: PathBuf,
    source: PathBuf,
    snapshot_path: PathBuf,
    pub snapshot: Snapshot,
    config: Config,
    index_hash: String,
    _root: File,
    directory: File,
}
const RESOLVE: ResolveFlags = ResolveFlags::BENEATH
    .union(ResolveFlags::NO_SYMLINKS)
    .union(ResolveFlags::NO_MAGICLINKS);
fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::cancelled())
    } else {
        Ok(())
    }
}
fn io<T>(what: &str, r: std::io::Result<T>) -> Result<T> {
    r.map_err(|e| Error::io(what, e))
}
fn sql<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|e| Error::corrupt(format!("collection index: {e}")))
}
fn at(dir: &File, path: &str, flags: OFlags) -> Result<File> {
    rfs::openat2(
        dir,
        path,
        flags | OFlags::CLOEXEC,
        if flags.contains(OFlags::CREATE) {
            Mode::from_raw_mode(0o600)
        } else {
            Mode::empty()
        },
        RESOLVE,
    )
    .map(File::from)
    .map_err(|e| Error::io("open collection record", e.into()))
}
fn pinned(f: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", f.as_raw_fd()))
}
fn open_root(path: &Path) -> Result<File> {
    io(
        "open collection directory",
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path),
    )
}
fn read_at(dir: &File, path: &str) -> Result<Vec<u8>> {
    let mut f = at(dir, path, OFlags::RDONLY | OFlags::NONBLOCK)?;
    let meta = io("inspect collection record", f.metadata())?;
    if !meta.is_file() || meta.len() > 256 * 1024 * 1024 {
        return Err(Error::corrupt(
            "collection record must be a regular file at most 256MiB",
        ));
    }
    let mut b = Vec::new();
    io("read collection record", f.read_to_end(&mut b))?;
    Ok(b)
}
fn hash_at(dir: &File, path: &str) -> Result<String> {
    let mut f = at(dir, path, OFlags::RDONLY | OFlags::NONBLOCK)?;
    if !io("inspect hashed record", f.metadata())?.is_file() {
        return Err(Error::corrupt("hashed record must be regular"));
    }
    let mut h = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = io("hash collection record", f.read(&mut buffer))?;
        if n == 0 {
            break;
        }
        h.update(&buffer[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn mkdir(dir: &File, name: &str) -> Result<()> {
    match rfs::mkdirat(dir, name, Mode::from_raw_mode(0o700)) {
        Ok(()) | Err(rustix::io::Errno::EXIST) => {}
        Err(e) => return Err(Error::io("create collection directory", e.into())),
    }
    at(dir, name, OFlags::RDONLY | OFlags::DIRECTORY)?;
    Ok(())
}
fn disk_usage(path: &Path) -> Result<u64> {
    fn walk(p: &Path, seen: &mut HashSet<(u64, u64)>) -> Result<u64> {
        let m = io("inspect collection usage", fs::symlink_metadata(p))?;
        if m.file_type().is_symlink() {
            return Err(Error::corrupt("symlink inside collection store"));
        }
        if m.is_file() {
            return Ok(if seen.insert((m.dev(), m.ino())) {
                m.len()
            } else {
                0
            });
        }
        if !m.is_dir() {
            return Err(Error::corrupt("unsupported file inside collection store"));
        }
        let mut n = 0u64;
        for e in io("list collection usage", fs::read_dir(p))? {
            n = n
                .checked_add(walk(&io("list collection usage", e)?.path(), seen)?)
                .ok_or_else(|| Error::invalid("disk usage overflow"))?;
        }
        Ok(n)
    }
    walk(&path.join("."), &mut HashSet::new())
}
fn reserve(path: &Path, max: u64, bytes: u64) -> Result<()> {
    if disk_usage(path)?.checked_add(bytes).is_none_or(|n| n > max) {
        Err(Error::invalid(
            "collection max_disk_bytes exceeded (including staging); retained generations were preserved",
        ))
    } else {
        Ok(())
    }
}
struct Writer {
    root: File,
    _lock: File,
    path: PathBuf,
}
impl Writer {
    fn open(path: &Path, cancel: &AtomicBool) -> Result<Self> {
        check(cancel)?;
        let root = open_root(path)?;
        let lock = at(
            &root,
            "writer.lock",
            OFlags::RDWR | OFlags::CREATE | OFlags::NONBLOCK,
        )?;
        if !io("inspect writer lock", lock.metadata())?.is_file() {
            return Err(Error::corrupt("writer lock must be regular"));
        }
        loop {
            check(cancel)?;
            match rfs::flock(&lock, FlockOperation::NonBlockingLockExclusive) {
                Ok(()) => break,
                Err(rustix::io::Errno::WOULDBLOCK) => {
                    std::thread::sleep(std::time::Duration::from_millis(20))
                }
                Err(e) => return Err(Error::io("lock collection", e.into())),
            }
        }
        let path = pinned(&root);
        // Config/pointer files left before atomic rename are private work, not
        // retained revisions. Reconcile them only while owning the OS lock.
        for entry in io("list publication temporaries", fs::read_dir(&path))? {
            let entry = io("list publication temporaries", entry)?;
            let name = entry.file_name();
            if name.as_bytes().starts_with(b".config-") || name.as_bytes().starts_with(b".current-")
            {
                if !io("inspect publication temporary", entry.file_type())?.is_file() {
                    return Err(Error::corrupt("invalid publication temporary"));
                }
                rfs::unlinkat(&root, &name, rfs::AtFlags::empty()).map_err(|e| {
                    Error::io("discard interrupted publication temporary", e.into())
                })?;
            }
        }
        Ok(Self {
            root,
            _lock: lock,
            path,
        })
    }
}
fn write_new(store: &Path, max: u64, path: &Path, bytes: &[u8]) -> Result<()> {
    reserve(store, max, bytes.len() as u64)?;
    let mut f = io(
        "create collection record",
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path),
    )?;
    io("write collection record", f.write_all(bytes))?;
    io("sync collection record", f.sync_all())
}

impl Collection {
    pub fn open(store: &Path, generation: Option<&str>) -> Result<Self> {
        let store = io("resolve collection store", store.canonicalize())?;
        let root = open_root(&store)?;
        let config: Config = codec::decode(&read_at(&root, "collection.json")?)?;
        if config.version != 1 || config.max_disk_bytes == 0 {
            return Err(Error::corrupt("unsupported collection configuration"));
        }
        config.workspace.validate()?;
        config.policy.validate()?;
        let generation = match generation {
            Some(g) => g.to_owned(),
            None => String::from_utf8(read_at(&root, "current")?)
                .map_err(|_| Error::corrupt("invalid generation pointer"))?
                .trim()
                .to_owned(),
        };
        uuid::Uuid::parse_str(&generation)
            .map_err(|_| Error::invalid("generation must be a UUID"))?;
        let directory = at(
            &root,
            &format!("generations/{generation}"),
            OFlags::RDONLY | OFlags::DIRECTORY,
        )?;
        let snapshot: Snapshot = codec::decode(&read_at(&directory, "snapshot.json")?)?;
        snapshot.validate()?;
        if snapshot.snapshot_id != generation
            || snapshot.workspace_id != config.workspace.workspace_id
        {
            return Err(Error::corrupt("generation identity mismatch"));
        }
        let source_record: SourceDescriptor =
            codec::decode(&read_at(&directory, "source/source.json")?)?;
        if source_record.source_schema_version != "1"
            || source_record.source_contract_version != "0.1.0"
            || source_record.workspace_id != snapshot.workspace_id
            || source_record.snapshot_id != snapshot.snapshot_id
            || source_record.policy_id != snapshot.policy_id
            || source_record.snapshot_file_sha256
                != model::digest(&read_at(&directory, "snapshot.json")?)
        {
            return Err(Error::corrupt("generation source identity mismatch"));
        }
        let receipt: Value = codec::decode(&read_at(&directory, "generation.json")?)?;
        if receipt["index_sha256"] != hash_at(&directory, "index.sqlite")? {
            return Err(Error::corrupt("collection index hash mismatch"));
        }
        let generation_dir = store.join("generations").join(&generation);
        let source = generation_dir.join("source");
        let snapshot_path = generation_dir.join("snapshot.json");
        Ok(Self {
            store,
            generation,
            generation_dir,
            source,
            snapshot_path,
            snapshot,
            config,
            index_hash: receipt["index_sha256"]
                .as_str()
                .ok_or_else(|| Error::corrupt("invalid index receipt"))?
                .to_owned(),
            _root: root,
            directory,
        })
    }
    pub fn store_path(&self) -> &Path {
        &self.store
    }
    pub fn generation_dir(&self) -> &Path {
        &self.generation_dir
    }
    pub fn pinned_generation_dir(&self) -> PathBuf {
        pinned(&self.directory)
    }
    pub fn pinned_store_path(&self) -> PathBuf {
        pinned(&self._root).join(".")
    }
    pub fn pinned_snapshot_path(&self) -> PathBuf {
        pinned(&self.directory).join("snapshot.json")
    }
    pub fn pinned_source_path(&self) -> PathBuf {
        pinned(&self.directory).join("source")
    }

    pub fn generation_id(&self) -> &str {
        &self.generation
    }
    pub fn snapshot_path(&self) -> &Path {
        &self.snapshot_path
    }
    pub fn source_path(&self) -> &Path {
        &self.source
    }
    pub fn max_disk_bytes(&self) -> u64 {
        self.config.max_disk_bytes
    }
    pub fn reserve_storage(&self, bytes: u64) -> Result<()> {
        reserve(&pinned(&self._root), self.config.max_disk_bytes, bytes)
    }
    pub fn with_writer<T>(&self, cancel: &AtomicBool, f: impl FnOnce() -> Result<T>) -> Result<T> {
        let _lock = Writer::open(&self.pinned_store_path(), cancel)?;
        f()
    }
    fn object(&self, entry: &Entry, cancel: &AtomicBool) -> Result<Vec<u8>> {
        check(cancel)?;
        let hash = entry
            .content_sha256
            .as_ref()
            .ok_or_else(|| Error::corrupt("entry lacks content revision"))?;
        let mut f = at(
            &self.directory,
            &format!("source/objects/{hash}"),
            OFlags::RDONLY | OFlags::NONBLOCK,
        )?;
        let before = io("inspect captured object", f.metadata())?;
        let expected = entry
            .size_bytes
            .as_ref()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| Error::corrupt("invalid captured object size"))?;
        if !before.is_file() || before.len() != expected {
            return Err(Error::corrupt("captured object size/type mismatch"));
        }
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 65536];
        loop {
            check(cancel)?;
            let n = io("read captured object", f.read(&mut buffer))?;
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..n]);
            if bytes.len() as u64 > expected {
                return Err(Error::corrupt("captured object grew"));
            }
        }
        let after = io("reinspect captured object", f.metadata())?;
        let named = io(
            "reinspect named captured object",
            at(
                &self.directory,
                &format!("source/objects/{hash}"),
                OFlags::RDONLY | OFlags::NONBLOCK,
            )?
            .metadata(),
        )?;
        if named.dev() != before.dev()
            || named.ino() != before.ino()
            || named.len() != before.len()
            || bytes.len() as u64 != expected
            || before.ino() != after.ino()
            || before.size() != after.size()
            || before.mtime_nsec() != after.mtime_nsec()
            || before.mtime() != after.mtime()
            || before.ctime() != after.ctime()
            || before.ctime_nsec() != after.ctime_nsec()
            || model::digest(&bytes) != *hash
        {
            return Err(Error::corrupt("captured object hash/stability mismatch"));
        }
        Ok(bytes)
    }
    pub fn verified_document(
        &self,
        path_key: &str,
        cancel: &AtomicBool,
    ) -> Result<VerifiedDocument> {
        let e = self
            .snapshot
            .entries
            .iter()
            .find(|e| e.path_key == path_key && e.state == "observed")
            .ok_or_else(|| Error::invalid("path_key not observed in generation"))?;
        Ok(VerifiedDocument {
            path_key: e.path_key.clone(),
            content_sha256: e.content_sha256.clone().unwrap(),
            bytes: self.object(e, cancel)?,
        })
    }
    pub fn verified_documents(&self, cancel: &AtomicBool) -> Result<Vec<VerifiedDocument>> {
        self.snapshot
            .entries
            .iter()
            .filter(|e| e.state == "observed")
            .map(|e| {
                Ok(VerifiedDocument {
                    path_key: e.path_key.clone(),
                    content_sha256: e.content_sha256.clone().unwrap(),
                    bytes: self.object(e, cancel)?,
                })
            })
            .collect()
    }
    fn connection(&self) -> Result<Connection> {
        if hash_at(&self.directory, "index.sqlite")? != self.index_hash {
            return Err(Error::corrupt("collection index hash mismatch"));
        }
        sql(Connection::open_with_flags(
            pinned(&self.directory).join("index.sqlite"),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ))
    }
    pub fn files(&self, query: &str, limit: usize, offset: usize) -> Result<Value> {
        validate_page(limit, offset)?;
        let needle = query.to_lowercase();
        let entries: Vec<_> = self
            .snapshot
            .entries
            .iter()
            .filter(|e| e.state == "observed")
            .filter(|e| display_path(&e.path_key).to_lowercase().contains(&needle))
            .collect();
        let results:Vec<_>=entries.iter().skip(offset).take(limit).map(|e|json!({"path_key":e.path_key,"path":display_path(&e.path_key),"content_sha256":e.content_sha256,"size_bytes":e.size_bytes})).collect();
        Ok(
            json!({"generation":self.generation,"snapshot_id":self.generation,"quality":self.snapshot.quality,"results":results,"total":entries.len(),"offset":offset,"limit":limit}),
        )
    }
    pub fn query(&self, query: &str, limit: usize, offset: usize) -> Result<Value> {
        self.query_cancel(query, limit, offset, &AtomicBool::new(false))
    }
    pub fn query_cancel(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        cancel: &AtomicBool,
    ) -> Result<Value> {
        check(cancel)?;
        if query.len() > 32768 {
            return Err(Error::invalid("query exceeds 32768 bytes"));
        }
        validate_page(limit, offset)?;
        let terms = segment_identifiers(query);
        if terms.trim().is_empty() {
            return Err(Error::invalid(
                "lexical query must contain searchable terms",
            ));
        }
        let expression = terms
            .split_whitespace()
            .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        let db = self.connection()?;
        let finished = AtomicBool::new(false);
        let interrupt = db.get_interrupt_handle();
        let result = std::thread::scope(|scope| {
            scope.spawn(|| {
                while !finished.load(Ordering::Relaxed) {
                    if cancel.load(Ordering::Relaxed) {
                        interrupt.interrupt();
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            });
            let result = (|| {
                let mut statement=sql(db.prepare("SELECT path_key, hash, start_line, end_line, start_byte, end_byte, bm25(search,3.0,1.0,2.0) FROM search WHERE search MATCH ?1 ORDER BY bm25(search,3.0,1.0,2.0), path_key, start_byte LIMIT ?2 OFFSET ?3"))?;
                let rows = sql(statement.query_map(
                    params![expression, limit as i64, offset as i64],
                    |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, u64>(2)?,
                            r.get::<_, u64>(3)?,
                            r.get::<_, u64>(4)?,
                            r.get::<_, u64>(5)?,
                            r.get::<_, f64>(6)?,
                        ))
                    },
                ))?;
                let mut verified = BTreeMap::new();
                let mut results = Vec::new();
                let mut output_bytes = 0usize;
                for row in rows {
                    check(cancel)?;
                    let (path, hash, start, end, a, b, score) = sql(row)?;
                    let e = self
                        .snapshot
                        .entries
                        .iter()
                        .find(|e| {
                            e.path_key == path && e.content_sha256.as_deref() == Some(hash.as_str())
                        })
                        .ok_or_else(|| Error::corrupt("indexed revision absent from snapshot"))?;
                    if !verified.contains_key(&path) {
                        verified.insert(path.clone(), self.object(e, cancel)?);
                    }
                    let raw = &verified[&path];
                    let spans = chunk_spans(raw);
                    if !spans.contains(&(start, end, a, b)) {
                        return Err(Error::corrupt(
                            "indexed span does not match verified content",
                        ));
                    }
                    output_bytes = output_bytes.saturating_add((b - a) as usize);
                    if output_bytes > 8 * 1024 * 1024 {
                        return Err(Error::invalid("lexical excerpts exceed 8MiB; reduce limit"));
                    }
                    let text = std::str::from_utf8(&raw[a as usize..b as usize])
                        .map_err(|_| Error::corrupt("indexed non-text object"))?;
                    results.push(json!({"path_key":path,"path":display_path(&e.path_key),"content_sha256":hash,"start_line":start.to_string(),"end_line":end.to_string(),"start_byte":a.to_string(),"end_byte":b.to_string(),"text":text,"score":score}));
                }
                if hash_at(&self.directory, "index.sqlite")? != self.index_hash {
                    return Err(Error::corrupt("collection index changed during query"));
                }
                check(cancel)?;
                Ok(
                    json!({"generation":self.generation,"snapshot_id":self.generation,"quality":self.snapshot.quality,"index":"sqlite_fts5_bm25_v1","results":results,"limit":limit,"offset":offset}),
                )
            })();
            finished.store(true, Ordering::Relaxed);
            result
        });
        check(cancel)?;
        result
    }
}
fn validate_page(limit: usize, offset: usize) -> Result<()> {
    if limit == 0 || limit > 1000 || offset > i64::MAX as usize {
        return Err(Error::invalid(
            "limit must be 1..1000 and offset must fit i64",
        ));
    }
    Ok(())
}
fn display_path(key: &str) -> String {
    model::decode_path_key(key)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}
/// Split snake_case, camelCase and acronym boundaries while retaining whole
/// identifiers, so both exact names and their component words are searchable.
pub fn segment_identifiers(text: &str) -> String {
    let mut out = String::new();
    for token in text.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if token.is_empty() {
            continue;
        }
        out.push_str(token);
        out.push(' ');
        let chars: Vec<char> = token.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if *c == '_' {
                out.push(' ');
                continue;
            }
            if i > 0
                && c.is_uppercase()
                && (chars[i - 1].is_lowercase()
                    || (chars[i - 1].is_uppercase()
                        && chars.get(i + 1).is_some_and(|n| n.is_lowercase())))
            {
                out.push(' ');
            }
            out.extend(c.to_lowercase());
        }
        out.push(' ');
    }
    out
}
fn chunk_spans(bytes: &[u8]) -> Vec<(u64, u64, u64, u64)> {
    if bytes.contains(&0) || std::str::from_utf8(bytes).is_err() {
        return Vec::new();
    }
    let mut spans = Vec::new();
    let mut start = 0;
    let mut first = 1;
    let mut line = 1;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            if line - first + 1 >= 80 {
                spans.push((first, line, start as u64, (i + 1) as u64));
                start = i + 1;
                first = line + 1;
            }
            line += 1;
        }
    }
    if start < bytes.len() {
        spans.push((
            first,
            if bytes.last() == Some(&b'\n') {
                line - 1
            } else {
                line
            },
            start as u64,
            bytes.len() as u64,
        ));
    }
    if bytes.is_empty() {
        spans.push((1, 1, 0, 0));
    }
    spans
}

struct Capture<'a> {
    store: &'a Path,
    max: u64,
    temp: &'a Path,
    current: Option<File>,
    objects: File,
    reused: u64,
    added: u64,
}
impl ContentCapture for Capture<'_> {
    fn begin(&mut self, _: &[u8]) -> Result<()> {
        self.current.take();
        if self.temp.exists() {
            io("discard failed capture attempt", fs::remove_file(self.temp))?;
        }
        self.current = Some(io(
            "create capture temporary",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .custom_flags(libc::O_NOFOLLOW)
                .open(self.temp),
        )?);
        Ok(())
    }
    fn chunk(&mut self, bytes: &[u8]) -> Result<()> {
        reserve(self.store, self.max, bytes.len() as u64)?;
        io(
            "capture observed stream",
            self.current.as_mut().unwrap().write_all(bytes),
        )
    }
    fn finish(&mut self, e: &Entry) -> Result<()> {
        let f = self.current.take().unwrap();
        io("sync captured stream", f.sync_all())?;
        drop(f);
        let object = pinned(&self.objects).join(e.content_sha256.as_ref().unwrap());
        if object.exists() {
            let m = io("inspect existing CAS object", fs::symlink_metadata(&object))?;
            if !m.is_file()
                || hash_at(&self.objects, e.content_sha256.as_ref().unwrap())?
                    != *e.content_sha256.as_ref().unwrap()
            {
                return Err(Error::corrupt("existing CAS object is corrupt"));
            }
            io("discard duplicate capture", fs::remove_file(self.temp))?;
            self.reused += 1;
        } else {
            rfs::renameat_with(
                rfs::CWD,
                self.temp,
                &self.objects,
                e.content_sha256.as_ref().unwrap().as_str(),
                rfs::RenameFlags::NOREPLACE,
            )
            .map_err(|e| Error::io("publish CAS object", e.into()))?;
            self.added += 1;
        }
        Ok(())
    }
}

pub fn prepare(request: &PrepareRequest, cancel: &AtomicBool) -> Result<Value> {
    let result = prepare_staged(request, cancel)?;
    publish_current(
        &request.store,
        result["generation"].as_str().unwrap(),
        cancel,
    )?;
    Ok(result)
}
pub fn prepare_staged(request: &PrepareRequest, cancel: &AtomicBool) -> Result<Value> {
    check(cancel)?;
    if request.max_disk_bytes == 0 {
        return Err(Error::invalid("max_disk_bytes must be positive"));
    }
    let root = io("resolve collection source", request.root.canonicalize())?;
    io(
        "create explicit collection store",
        fs::create_dir_all(&request.store),
    )?;
    let store = io(
        "resolve collection destination",
        request.store.canonicalize(),
    )?;
    if root == store {
        return Err(Error::invalid(
            "collection store must differ from source root",
        ));
    }
    let writer = Writer::open(&store, cancel)?;
    let config = if writer.path.join("collection.json").exists() {
        let config: Config = codec::decode(&read_at(&writer.root, "collection.json")?)?;
        if URL_SAFE_NO_PAD.decode(&config.root_base64).ok().as_deref()
            != Some(root.as_os_str().as_bytes())
            || config.max_disk_bytes != request.max_disk_bytes
        {
            return Err(Error::invalid(
                "existing collection source/budget differs; use update",
            ));
        }
        config
    } else {
        storage::init(&root, cancel)?;
        let root_fd = open_root(&root)?;
        let workspace: Workspace = codec::decode(&read_at(&root_fd, ".compass/workspace.json")?)?;
        let mut policy: Policy = codec::decode(&read_at(&root_fd, ".compass/policy.json")?)?;
        policy.max_file_bytes = Some(
            policy
                .max_file_bytes
                .as_deref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(4 * 1024 * 1024)
                .min(4 * 1024 * 1024)
                .to_string(),
        );
        policy.max_total_bytes_read = Some(
            policy
                .max_total_bytes_read
                .as_deref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(256 * 1024 * 1024)
                .min(256 * 1024 * 1024)
                .to_string(),
        );
        policy.max_entries = Some(
            policy
                .max_entries
                .as_deref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(10_000)
                .min(10_000)
                .to_string(),
        );
        if let Ok(relative) = store.strip_prefix(&root) {
            policy
                .exclude_paths
                .push(model::path_key(relative.as_os_str().as_bytes()));
            policy.exclude_paths.sort();
            policy.exclude_paths.dedup();
        }
        let config = Config {
            version: 1,
            root_base64: URL_SAFE_NO_PAD.encode(root.as_os_str().as_bytes()),
            max_disk_bytes: request.max_disk_bytes,
            workspace,
            policy,
        };
        let temporary = writer
            .path
            .join(format!(".config-{}", uuid::Uuid::new_v4()));
        write_new(
            &writer.path,
            config.max_disk_bytes,
            &temporary,
            &codec::canonical(&config)?,
        )?;
        rfs::renameat_with(
            &writer.root,
            temporary.strip_prefix(&writer.path).unwrap(),
            &writer.root,
            "collection.json",
            rfs::RenameFlags::NOREPLACE,
        )
        .map_err(|e| Error::io("publish collection configuration", e.into()))?;
        io("sync collection configuration", writer.root.sync_all())?;
        config
    };
    build(&writer, &config, cancel)
}
pub fn update(store: &Path, cancel: &AtomicBool) -> Result<Value> {
    let result = update_staged(store, cancel)?;
    publish_current(store, result["generation"].as_str().unwrap(), cancel)?;
    Ok(result)
}
pub fn update_staged(store: &Path, cancel: &AtomicBool) -> Result<Value> {
    let writer = Writer::open(store, cancel)?;
    let config: Config = codec::decode(&read_at(&writer.root, "collection.json")?)?;
    build(&writer, &config, cancel)
}
pub fn publish_current(store: &Path, generation: &str, cancel: &AtomicBool) -> Result<()> {
    publish_current_inner(store, generation, None, cancel)
}
pub fn publish_current_expected(
    store: &Path,
    generation: &str,
    expected_previous: Option<&str>,
    cancel: &AtomicBool,
) -> Result<()> {
    publish_current_inner(store, generation, Some(expected_previous), cancel)
}
fn publish_current_inner(
    store: &Path,
    generation: &str,
    expected: Option<Option<&str>>,
    cancel: &AtomicBool,
) -> Result<()> {
    let writer = Writer::open(store, cancel)?;
    let collection = Collection::open(store, Some(generation))?;
    let config = &collection.config;
    let receipt: Value = codec::decode(&read_at(&collection.directory, "generation.json")?)?;
    let current = match read_at(&writer.root, "current") {
        Ok(b) => Some(
            String::from_utf8(b)
                .map_err(|_| Error::corrupt("invalid current generation"))?
                .trim()
                .to_owned(),
        ),
        Err(e) if e.message.contains("No such file") => None,
        Err(e) => return Err(e),
    };
    if current.as_deref() == Some(generation) {
        return Ok(());
    }
    if expected.is_some_and(|wanted| wanted != current.as_deref()) {
        return Err(Error::invalid("current generation changed; retry update"));
    }
    if receipt["parent_generation"].as_str() != current.as_deref() {
        return Err(Error::invalid(
            "collection changed while preparing generation; retry update",
        ));
    }
    let pointer = writer
        .path
        .join(format!(".current-{}", uuid::Uuid::new_v4()));
    write_new(
        &writer.path,
        config.max_disk_bytes,
        &pointer,
        generation.as_bytes(),
    )?;
    check(cancel)?;
    io(
        "publish current generation",
        fs::rename(&pointer, writer.path.join("current")),
    )?;
    io("sync collection publication", writer.root.sync_all())?;
    Ok(())
}
pub fn status(store: &Path) -> Result<Value> {
    let c = Collection::open(store, None)?;
    Ok(
        json!({"generation":c.generation,"snapshot_id":c.generation,"quality":c.snapshot.quality,"root_base64":c.config.root_base64,"max_disk_bytes":c.config.max_disk_bytes.to_string(),"disk_bytes":disk_usage(&pinned(&c._root))?.to_string(),"retention":"explicit_no_automatic_deletion","snapshot":c.snapshot_path,"source":c.source,"observed_files":c.snapshot.stats.observed_files,"observed_at":c.snapshot.finished_at,"policy":c.snapshot.policy}),
    )
}
fn build(writer: &Writer, config: &Config, cancel: &AtomicBool) -> Result<Value> {
    check(cancel)?;
    config.workspace.validate()?;
    config.policy.validate()?;
    mkdir(&writer.root, "objects")?;
    mkdir(&writer.root, "generations")?;
    // Only private abandoned staging directories are reclaimed. Published
    // generations and objects are never removed by automatic maintenance.
    for entry in io("list interrupted staging", fs::read_dir(&writer.path))? {
        let e = io("list interrupted staging", entry)?;
        if e.file_name().as_bytes().starts_with(b".stage-") {
            let m = io("inspect interrupted staging", e.file_type())?;
            if m.is_dir() {
                io("discard interrupted staging", fs::remove_dir_all(e.path()))?;
            } else {
                return Err(Error::corrupt("invalid staging entry"));
            }
        }
    }
    reserve(&writer.path, config.max_disk_bytes, 0)?;
    let stage = io(
        "create generation staging",
        tempfile::Builder::new()
            .prefix(".stage-")
            .tempdir_in(&writer.path),
    )?;
    let root_bytes = URL_SAFE_NO_PAD
        .decode(&config.root_base64)
        .map_err(|_| Error::corrupt("invalid source root encoding"))?;
    let root = PathBuf::from(std::ffi::OsString::from_vec(root_bytes));
    let root_fd = open_root(&root)?;
    let temp = stage.path().join("capture.tmp");
    let mut capture = Capture {
        store: &writer.path,
        max: config.max_disk_bytes,
        temp: &temp,
        current: None,
        objects: at(&writer.root, "objects", OFlags::RDONLY | OFlags::DIRECTORY)?,
        reused: 0,
        added: 0,
    };
    let snapshot = observe::observe_captured(
        &root_fd,
        &config.workspace,
        &config.policy,
        cancel,
        &mut capture,
    )?;
    capture.current.take();
    if temp.exists() {
        io("discard unaccepted capture", fs::remove_file(&temp))?;
    }
    check(cancel)?;
    let manifest = codec::canonical(&snapshot)?;
    write_new(
        &writer.path,
        config.max_disk_bytes,
        &stage.path().join("snapshot.json"),
        &manifest,
    )?;
    io(
        "create source package",
        fs::create_dir(stage.path().join("source")),
    )?;
    io(
        "create source object directory",
        fs::create_dir(stage.path().join("source/objects")),
    )?;
    for e in snapshot.entries.iter().filter(|e| e.state == "observed") {
        check(cancel)?;
        let h = e.content_sha256.as_ref().unwrap();
        let target = stage.path().join("source/objects").join(h);
        if !target.exists() {
            io(
                "link captured revision",
                fs::hard_link(writer.path.join("objects").join(h), target),
            )?;
        }
    }
    let source = json!({"source_schema_version":"1","source_contract_version":"0.1.0","workspace_id":snapshot.workspace_id,"snapshot_id":snapshot.snapshot_id,"policy_id":snapshot.policy_id,"snapshot_file_sha256":model::digest(&manifest)});
    write_new(
        &writer.path,
        config.max_disk_bytes,
        &stage.path().join("source/source.json"),
        &codec::canonical(&source)?,
    )?;
    let previous = match read_at(&writer.root, "current") {
        Ok(b) => Some(
            String::from_utf8(b)
                .map_err(|_| Error::corrupt("invalid current generation"))?
                .trim()
                .to_owned(),
        ),
        Err(e) if e.message.contains("No such file") => None,
        Err(e) => return Err(e),
    };
    let dbpath = stage.path().join("index.sqlite");
    let old = previous
        .as_deref()
        .map(|g| Collection::open(&pinned(&writer.root), Some(g)))
        .transpose()?;
    if let Some(old) = &old {
        let bytes = read_at(&old.directory, "index.sqlite")?;
        write_new(&writer.path, config.max_disk_bytes, &dbpath, &bytes)?;
    }
    let db = sql(Connection::open(&dbpath))?;
    sql(db.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=FULL; PRAGMA temp_store=MEMORY; PRAGMA page_size=4096;"))?;
    // SQLite writes only to private staging, with a page cap covering the entire
    // remaining logical-file budget. No journal or on-disk temp duplicates exist.
    let usage = disk_usage(&writer.path)?;
    let db_size = io("inspect staged index", fs::metadata(&dbpath))?.len();
    let cap = config
        .max_disk_bytes
        .saturating_sub(usage)
        .saturating_add(db_size)
        / 4096;
    if cap == 0 {
        return Err(Error::invalid("collection max_disk_bytes cannot fit index"));
    }
    sql(db.pragma_update(None, "max_page_count", cap))?;
    sql(db.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS search USING fts5(path,content,identifiers,path_key UNINDEXED,hash UNINDEXED,start_line UNINDEXED,end_line UNINDEXED,start_byte UNINDEXED,end_byte UNINDEXED,tokenize='unicode61');"))?;
    let new_entries: BTreeMap<_, _> = snapshot
        .entries
        .iter()
        .filter(|e| e.state == "observed")
        .map(|e| (e.path_key.as_str(), e))
        .collect();
    let mut kept = HashSet::new();
    if let Some(old) = &old {
        for e in old
            .snapshot
            .entries
            .iter()
            .filter(|e| e.state == "observed")
        {
            if new_entries
                .get(e.path_key.as_str())
                .is_some_and(|n| n.content_sha256 == e.content_sha256)
            {
                kept.insert(e.path_key.as_str());
            } else {
                sql(db.execute("DELETE FROM search WHERE path_key=?1", [&e.path_key]))?;
            }
        }
    }
    let mut indexed = 0;
    for e in new_entries.values() {
        check(cancel)?;
        if kept.contains(e.path_key.as_str()) {
            continue;
        }
        let h = e.content_sha256.as_ref().unwrap();
        let raw = read_at(&writer.root, &format!("objects/{h}"))?;
        if model::digest(&raw) != *h {
            return Err(Error::corrupt("CAS object changed before indexing"));
        }
        for (start, end, a, b) in chunk_spans(&raw) {
            let text = std::str::from_utf8(&raw[a as usize..b as usize]).unwrap();
            sql(db.execute("INSERT INTO search(path,content,identifiers,path_key,hash,start_line,end_line,start_byte,end_byte) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![display_path(&e.path_key),text,segment_identifiers(text),e.path_key,h,start,end,a,b]))?;
        }
        indexed += 1;
    }
    sql(db.execute_batch("PRAGMA optimize;"))?;
    drop(db);
    reserve(&writer.path, config.max_disk_bytes, 0)?;
    let stage_fd = open_root(stage.path())?;
    let index_hash = hash_at(&stage_fd, "index.sqlite")?;
    let receipt = json!({"generation":snapshot.snapshot_id,"parent_generation":previous,"index_sha256":index_hash,"index_version":"sqlite_fts5_bm25_v1","reused_objects":capture.reused,"new_objects":capture.added,"indexed_files":indexed,"reused_indexed_files":kept.len()});
    write_new(
        &writer.path,
        config.max_disk_bytes,
        &stage.path().join("generation.json"),
        &codec::canonical(&receipt)?,
    )?;
    for directory in [
        stage.path().join("source/objects"),
        stage.path().join("source"),
        stage.path().to_owned(),
        writer.path.join("objects"),
    ] {
        io("sync generation directory", File::open(directory))?
            .sync_all()
            .map_err(|e| Error::io("sync generation directory", e))?;
    }
    check(cancel)?;
    let generation = snapshot.snapshot_id.clone();
    rfs::renameat_with(
        &writer.root,
        stage.path().strip_prefix(&writer.path).unwrap(),
        at(
            &writer.root,
            "generations",
            OFlags::RDONLY | OFlags::DIRECTORY,
        )?,
        generation.as_str(),
        rfs::RenameFlags::NOREPLACE,
    )
    .map_err(|e| Error::io("publish immutable generation", e.into()))?;
    io(
        "sync generations",
        File::open(writer.path.join("generations")),
    )?
    .sync_all()
    .map_err(|e| Error::io("sync generations", e))?;
    let _ = stage.close();
    let store = io("resolve collection path", fs::canonicalize(&writer.path))?;
    Ok(
        json!({"generation":generation,"snapshot_id":generation,"quality":snapshot.quality,"snapshot":store.join("generations").join(&generation).join("snapshot.json"),"source":store.join("generations").join(&generation).join("source"),"index":receipt,"max_disk_bytes":config.max_disk_bytes.to_string(),"disk_bytes":disk_usage(&writer.path)?.to_string(),"retention":"explicit_no_automatic_deletion","generation_path":store.join("generations").join(&generation)}),
    )
}
