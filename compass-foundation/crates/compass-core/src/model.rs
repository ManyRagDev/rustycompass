//! Versioned Foundation transport and semantic validation.
use crate::{Error, Result, codec};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub const SCHEMA_VERSION: &str = "1";
pub const CONTRACT_VERSION: &str = "0.1.0";
pub const PATH_NAMESPACE: &str = "linux_bytes_v1";

// Distinguish a missing optional field from an explicitly null revision field.
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
fn nullable<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub schema_version: String,
    pub contract_version: String,
    pub workspace_id: String,
    pub path_namespace: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema_version: String,
    pub contract_version: String,
    pub exclude_directory_names: Vec<String>,
    pub exclude_file_names: Vec<String>,
    pub exclude_file_prefixes: Vec<String>,
    pub exclude_paths: Vec<String>,
    #[serde(deserialize_with = "nullable")]
    pub max_file_bytes: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub max_total_bytes_read: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub max_entries: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub max_duration_ms: Option<String>,
    pub read_attempts: String,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION.into(),
            contract_version: CONTRACT_VERSION.into(),
            exclude_directory_names: [".aws", ".compass", ".git", ".ssh"]
                .map(String::from)
                .to_vec(),
            exclude_file_names: vec![".env".into()],
            exclude_file_prefixes: vec![".env.".into()],
            exclude_paths: vec![],
            max_file_bytes: None,
            max_total_bytes_read: None,
            max_entries: None,
            max_duration_ms: None,
            read_attempts: "2".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub path_key: String,
    pub kind: String,
    pub state: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub reason: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub content_sha256: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub size_bytes: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub executable: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub path_key: String,
    pub state: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Issue {
    pub path_key: String,
    pub ordinal: String,
    pub code: String,
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub observed_files: String,
    pub excluded_entries: String,
    pub unknown_entries: String,
    pub bytes_read: String,
    pub read_attempts: String,
    pub duration_ms: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version: String,
    pub contract_version: String,
    pub producer_version: String,
    pub producer_build: String,
    pub workspace_id: String,
    pub snapshot_id: String,
    pub path_namespace: String,
    pub policy: Policy,
    pub policy_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub consistency: String,
    pub quality: String,
    pub entries: Vec<Entry>,
    pub directory_coverage: Vec<Coverage>,
    pub issues: Vec<Issue>,
    #[serde(deserialize_with = "nullable")]
    pub state_digest: Option<String>,
    pub stats: Stats,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn path_key(path: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(path)
}
pub fn decode_path_key(key: &str) -> Result<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD
        .decode(key)
        .map_err(|_| Error::corrupt("invalid base64url path key"))?;
    if path_key(&bytes) != key
        || bytes.contains(&0)
        || (!bytes.is_empty()
            && bytes
                .split(|b| *b == b'/')
                .any(|c| c.is_empty() || c == b"." || c == b".."))
    {
        return Err(Error::corrupt(
            "path must be canonical, relative native bytes without NUL, dot or empty components",
        ));
    }
    Ok(bytes)
}
pub fn decimal(value: &str) -> Result<u64> {
    if value.is_empty()
        || !value.bytes().all(|b| b.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(Error::corrupt(
            "integer must be a canonical unsigned decimal string",
        ));
    }
    value
        .parse()
        .map_err(|_| Error::corrupt("integer exceeds u64"))
}
fn versions(schema: &str, contract: &str) -> Result<()> {
    if schema != SCHEMA_VERSION || contract != CONTRACT_VERSION {
        return Err(Error::corrupt("unsupported schema or contract version"));
    }
    Ok(())
}
fn uuid(value: &str) -> Result<()> {
    let id = uuid::Uuid::parse_str(value).map_err(|_| Error::corrupt("invalid UUID"))?;
    if id.get_version_num() != 4
        || id.get_variant() != uuid::Variant::RFC4122
        || id.to_string() != value
    {
        return Err(Error::corrupt(
            "UUID must use canonical lowercase v4 representation",
        ));
    }
    Ok(())
}
fn hash(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Error::corrupt("invalid lowercase SHA-256"));
    }
    Ok(())
}
fn timestamp(value: &str) -> Result<OffsetDateTime> {
    let t = OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| Error::corrupt("invalid RFC3339 timestamp"))?;
    if !t.offset().is_utc() || !(value.ends_with('Z') || value.ends_with("+00:00")) {
        return Err(Error::corrupt("timestamp must have an explicit UTC offset"));
    }
    Ok(t)
}
fn nonempty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::corrupt(format!("{field} must not be empty")));
    }
    Ok(())
}
fn ordered<T: Ord>(items: &[T], what: &str) -> Result<()> {
    if items.windows(2).any(|p| p[0] >= p[1]) {
        return Err(Error::corrupt(format!(
            "{what} must be strictly sorted and unique"
        )));
    }
    Ok(())
}
pub(crate) fn parent(path: &[u8]) -> &[u8] {
    path.iter()
        .rposition(|b| *b == b'/')
        .map_or(&[], |pos| &path[..pos])
}
impl Workspace {
    pub fn validate(&self) -> Result<()> {
        versions(&self.schema_version, &self.contract_version)?;
        uuid(&self.workspace_id)?;
        if self.path_namespace != PATH_NAMESPACE {
            return Err(Error::corrupt("unsupported path namespace"));
        }
        timestamp(&self.created_at)?;
        Ok(())
    }
}
impl Policy {
    pub fn validate(&self) -> Result<()> {
        versions(&self.schema_version, &self.contract_version)?;
        for list in [
            &self.exclude_directory_names,
            &self.exclude_file_names,
            &self.exclude_file_prefixes,
        ] {
            ordered(list, "policy exclusion list")?;
            for name in list {
                if name.is_empty()
                    || name.contains('/')
                    || name.contains('\0')
                    || name == "."
                    || name == ".."
                {
                    return Err(Error::invalid("invalid exclusion basename or prefix"));
                }
            }
        }
        for required in [".aws", ".compass", ".git", ".ssh"] {
            if !self.exclude_directory_names.iter().any(|n| n == required) {
                return Err(Error::invalid(
                    "required internal directory exclusion missing",
                ));
            }
        }
        if !self.exclude_file_names.iter().any(|n| n == ".env")
            || !self.exclude_file_prefixes.iter().any(|n| n == ".env.")
        {
            return Err(Error::invalid(
                "required environment file exclusions missing",
            ));
        }
        let paths: Vec<_> = self
            .exclude_paths
            .iter()
            .map(|p| decode_path_key(p))
            .collect::<Result<_>>()?;
        ordered(&paths, "excluded paths")?;
        if paths.iter().any(Vec::is_empty) {
            return Err(Error::invalid("cannot exclude the workspace root"));
        }
        for limit in [
            &self.max_file_bytes,
            &self.max_total_bytes_read,
            &self.max_entries,
            &self.max_duration_ms,
        ]
        .into_iter()
        .flatten()
        {
            if decimal(limit)? == 0 {
                return Err(Error::invalid("limits must be positive"));
            }
        }
        if self.read_attempts != "2" {
            return Err(Error::invalid(
                "contract 0.1.0 requires two maximum read attempts",
            ));
        }
        Ok(())
    }
    pub fn id(&self) -> Result<String> {
        self.validate()?;
        let mut bytes = b"compass-policy-v1\0".to_vec();
        bytes.extend(codec::canonical(self)?);
        Ok(digest(&bytes))
    }
    pub fn excludes(&self, path: &[u8], directory: bool) -> Result<bool> {
        let name = path.rsplit(|b| *b == b'/').next().unwrap_or(path);
        if directory
            && self
                .exclude_directory_names
                .iter()
                .any(|n| n.as_bytes() == name)
        {
            return Ok(true);
        }
        if !directory
            && (self.exclude_file_names.iter().any(|n| n.as_bytes() == name)
                || self
                    .exclude_file_prefixes
                    .iter()
                    .any(|n| name.starts_with(n.as_bytes())))
        {
            return Ok(true);
        }
        for key in &self.exclude_paths {
            let excluded = decode_path_key(key)?;
            if path == excluded
                || (path.starts_with(&excluded) && path.get(excluded.len()) == Some(&b'/'))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
impl Snapshot {
    pub fn compute_state_digest(&self) -> Result<String> {
        #[derive(Serialize)]
        struct File<'a> {
            path_key: &'a str,
            content_sha256: &'a str,
            size_bytes: &'a str,
            executable: bool,
        }
        #[derive(Serialize)]
        struct State<'a> {
            workspace_id: &'a str,
            path_namespace: &'a str,
            policy_id: &'a str,
            files: Vec<File<'a>>,
        }
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.state == "observed")
            .map(|e| Ok((decode_path_key(&e.path_key)?, e)))
            .collect::<Result<_>>()?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let files = entries
            .into_iter()
            .map(|(_, e)| {
                Ok(File {
                    path_key: &e.path_key,
                    content_sha256: e
                        .content_sha256
                        .as_deref()
                        .ok_or_else(|| Error::corrupt("observed entry lacks hash"))?,
                    size_bytes: e
                        .size_bytes
                        .as_deref()
                        .ok_or_else(|| Error::corrupt("observed entry lacks size"))?,
                    executable: e
                        .executable
                        .ok_or_else(|| Error::corrupt("observed entry lacks executable flag"))?,
                })
            })
            .collect::<Result<_>>()?;
        let state = State {
            workspace_id: &self.workspace_id,
            path_namespace: &self.path_namespace,
            policy_id: &self.policy_id,
            files,
        };
        let mut bytes = b"compass-state-v1\0".to_vec();
        bytes.extend(codec::canonical(&state)?);
        Ok(digest(&bytes))
    }
    pub fn validate(&self) -> Result<()> {
        versions(&self.schema_version, &self.contract_version)?;
        uuid(&self.workspace_id)?;
        uuid(&self.snapshot_id)?;
        if self.path_namespace != PATH_NAMESPACE {
            return Err(Error::corrupt("unsupported path namespace"));
        }
        nonempty(&self.producer_version, "producer_version")?;
        nonempty(&self.producer_build, "producer_build")?;
        if self.consistency != "observation_window" {
            return Err(Error::corrupt("unsupported consistency"));
        }
        if timestamp(&self.finished_at)? < timestamp(&self.started_at)? {
            return Err(Error::corrupt("observation window ends before it starts"));
        }
        self.policy.validate()?;
        hash(&self.policy_id)?;
        if self.policy.id()? != self.policy_id {
            return Err(Error::corrupt("policy_id mismatch"));
        }
        let mut coverage = BTreeMap::new();
        let mut coverage_keys = vec![];
        let mut partial = false;
        for c in &self.directory_coverage {
            let key = decode_path_key(&c.path_key)?;
            if !["complete", "unreadable", "unstable", "not_visited"].contains(&c.state.as_str()) {
                return Err(Error::corrupt("invalid directory coverage state"));
            }
            if c.state == "complete" {
                if c.reason.is_some() {
                    return Err(Error::corrupt(
                        "complete coverage must not have a failure reason",
                    ));
                }
            } else {
                partial = true;
                nonempty(c.reason.as_deref().unwrap_or(""), "coverage reason")?;
            }
            if !key.is_empty() && self.policy.excludes(&key, true)? {
                return Err(Error::corrupt(
                    "excluded directory cannot have visited coverage",
                ));
            }
            coverage_keys.push(key.clone());
            coverage.insert(key, c);
        }
        ordered(&coverage_keys, "directory coverage")?;
        if !coverage.contains_key(&Vec::new()) {
            return Err(Error::corrupt("root directory coverage is required"));
        }
        let mut entry_keys = vec![];
        let mut seen = BTreeSet::new();
        let mut counts = [0_u64; 3];
        let mut accepted_bytes = 0_u128;
        for e in &self.entries {
            let key = decode_path_key(&e.path_key)?;
            if key.is_empty() {
                return Err(Error::corrupt("entry cannot use root path"));
            }
            if coverage.contains_key(&key) {
                return Err(Error::corrupt("entry overlaps directory coverage"));
            }
            if !coverage.contains_key(parent(&key)) {
                return Err(Error::corrupt("entry lacks parent coverage"));
            }
            if !["file", "directory", "symlink", "other", "unknown"].contains(&e.kind.as_str()) {
                return Err(Error::corrupt("invalid entry kind"));
            }
            let excluded = self.policy.excludes(&key, e.kind == "directory")?;
            match e.state.as_str() {
                "observed" => {
                    if e.kind != "file" || e.reason.is_some() || excluded {
                        return Err(Error::corrupt(
                            "observed entry must be an eligible regular file without a reason",
                        ));
                    }
                    hash(
                        e.content_sha256
                            .as_deref()
                            .ok_or_else(|| Error::corrupt("observed entry requires hash"))?,
                    )?;
                    accepted_bytes += u128::from(decimal(
                        e.size_bytes
                            .as_deref()
                            .ok_or_else(|| Error::corrupt("observed entry requires size"))?,
                    )?);
                    if e.executable.is_none() {
                        return Err(Error::corrupt("observed entry requires executable flag"));
                    }
                    counts[0] += 1;
                }
                "excluded" | "unreadable" | "unstable" | "limit_exceeded" => {
                    if e.content_sha256.is_some()
                        || e.size_bytes.is_some()
                        || e.executable.is_some()
                    {
                        return Err(Error::corrupt(
                            "unaccepted entry cannot contain revision fields",
                        ));
                    }
                    let reason = e.reason.as_deref().unwrap_or("");
                    nonempty(reason, "entry reason")?;
                    if e.state == "excluded" {
                        let valid = match e.kind.as_str() {
                            "symlink" => reason == "symlink_not_followed",
                            "other" => reason == "unsupported_kind",
                            "file" | "directory" => reason == "policy_excluded" && excluded,
                            _ => false,
                        };
                        if !valid {
                            return Err(Error::corrupt(
                                "exclusion kind, policy and reason disagree",
                            ));
                        }
                        counts[1] += 1;
                    } else {
                        counts[2] += 1;
                        partial = true;
                    }
                }
                _ => return Err(Error::corrupt("invalid entry state")),
            }
            seen.insert(key.clone());
            entry_keys.push(key);
        }
        ordered(&entry_keys, "entries")?;
        for key in coverage.keys().filter(|p| !p.is_empty()) {
            if !coverage.contains_key(parent(key)) {
                return Err(Error::corrupt("directory lacks parent coverage"));
            }
        }
        // Coverage and entry parent checks prohibit descendants beneath files or excluded boundaries.
        let mut issue_keys = vec![];
        for issue in &self.issues {
            let key = decode_path_key(&issue.path_key)?;
            if !coverage.contains_key(&key) && !seen.contains(&key) {
                return Err(Error::corrupt(
                    "issue path is not represented in the observation",
                ));
            }
            nonempty(&issue.code, "issue code")?;
            nonempty(&issue.message, "issue message")?;
            issue_keys.push((key, decimal(&issue.ordinal)?));
        }
        ordered(&issue_keys, "issues")?;
        for (expected, actual) in counts.into_iter().zip([
            &self.stats.observed_files,
            &self.stats.excluded_entries,
            &self.stats.unknown_entries,
        ]) {
            if decimal(actual)? != expected {
                return Err(Error::corrupt("entry counters disagree with manifest"));
            }
        }
        if u128::from(decimal(&self.stats.bytes_read)?) < accepted_bytes {
            return Err(Error::corrupt(
                "bytes_read is smaller than accepted content",
            ));
        }
        if decimal(&self.stats.read_attempts)? < counts[0] {
            return Err(Error::corrupt(
                "read_attempts is smaller than accepted file count",
            ));
        }
        decimal(&self.stats.duration_ms)?;
        if self.quality != if partial { "partial" } else { "complete" } {
            return Err(Error::corrupt(
                "quality disagrees with coverage or entry states",
            ));
        }
        if partial {
            if self.state_digest.is_some() {
                return Err(Error::corrupt(
                    "partial snapshot cannot have a state digest",
                ));
            }
        } else {
            let state_digest = self
                .state_digest
                .as_deref()
                .ok_or_else(|| Error::corrupt("complete snapshot requires state digest"))?;
            hash(state_digest)?;
            if state_digest != self.compute_state_digest()? {
                return Err(Error::corrupt("state_digest mismatch"));
            }
        }
        Ok(())
    }
}
