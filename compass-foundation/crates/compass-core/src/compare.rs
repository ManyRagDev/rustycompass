//! Offline, deterministic comparison. Unknown coverage takes precedence over absence.
use crate::{
    Error, Result,
    model::{CONTRACT_VERSION, Entry, SCHEMA_VERSION, Snapshot, decode_path_key, parent},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub path_key: String,
    pub classification: String,
    pub before: Option<Entry>,
    pub after: Option<Entry>,
    pub content_changed: bool,
    pub executable_changed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnknownRegion {
    pub side: String,
    pub path_key: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub unchanged: String,
    pub added: String,
    pub removed: String,
    pub modified: String,
    pub metadata_changed: String,
    pub left_scope: String,
    pub entered_scope: String,
    pub unknown: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    pub schema_version: String,
    pub contract_version: String,
    pub baseline_snapshot_id: String,
    pub current_snapshot_id: String,
    pub workspace_id: String,
    pub path_namespace: String,
    pub policy_id: String,
    pub quality: String,
    pub relation: String,
    pub changes: Vec<Change>,
    pub unknown_regions: Vec<UnknownRegion>,
    pub counts: Counts,
}
struct View<'a> {
    entries: BTreeMap<Vec<u8>, &'a Entry>,
    coverage: BTreeMap<Vec<u8>, &'a crate::model::Coverage>,
}
enum State<'a> {
    Observed(&'a Entry),
    Excluded(&'a Entry),
    Absent,
    Unknown,
}
impl<'a> View<'a> {
    fn new(snapshot: &'a Snapshot) -> Result<Self> {
        Ok(Self {
            entries: snapshot
                .entries
                .iter()
                .map(|e| Ok((decode_path_key(&e.path_key)?, e)))
                .collect::<Result<_>>()?,
            coverage: snapshot
                .directory_coverage
                .iter()
                .map(|c| Ok((decode_path_key(&c.path_key)?, c)))
                .collect::<Result<_>>()?,
        })
    }
    fn state(&self, path: &[u8]) -> State<'a> {
        // A successful individual read does not repair an unstable ancestor listing.
        let mut ancestor = path;
        loop {
            if self
                .coverage
                .get(ancestor)
                .is_some_and(|c| c.state != "complete")
            {
                return State::Unknown;
            }
            if ancestor.is_empty() {
                break;
            }
            ancestor = parent(ancestor);
        }
        // The first missing component under complete coverage proves absence,
        // including descendants whose immediate parent no longer exists.
        let boundaries = path
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (*b == b'/').then_some(i))
            .chain(std::iter::once(path.len()));
        for end in boundaries {
            let prefix = &path[..end];
            if let Some(entry) = self.entries.get(prefix) {
                return match entry.state.as_str() {
                    "observed" if prefix == path => State::Observed(entry),
                    "observed" => State::Absent,
                    "excluded" => State::Excluded(entry),
                    _ => State::Unknown,
                };
            }
            if !self.coverage.contains_key(prefix) {
                return State::Absent;
            }
        }
        State::Absent // A known directory is outside the set of regular files.
    }
}
pub fn compare(baseline: &Snapshot, current: &Snapshot) -> Result<Comparison> {
    baseline.validate()?;
    current.validate()?;
    if baseline.workspace_id != current.workspace_id
        || baseline.path_namespace != current.path_namespace
        || baseline.policy_id != current.policy_id
    {
        return Err(Error::invalid(
            "snapshots have incompatible workspace, path namespace or policy",
        ));
    }
    let before = View::new(baseline)?;
    let after = View::new(current)?;
    let paths: BTreeSet<_> = before
        .entries
        .keys()
        .chain(after.entries.keys())
        .cloned()
        .collect();
    let mut changes = vec![];
    let mut counters = [0_u64; 8];
    for path in paths {
        let a = before.state(&path);
        let b = after.state(&path);
        let (index, classification, content_changed, executable_changed, reason) = match (a, b) {
            (State::Unknown, _) | (_, State::Unknown) => (7, "unknown", false, false, None),
            (State::Observed(a), State::Observed(b)) => {
                let content = a.content_sha256 != b.content_sha256 || a.size_bytes != b.size_bytes;
                let executable = a.executable != b.executable;
                if content {
                    (3, "modified", true, executable, None)
                } else if executable {
                    (4, "metadata_changed", false, true, None)
                } else {
                    counters[0] += 1;
                    continue;
                }
            }
            (State::Absent, State::Observed(_)) => (1, "added", true, false, None),
            (State::Observed(_), State::Absent) => (2, "removed", true, false, None),
            (State::Observed(_), State::Excluded(e)) => {
                (5, "left_scope", true, false, e.reason.clone())
            }
            (State::Excluded(e), State::Observed(_)) => {
                (6, "entered_scope", true, false, e.reason.clone())
            }
            _ => continue,
        };
        counters[index] += 1;
        changes.push(Change {
            path_key: crate::model::path_key(&path),
            classification: classification.into(),
            before: before.entries.get(&path).map(|e| (*e).clone()),
            after: after.entries.get(&path).map(|e| (*e).clone()),
            content_changed,
            executable_changed,
            reason,
        });
    }
    let mut regions = vec![];
    for (side, snapshot) in [("baseline", baseline), ("current", current)] {
        for c in snapshot
            .directory_coverage
            .iter()
            .filter(|c| c.state != "complete")
        {
            regions.push((
                decode_path_key(&c.path_key)?,
                side,
                UnknownRegion {
                    side: side.into(),
                    path_key: c.path_key.clone(),
                    state: c.state.clone(),
                    reason: c.reason.clone(),
                },
            ));
        }
        for e in snapshot
            .entries
            .iter()
            .filter(|e| !["observed", "excluded"].contains(&e.state.as_str()))
        {
            regions.push((
                decode_path_key(&e.path_key)?,
                side,
                UnknownRegion {
                    side: side.into(),
                    path_key: e.path_key.clone(),
                    state: e.state.clone(),
                    reason: e.reason.clone(),
                },
            ));
        }
    }
    regions.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    let partial = baseline.quality == "partial" || current.quality == "partial";
    Ok(Comparison {
        schema_version: SCHEMA_VERSION.into(),
        contract_version: CONTRACT_VERSION.into(),
        baseline_snapshot_id: baseline.snapshot_id.clone(),
        current_snapshot_id: current.snapshot_id.clone(),
        workspace_id: baseline.workspace_id.clone(),
        path_namespace: baseline.path_namespace.clone(),
        policy_id: baseline.policy_id.clone(),
        quality: if partial { "partial" } else { "complete" }.into(),
        relation: if partial {
            "undetermined"
        } else if changes.is_empty() {
            "equal"
        } else {
            "different"
        }
        .into(),
        changes,
        unknown_regions: regions.into_iter().map(|(_, _, region)| region).collect(),
        counts: Counts {
            unchanged: counters[0].to_string(),
            added: counters[1].to_string(),
            removed: counters[2].to_string(),
            modified: counters[3].to_string(),
            metadata_changed: counters[4].to_string(),
            left_scope: counters[5].to_string(),
            entered_scope: counters[6].to_string(),
            unknown: counters[7].to_string(),
        },
    })
}
