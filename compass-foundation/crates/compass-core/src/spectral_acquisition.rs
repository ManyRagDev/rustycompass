//! Experimental open acquisition. A terminal window is not a semantic sufficiency claim.
use crate::{
    Error, Result, codec,
    collection::{Collection, VerifiedDocument},
    diffusion, model, retrieval,
    structure::{Graph, Node, Relation, Resolution},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

pub const POLICY: &str = "spectral_window_v1";
const PROFILE: &str = "spectral-window-v1-dev-1";
const PROJECTION: &str = "resolved_contains_imports_calls_unit_v1";
const MAX_SEEDS: usize = 8;
const MAX_REGION: usize = 512;
const MAX_DEPTH: usize = 8;
const MAX_CACHE: usize = 32 * 1024 * 1024;
const MAX_DOCS: usize = 128;
const MAX_MS: u64 = 20_000;
const MAX_NODE_VISITS: usize = 2_000_000;
const MAX_EDGE_VISITS: usize = 2_000_000;
const MAX_READ_BYTES: usize = 64 * 1024 * 1024;
const MASS: f64 = 0.8;
const LOW: f64 = 0.15;
const PERSISTENCE: usize = 2;

#[derive(Clone, Debug)]
pub struct Request {
    pub query: String,
    pub mode: String,
    pub scope: String,
    pub budget_bytes: usize,
}
/// Compact query intent. Targets are user-provided spellings, never graph IDs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryIntent {
    pub intent: String,
    pub search_query: String,
    pub targets: Vec<IntentTarget>,
    pub match_preference: String,
    pub scope: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentTarget {
    pub name: Option<String>,
    pub path: Option<String>,
}
impl QueryIntent {
    pub fn validate(&self, request: &Request) -> Result<()> {
        if self.intent.trim().is_empty()
            || self.intent.len() > 2400
            || self.search_query != request.query
            || self.scope != request.scope
            || self.targets.len() > MAX_SEEDS
            || !matches!(self.match_preference.as_str(), "exact" | "descriptive")
            || (self.match_preference == "exact") == self.targets.is_empty()
            || self.targets.iter().any(|t| {
                (t.name.is_none() && t.path.is_none())
                    || t.name
                        .as_ref()
                        .is_some_and(|n| n.is_empty() || n.len() > 512)
                    || t.path.as_ref().is_some_and(|p| {
                        p.is_empty()
                            || p.len() > 1024
                            || p.starts_with('/')
                            || p.split('/').any(|c| c.is_empty() || c == "." || c == "..")
                    })
            })
        {
            return Err(Error::invalid(
                "invalid query_intent or query/scope mismatch",
            ));
        }
        Ok(())
    }
}

/// Validate contradictory wire options before opening a collection or parsing cursors.
pub fn validate_options(
    policy: Option<&str>,
    scope: Option<&str>,
    contract: bool,
    cursor: bool,
    theta: Option<f64>,
    expansion: &str,
) -> Result<()> {
    match policy {
        None if scope.is_some() => Err(Error::invalid("scope requires acquisition_policy")),
        None => Ok(()),
        Some(POLICY)
            if matches!(scope, Some("local" | "dependencies" | "callers"))
                && !contract
                && !cursor
                && theta.is_none()
                && expansion == "none" =>
        {
            Ok(())
        }
        Some(POLICY) => Err(Error::invalid(
            "spectral_window_v1 requires scope local/dependencies/callers and rejects contract, cursor, explicit theta and expansion",
        )),
        Some(_) => Err(Error::invalid("unsupported acquisition_policy")),
    }
}
#[derive(Default, Serialize)]
struct Work {
    source_revalidation_bytes: usize,
    source_revalidation_nodes: usize,
    source_revalidation_edges: usize,
    source_revalidation_elapsed_ms: u64,
    graph_nodes: usize,
    graph_edges: usize,
    projection_node_visits: usize,
    projection_edge_visits: usize,
    seed_node_visits: usize,
    bfs_edge_visits: usize,
    ranking_candidates: usize,
    ranking_verified_reads: usize,
    ranking_literal_verified_files: Option<usize>,
    total_verified_document_reads: usize,
    verified_document_reads: usize,
    verified_document_bytes: usize,
    cache_bytes: usize,
    candidate_nodes: usize,
    closure_node_visits: usize,
    closure_edge_visits: usize,
    materialized_units: usize,
    scales: usize,
    trajectory_bytes: usize,
    numeric: diffusion::SpectralWork,
    elapsed_ms: u64,
}
struct Engine<'a> {
    collection: &'a Collection,
    graph: Graph,
    by_id: BTreeMap<String, usize>,
    outgoing: BTreeMap<String, Vec<usize>>,
    incoming: BTreeMap<String, Vec<usize>>,
    docs: BTreeMap<String, VerifiedDocument>,
    denied_docs: BTreeSet<String>,
    bindings: BTreeMap<usize, BTreeSet<String>>,
    work: Work,
    start: Instant,
    cancel: &'a AtomicBool,
}
#[derive(Clone)]
struct Candidate {
    ids: BTreeSet<String>,
    edges: BTreeSet<usize>,
    imports: BTreeSet<usize>,
    gaps: BTreeSet<String>,
    mass_omitted: f64,
    cut: bool,
}
fn validate_graph(c: &Collection, g: &Graph, cancel: &AtomicBool) -> Result<()> {
    let nodes: BTreeMap<_, _> = g.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let files: BTreeMap<_, _> = c
        .snapshot
        .entries
        .iter()
        .filter(|e| e.state == "observed")
        .map(|e| (e.path_key.as_str(), e))
        .collect();
    if nodes.len() != g.nodes.len() {
        return Err(Error::corrupt("spectral duplicate graph node identity"));
    }
    for n in &g.nodes {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::cancelled());
        }
        let file = files
            .get(n.path_key.as_str())
            .ok_or_else(|| Error::corrupt("spectral graph path absent from generation"))?;
        let size = file
            .size_bytes
            .as_deref()
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(|| Error::corrupt("spectral graph file size missing"))?;
        if file.content_sha256.as_deref() != Some(n.content_sha256.as_str())
            || n.start_byte > n.end_byte
            || n.end_byte > size
            || n.start_line == 0
            || n.end_line < n.start_line
        {
            return Err(Error::corrupt(
                "spectral invalid graph node provenance or interval",
            ));
        }
        if let Some(parent) = &n.parent_id {
            let p = nodes
                .get(parent.as_str())
                .ok_or_else(|| Error::corrupt("spectral graph parent absent"))?;
            if p.path_key != n.path_key || p.start_byte > n.start_byte || p.end_byte < n.end_byte {
                return Err(Error::corrupt("spectral graph parent interval mismatch"));
            }
        }
    }
    let mut done = BTreeSet::new();
    for n in &g.nodes {
        let mut chain = BTreeSet::new();
        let mut next = Some(n.id.as_str());
        while let Some(id) = next {
            if done.contains(id) {
                break;
            }
            if !chain.insert(id) {
                return Err(Error::corrupt("spectral graph parent cycle"));
            }
            next = nodes[id].parent_id.as_deref();
        }
        done.extend(chain);
    }
    let mut edges = BTreeSet::new();
    for e in &g.edges {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::cancelled());
        }
        let source = nodes
            .get(e.source.as_str())
            .ok_or_else(|| Error::corrupt("spectral graph relation source absent"))?;
        if !edges.insert(&e.id)
            || e.target
                .as_ref()
                .is_some_and(|id| !nodes.contains_key(id.as_str()))
        {
            return Err(Error::corrupt(
                "spectral graph relation identity or target invalid",
            ));
        }
        if e.evidence.path_key != source.path_key
            || e.evidence.content_sha256 != source.content_sha256
            || e.evidence.start_byte < source.start_byte
            || e.evidence.start_byte > e.evidence.end_byte
            || e.evidence.end_byte > source.end_byte
            || e.evidence.start_line < source.start_line
            || e.evidence.end_line < e.evidence.start_line
            || e.evidence.end_line > source.end_line
        {
            return Err(Error::corrupt(
                "spectral graph relation witness outside source",
            ));
        }
        if e.resolution == Resolution::Resolved && e.target.is_none() {
            return Err(Error::corrupt("spectral resolved relation target missing"));
        }
    }
    Ok(())
}

impl Engine<'_> {
    fn check(&self) -> Result<bool> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(Error::cancelled());
        }
        Ok(self.start.elapsed().as_millis() < MAX_MS as u128)
    }
    fn region(
        &mut self,
        seeds: &[(String, f64)],
        projection: &diffusion::Graph,
        h: &[f64],
    ) -> Result<Candidate> {
        let mut c = Candidate {
            ids: seeds.iter().map(|(id, _)| id.clone()).collect(),
            edges: BTreeSet::new(),
            imports: BTreeSet::new(),
            gaps: BTreeSet::new(),
            mass_omitted: 0.0,
            cut: false,
        };
        if projection
            .ids
            .len()
            .saturating_add(self.work.candidate_nodes)
            > MAX_NODE_VISITS
        {
            c.cut = true;
            c.gaps.insert("candidate_node_work_limit".into());
            return Ok(c);
        }
        self.work.candidate_nodes += projection.ids.len();
        let mut scores: Vec<_> = projection
            .ids
            .iter()
            .zip(h)
            .filter(|(id, w)| **w > 0.0 && self.graph.nodes[self.by_id[*id]].kind != "file")
            .collect();
        scores.sort_by(|(a, x), (b, y)| y.total_cmp(x).then(a.cmp(b)));
        let total: f64 = scores.iter().map(|(_, v)| **v).sum();
        let mut mass = 0.0;
        for (id, weight) in scores {
            if mass >= MASS * total {
                break;
            }
            if c.ids.len() >= MAX_REGION || !self.check()? {
                c.cut = true;
                c.gaps.insert("region_work_limit".into());
                break;
            }
            c.ids.insert(id.clone());
            mass += *weight;
        }
        c.mass_omitted = if total > 0.0 {
            ((total - mass) / total).max(0.0)
        } else {
            0.0
        };
        Ok(c)
    }
    fn close(&mut self, mut c: Candidate, scope: &str) -> Result<Candidate> {
        let mut queue: VecDeque<_> = c.ids.iter().map(|id| (id.clone(), 0usize)).collect();
        let mut seen = BTreeSet::new();
        while let Some((id, depth)) = queue.pop_front() {
            if !seen.insert(id.clone()) {
                continue;
            }
            if !self.check()? {
                c.cut = true;
                c.gaps.insert("global_time_limit".into());
                break;
            }
            if self.work.closure_node_visits >= MAX_NODE_VISITS {
                c.cut = true;
                c.gaps.insert("closure_node_work_limit".into());
                break;
            }
            self.work.closure_node_visits += 1;
            let node = self.graph.nodes[self.by_id[&id]].clone();
            self.lexical_imports(&node, &mut c)?;
            if let Some(coverage) = self
                .graph
                .coverage
                .iter()
                .find(|v| v.path_key == node.path_key)
                && (coverage.status != Resolution::Resolved || coverage.parse_errors > 0)
            {
                c.gaps.insert("unsupported_structure".into());
            }
            // Relations remain directed. Heat never supplies semantic witnesses.
            let edges = if scope == "callers" {
                self.incoming.get(&id)
            } else {
                self.outgoing.get(&id)
            }
            .cloned()
            .unwrap_or_default();
            for i in edges {
                if self.work.closure_edge_visits >= MAX_EDGE_VISITS || !self.check()? {
                    c.cut = true;
                    c.gaps.insert("closure_edge_or_time_limit".into());
                    break;
                }
                self.work.closure_edge_visits += 1;
                let edge = self.graph.edges[i].clone();
                if matches!(edge.kind, Relation::Calls | Relation::References) {
                    if self.document(&edge.evidence.path_key)? {
                        let doc = &self.docs[&edge.evidence.path_key];
                        let raw = doc
                            .bytes
                            .get(edge.evidence.start_byte..edge.evidence.end_byte)
                            .ok_or_else(|| {
                                Error::corrupt("spectral relation witness interval invalid")
                            })?;
                        if raw != edge.name.as_bytes() {
                            return Err(Error::corrupt(
                                "spectral relation witness does not match source token",
                            ));
                        }
                    } else {
                        c.gaps.insert("relation_source_verification_limit".into());
                    }
                }
                if edge.kind == Relation::Contains {
                    continue;
                }
                if edge.resolution != Resolution::Resolved || edge.target.is_none() {
                    c.gaps.insert(format!(
                        "unresolved_relation:{}",
                        edge.reason
                            .as_deref()
                            .unwrap_or("dynamic_external_or_ambiguous")
                    ));
                    continue;
                }
                if scope == "local" {
                    continue;
                }
                c.edges.insert(i);
                let next = if scope == "callers" {
                    &edge.source
                } else {
                    edge.target.as_ref().unwrap()
                };
                if !c.ids.contains(next) {
                    if depth >= MAX_DEPTH || c.ids.len() >= MAX_REGION {
                        c.cut = true;
                        c.gaps.insert("closure_limit".into());
                    } else {
                        c.ids.insert(next.clone());
                        queue.push_back((next.clone(), depth + 1));
                    }
                }
            }
        }
        Ok(c)
    }
    fn lexical_imports(&mut self, node: &Node, c: &mut Candidate) -> Result<()> {
        if node.kind == "file" {
            return Ok(());
        }
        let used: BTreeSet<_> = self
            .outgoing
            .get(&node.id)
            .into_iter()
            .flatten()
            .filter_map(|i| {
                let e = &self.graph.edges[*i];
                (e.kind == Relation::References).then(|| e.name.clone())
            })
            .collect();
        if used.is_empty() {
            return Ok(());
        }
        let mut ancestors = BTreeSet::new();
        let mut parent = Some(node.id.clone());
        while let Some(id) = parent {
            if !ancestors.insert(id.clone()) {
                break;
            }
            parent = self.graph.nodes[self.by_id[&id]].parent_id.clone();
        }
        let indices: Vec<_> = ancestors
            .iter()
            .flat_map(|id| self.outgoing.get(id).into_iter().flatten().copied())
            .filter(|i| self.graph.edges[*i].kind == Relation::Imports)
            .collect();
        for i in indices {
            if self.work.closure_edge_visits >= MAX_EDGE_VISITS || !self.check()? {
                c.cut = true;
                c.gaps.insert("lexical_import_work_limit".into());
                break;
            }
            self.work.closure_edge_visits += 1;
            let edge = self.graph.edges[i].clone();
            if !self.bindings.contains_key(&i) {
                if self.bindings.len() >= MAX_REGION {
                    c.cut = true;
                    c.gaps.insert("lexical_import_cache_limit".into());
                    continue;
                }
                if !self.document(&node.path_key)? {
                    c.gaps.insert("lexical_import_document_limit".into());
                    continue;
                }
                let doc = &self.docs[&node.path_key];
                if doc.content_sha256 != edge.evidence.content_sha256 {
                    return Err(Error::corrupt("lexical import revision mismatch"));
                }
                let raw = doc
                    .bytes
                    .get(edge.evidence.start_byte..edge.evidence.end_byte)
                    .ok_or_else(|| Error::corrupt("invalid lexical import witness"))?;
                let language = self
                    .graph
                    .coverage
                    .iter()
                    .find(|v| v.path_key == node.path_key)
                    .and_then(|v| v.language.as_deref())
                    .unwrap_or("");
                self.bindings.insert(i, import_bindings(language, raw)?);
            }
            if self.bindings[&i]
                .iter()
                .any(|binding| used.contains(binding))
            {
                c.imports.insert(i);
                if edge.resolution != Resolution::Resolved {
                    c.gaps.insert("external_or_ambiguous_lexical_import".into());
                }
            }
        }
        Ok(())
    }
    fn document(&mut self, key: &str) -> Result<bool> {
        if !self.check()? {
            return Ok(false);
        }
        if self.docs.contains_key(key) {
            return Ok(true);
        }
        if self.denied_docs.contains(key) {
            return Ok(false);
        }
        if !self.check()?
            || self.docs.len() >= MAX_DOCS
            || self.work.verified_document_bytes >= MAX_READ_BYTES
        {
            self.denied_docs.insert(key.into());
            return Ok(false);
        }
        let expected = self
            .collection
            .snapshot
            .entries
            .iter()
            .find(|e| e.path_key == key && e.state == "observed")
            .and_then(|e| e.size_bytes.as_deref())
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(|| Error::corrupt("spectral observed size missing"))?;
        if expected.saturating_add(self.work.cache_bytes) > MAX_CACHE
            || expected.saturating_add(self.work.verified_document_bytes) > MAX_READ_BYTES
        {
            self.denied_docs.insert(key.into());
            return Ok(false);
        }
        let d = self.collection.verified_document(key, self.cancel)?;
        self.work.verified_document_reads += 1;
        self.work.verified_document_bytes += d.bytes.len();
        if d.bytes.len().saturating_add(self.work.cache_bytes) > MAX_CACHE
            || self.work.verified_document_bytes > MAX_READ_BYTES
        {
            self.denied_docs.insert(key.into());
            return Ok(false);
        }
        self.work.cache_bytes += d.bytes.len();
        self.docs.insert(key.into(), d);
        Ok(true)
    }
    fn materialize(
        &mut self,
        c: &Candidate,
        r: &Request,
        theta: Option<f64>,
        reason: &str,
        diagnostics: Value,
    ) -> Result<Value> {
        let mut pack = json!({"schema_version":"3","report_type":"evidence_pack","generation":self.collection.generation_id(),"producer":PROFILE,"graph_producer":self.graph.producer,"query":r.query,"mode":r.mode,"scope":{"kind":r.scope,"origin":"provided"},"items":[],"relations":[],"omitted":[],"budget_bytes":r.budget_bytes,"payload_bytes":r.budget_bytes,"next_cursor":null,"acquisition":{"policy":POLICY,"state":"terminal","reason":reason},"structural":{"status":"static_scope_supported"},"coverage":{"scope":r.scope,"scope_supplied":true,"gaps":c.gaps,"limitations":["static_resolution_only","external_configuration_and_dynamic_behavior_not_guaranteed"],"mass_omitted":c.mass_omitted,"candidate_units":c.ids.len()+c.imports.len(),"omitted_units":0},"diagnostics":diagnostics});
        pack["diagnostics"]["selected_theta"] = json!(theta);
        let mut gaps = c.gaps.clone();
        let mut nodes: Vec<Node> = c
            .ids
            .iter()
            .map(|id| self.graph.nodes[self.by_id[id]].clone())
            .collect();
        for i in &c.imports {
            let e = &self.graph.edges[*i];
            nodes.push(Node {
                id: e.id.clone(),
                path_key: e.evidence.path_key.clone(),
                path: String::new(),
                content_sha256: e.evidence.content_sha256.clone(),
                kind: "lexical_import_witness".into(),
                name: e.name.clone(),
                scope: vec![],
                parent_id: None,
                start_byte: e.evidence.start_byte,
                end_byte: e.evidence.end_byte,
                start_line: e.evidence.start_line,
                end_line: e.evidence.end_line,
            });
        }
        // Coherent containing declarations cover nested declarations without slicing bodies.
        nodes.sort_by(|a, b| {
            a.path_key
                .cmp(&b.path_key)
                .then(a.start_byte.cmp(&b.start_byte))
                .then(b.end_byte.cmp(&a.end_byte))
        });
        let mut coherent: Vec<Node> = vec![];
        for node in nodes {
            if let Some(previous) = coherent.last_mut()
                && previous.path_key == node.path_key
                && previous.end_byte <= node.start_byte
                && self.document(&node.path_key)?
            {
                let doc = &self.docs[&node.path_key];
                if doc.bytes[previous.end_byte..node.start_byte]
                    .iter()
                    .all(u8::is_ascii_whitespace)
                {
                    previous.end_byte = node.end_byte;
                    previous.end_line = node.end_line;
                    continue;
                }
            }
            coherent.push(node);
        }
        let mut ranges: Vec<(String, usize, usize)> = vec![];
        let mut omitted = 0usize;
        for n in coherent {
            if !self.check()? {
                gaps.insert("global_time_limit".into());
                omitted += 1;
                continue;
            }
            if ranges
                .iter()
                .any(|(p, a, b)| *p == n.path_key && *a <= n.start_byte && *b >= n.end_byte)
            {
                continue;
            }
            if !self.document(&n.path_key)? {
                gaps.insert("document_cache_or_time_limit".into());
                omitted += 1;
                continue;
            }
            let doc = &self.docs[&n.path_key];
            if doc.content_sha256 != n.content_sha256 {
                return Err(Error::corrupt("spectral selected revision mismatch"));
            }
            let raw = doc
                .bytes
                .get(n.start_byte..n.end_byte)
                .ok_or_else(|| Error::corrupt("spectral invalid selected interval"))?;
            let Ok(text) = std::str::from_utf8(raw) else {
                gaps.insert("unsupported_non_utf8".into());
                omitted += 1;
                continue;
            };
            let start_line = 1 + doc.bytes[..n.start_byte]
                .iter()
                .filter(|b| **b == b'\n')
                .count();
            let end_line = start_line + raw.iter().filter(|b| **b == b'\n').count()
                - usize::from(raw.ends_with(b"\n"));
            if start_line != n.start_line
                || start_line + raw.iter().filter(|b| **b == b'\n').count() != n.end_line
            {
                return Err(Error::corrupt("spectral graph line interval mismatch"));
            }
            let item = json!({"node_id":n.id,"path_key":n.path_key,"content_sha256":n.content_sha256,"start_byte":n.start_byte,"end_byte":n.end_byte,"start_line":start_line,"end_line":end_line.max(start_line),"text":text,"verification":"full_object_sha256","methods":["spectral_region_static_closure"]});
            pack["items"].as_array_mut().unwrap().push(item);
            // Reserve explicit coverage metadata before accepting a whole unit.
            pack["coverage"]["gaps"] = json!(["budget_or_work_limit"]);
            pack["coverage"]["omitted_units"] = json!(c.ids.len());
            if codec::canonical(&pack)?.len() > r.budget_bytes {
                pack["items"].as_array_mut().unwrap().pop();
                omitted += 1;
                gaps.insert("budget_unit_does_not_fit".into());
            } else {
                ranges.push((n.path_key, n.start_byte, n.end_byte));
                self.work.materialized_units += 1;
            }
        }
        // A relation is delivered only when its witness bytes and both declarations are present.
        let mut delivered = 0;
        for i in &c.imports {
            let e = &self.graph.edges[*i];
            if !ranges.iter().any(|(p, a, b)| {
                *p == e.evidence.path_key
                    && *a <= e.evidence.start_byte
                    && *b >= e.evidence.end_byte
            }) {
                gaps.insert("lexical_import_witness_omitted".into());
            }
        }
        for i in &c.edges {
            if !self.check()? {
                gaps.insert("global_time_limit".into());
                break;
            }
            let e = &self.graph.edges[*i];
            let contained = |key: &str, a: usize, b: usize| {
                ranges
                    .iter()
                    .any(|(p, x, y)| p == key && *x <= a && *y >= b)
            };
            let endpoints = [Some(&e.source), e.target.as_ref()]
                .into_iter()
                .flatten()
                .all(|id| {
                    let n = &self.graph.nodes[self.by_id[id]];
                    contained(&n.path_key, n.start_byte, n.end_byte)
                });
            if endpoints
                && contained(
                    &e.evidence.path_key,
                    e.evidence.start_byte,
                    e.evidence.end_byte,
                )
            {
                let doc = &self.docs[&e.evidence.path_key];
                let raw = doc
                    .bytes
                    .get(e.evidence.start_byte..e.evidence.end_byte)
                    .ok_or_else(|| Error::corrupt("spectral invalid relation witness"))?;
                let line = 1 + doc.bytes[..e.evidence.start_byte]
                    .iter()
                    .filter(|b| **b == b'\n')
                    .count();
                if doc.content_sha256 != e.evidence.content_sha256
                    || std::str::from_utf8(raw).is_err()
                    || line != e.evidence.start_line
                    || line + raw.iter().filter(|b| **b == b'\n').count() != e.evidence.end_line
                {
                    return Err(Error::corrupt(
                        "spectral relation witness verification failed",
                    ));
                }
                pack["relations"].as_array_mut().unwrap().push(json!({"id":e.id,"source":e.source,"target":e.target,"kind":e.kind,"evidence":e.evidence,"verification":"full_object_sha256"}));
                if codec::canonical(&pack)?.len() + 256 > r.budget_bytes {
                    pack["relations"].as_array_mut().unwrap().pop();
                    gaps.insert("relation_metadata_budget".into());
                } else {
                    delivered += 1;
                }
            } else {
                gaps.insert("relation_witness_or_endpoint_omitted".into());
            }
        }
        pack["coverage"]["gaps"] = json!(gaps);
        pack["coverage"]["omitted_units"] = json!(omitted);
        pack["coverage"]["directed_relations_delivered"] = json!(delivered);
        pack["structural"]["status"] = json!(if gaps.is_empty() {
            "static_scope_supported"
        } else {
            "gaps"
        });
        // Metadata is part of the budget. Additional final metadata can remove only whole units.
        while codec::canonical(&pack)?.len() > r.budget_bytes {
            if pack["items"].as_array_mut().unwrap().pop().is_none() {
                return Err(Error::invalid(
                    "budget cannot fit spectral response metadata",
                ));
            }
            omitted += 1;
            pack["coverage"]["omitted_units"] = json!(omitted);
            gaps.insert("budget_unit_does_not_fit".into());
            gaps.insert("relation_witness_or_endpoint_omitted".into());
            pack["coverage"]["gaps"] = json!(gaps);
            pack["relations"] = json!([]);
            pack["coverage"]["directed_relations_delivered"] = json!(0);
            pack["structural"]["status"] = json!("gaps");
        }
        for _ in 0..4 {
            let size = codec::canonical(&pack)?.len();
            if pack["payload_bytes"] == json!(size) {
                break;
            }
            pack["payload_bytes"] = json!(size);
        }
        Ok(pack)
    }
}

fn import_bindings(language: &str, raw: &[u8]) -> Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();
    let grammar = match language {
        "python" => tree_sitter_python::LANGUAGE.into(),
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "javascript" => tree_sitter_javascript::LANGUAGE.into(),
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        _ => return Ok(out),
    };
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&grammar)
        .map_err(|e| Error::corrupt(e.to_string()))?;
    let Some(tree) = parser.parse(raw, None) else {
        return Ok(out);
    };
    let text = |n: tree_sitter::Node<'_>| {
        std::str::from_utf8(&raw[n.start_byte()..n.end_byte()])
            .unwrap_or("")
            .to_owned()
    };
    let mut pending = vec![tree.root_node()];
    while let Some(n) = pending.pop() {
        if language == "python" && matches!(n.kind(), "import_statement" | "import_from_statement")
        {
            let module = n.child_by_field_name("module_name");
            let from = n.kind() == "import_from_statement";
            let mut cursor = n.walk();
            for child in n.named_children(&mut cursor) {
                if Some(child) == module {
                    continue;
                }
                if child.kind() == "aliased_import" {
                    if let Some(alias) = child.child_by_field_name("alias") {
                        out.insert(text(alias));
                    }
                } else if child.kind() == "dotted_name" {
                    let name = text(child);
                    out.insert(if from {
                        name
                    } else {
                        name.split('.').next().unwrap_or("").into()
                    });
                }
            }
            continue;
        }
        if language == "rust" && n.kind() == "use_declaration" {
            let value = text(n);
            let value = value
                .trim()
                .trim_start_matches("pub ")
                .trim_start_matches("use ")
                .trim_end_matches(';')
                .trim();
            if !value.contains(['{', '}', '*']) {
                let local = value
                    .split_once(" as ")
                    .map(|(_, alias)| alias)
                    .unwrap_or_else(|| value.rsplit("::").next().unwrap_or(""));
                out.insert(local.into());
            }
            continue;
        }
        if matches!(
            n.kind(),
            "import_specifier" | "namespace_import" | "import_clause"
        ) {
            if let Some(alias) = n
                .child_by_field_name("alias")
                .or_else(|| n.child_by_field_name("name"))
            {
                out.insert(text(alias));
            } else {
                let mut cursor = n.walk();
                for child in n.named_children(&mut cursor) {
                    if child.kind() == "identifier" {
                        out.insert(text(child));
                    }
                }
            }
        }
        let mut cursor = n.walk();
        pending.extend(n.named_children(&mut cursor));
    }
    Ok(out)
}

pub fn retrieve(c: &Collection, r: &Request, cancel: &AtomicBool) -> Result<Value> {
    Ok(run(c, r, "spectral_window_v1", None, false, None, cancel)?.0)
}

/// Backward-compatible opt-in intent entry point used by CLI and MCP.
pub fn retrieve_with_intent(
    c: &Collection,
    r: &Request,
    intent: Option<&QueryIntent>,
    cancel: &AtomicBool,
) -> Result<Value> {
    if let Some(intent) = intent {
        intent.validate(r)?;
    }
    let mut pack = run(c, r, POLICY, None, false, intent, cancel)?.0;
    if let Some(intent) = intent
        && pack.get("query_intent").is_none()
    {
        pack["query_intent"] = json!(intent);
        pack["diagnostics"]["effective_seeds"] = json!([]);
        pack["diagnostics"]["target_outcomes"] = json!([]);
        for _ in 0..4 {
            let size = codec::canonical(&pack)?.len();
            pack["payload_bytes"] = json!(size);
        }
        if codec::canonical(&pack)?.len() > r.budget_bytes {
            return Err(Error::invalid("budget cannot fit intent metadata"));
        }
    }
    Ok(pack)
}

/// Laboratory-only adapter; all arms share discovery, closure and final materialization.
#[doc(hidden)]
pub fn laboratory(
    c: &Collection,
    r: &Request,
    strategy: &str,
    theta: Option<f64>,
    cancel: &AtomicBool,
) -> Result<(Value, Value, Value)> {
    run(c, r, strategy, theta, true, None, cancel)
}

fn semantic_graph_digest(graph: &Graph) -> Result<String> {
    Ok(model::digest(
        &serde_json::to_vec(&(
            &graph.schema_version,
            &graph.generation,
            &graph.nodes,
            &graph.edges,
            &graph.coverage,
        ))
        .map_err(|e| Error::corrupt(e.to_string()))?,
    ))
}
fn structural_limit(
    c: &Collection,
    r: &Request,
    strategy: &str,
    work: Value,
) -> Result<(Value, Value, Value)> {
    let mut pack = json!({"schema_version":"3","report_type":"evidence_pack","generation":c.generation_id(),"producer":PROFILE,"query":r.query,"mode":r.mode,"scope":{"kind":r.scope,"origin":"provided"},"items":[],"omitted":[],"budget_bytes":r.budget_bytes,"payload_bytes":r.budget_bytes,"next_cursor":null,"acquisition":{"policy":strategy,"state":"terminal","reason":"structural_work_limit"},"structural":{"status":"not_revalidated"},"coverage":{"gaps":["complete_generation_source_revalidation_limit"],"scope":r.scope,"scope_supplied":true},"diagnostics":{"profile":PROFILE,"limits":{"generation_documents":MAX_DOCS,"generation_bytes":MAX_READ_BYTES/3,"elapsed_ms":MAX_MS},"work":work}});
    for _ in 0..4 {
        let size = codec::canonical(&pack)?.len();
        if size > r.budget_bytes {
            return Err(Error::invalid(
                "budget cannot fit structural limit metadata",
            ));
        }
        if pack["payload_bytes"] == json!(size) {
            break;
        }
        pack["payload_bytes"] = json!(size);
    }
    Ok((pack, work, json!([])))
}

#[allow(clippy::too_many_arguments)]
fn run(
    c: &Collection,
    r: &Request,
    strategy: &str,
    fixed: Option<f64>,
    lab: bool,
    intent: Option<&QueryIntent>,
    cancel: &AtomicBool,
) -> Result<(Value, Value, Value)> {
    if !matches!(r.scope.as_str(), "local" | "dependencies" | "callers")
        || !(2048..=1024 * 1024).contains(&r.budget_bytes)
    {
        return Err(Error::invalid(
            "invalid spectral scope or budget (2048..1048576)",
        ));
    }
    if !matches!(
        strategy,
        "none" | "bfs" | "fixed_heat" | "spectral_window_v1"
    ) || (strategy != "fixed_heat" && fixed.is_some())
        || fixed.is_some_and(|v| !v.is_finite() || v <= 0.0 || v > 256.0)
    {
        return Err(Error::invalid("invalid laboratory strategy/theta"));
    }
    if r.query.trim().is_empty()
        || r.query.len() > 4096
        || !matches!(
            r.mode.as_str(),
            "literal" | "lexical" | "semantic" | "hybrid"
        )
    {
        return Err(Error::invalid("invalid spectral query or mode"));
    }
    let start = Instant::now();
    if cancel.load(Ordering::Relaxed) {
        return Err(Error::cancelled());
    }
    let observed: Vec<_> = c
        .snapshot
        .entries
        .iter()
        .filter(|e| e.state == "observed")
        .collect();
    let generation_bytes = observed
        .iter()
        .try_fold(0usize, |sum, e| {
            e.size_bytes
                .as_deref()
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|size| sum.checked_add(size))
        })
        .ok_or_else(|| Error::corrupt("source validation generation size invalid"))?;
    if observed.len() > MAX_DOCS || generation_bytes > MAX_READ_BYTES / 3 {
        return structural_limit(
            c,
            r,
            strategy,
            json!({"generation_documents":observed.len(),"generation_bytes":generation_bytes,"elapsed_ms":start.elapsed().as_millis()}),
        );
    }
    // The complete generation is the resolution universe. A subset could erase
    // an ambiguous module or binding and manufacture a resolved relation.
    let documents = c.verified_documents(cancel)?;
    let source_documents: Vec<_> = documents
        .iter()
        .map(|d| {
            Ok(crate::structure::Document {
                path: String::from_utf8_lossy(&model::decode_path_key(&d.path_key)?).into_owned(),
                path_key: d.path_key.clone(),
                content_sha256: d.content_sha256.clone(),
                bytes: d.bytes.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let rebuilt = crate::structure::build(&source_documents, c.generation_id().into(), cancel)?;
    let source_revalidation_elapsed_ms = start.elapsed().as_millis() as u64;
    if source_revalidation_elapsed_ms >= MAX_MS {
        return structural_limit(
            c,
            r,
            strategy,
            json!({"verified_document_reads":documents.len(),"verified_document_bytes":generation_bytes,"source_revalidation_elapsed_ms":source_revalidation_elapsed_ms}),
        );
    }
    let discovery = retrieval::acquisition_discover(c, &r.query, &r.mode, cancel);
    if cancel.load(Ordering::Relaxed) {
        return Err(Error::cancelled());
    }
    let (graph, ranked, ranking) = discovery?;
    validate_graph(c, &graph, cancel)?;
    if semantic_graph_digest(&graph)? != semantic_graph_digest(&rebuilt)? {
        return Err(Error::corrupt(
            "derived factual graph does not match complete verified generation",
        ));
    }
    let revalidation_producer = model::digest(rebuilt.producer.as_bytes());
    let source_revalidation_nodes = rebuilt.nodes.len();
    let source_revalidation_edges = rebuilt.edges.len();
    drop(rebuilt);
    drop(source_documents);
    let ranking_cut = ranking["lexical"]["potentially_truncated"] == true
        || ranking["semantic"]["potentially_truncated"] == true
        || ranking["literal"]["matches_truncated"] == true
        || ranking["literal"]["status"]
            .as_str()
            .is_some_and(|s| matches!(s, "partial" | "unavailable"));
    let ranking_summary = json!({"candidate_limit_per_channel":ranking["candidate_limit_per_channel"],"lexical":ranking["lexical"],"literal":ranking["literal"],"semantic":{"status":ranking["semantic"]["status"],"potentially_truncated":ranking["semantic"]["potentially_truncated"],"segments":ranking["semantic"]["segments"]},"canonicalization":ranking["canonicalization"],"candidate_cut":ranking_cut});
    let by_id: BTreeMap<_, _> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect();
    let mut outgoing: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut incoming: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, e) in graph.edges.iter().enumerate() {
        outgoing.entry(e.source.clone()).or_default().push(i);
        if let Some(id) = &e.target {
            incoming.entry(id.clone()).or_default().push(i);
        }
    }
    let edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| {
            e.resolution == Resolution::Resolved
                && matches!(
                    e.kind,
                    Relation::Contains | Relation::Imports | Relation::Calls
                )
        })
        .filter_map(|e| e.target.as_ref().map(|t| (e.source.clone(), t.clone())))
        .collect();
    let projection =
        diffusion::Graph::new(graph.nodes.iter().map(|n| n.id.clone()).collect(), &edges)?;
    let projection_digest = model::digest(&codec::canonical(&json!([
        PROJECTION,
        projection.ids,
        edges
    ]))?);
    let mut seedmap: BTreeMap<String, f64> = BTreeMap::new();
    let mut seed_cut = false;
    let mut seed_node_visits = 0usize;
    let mut seed_work_cut = false;
    let mut by_path: BTreeMap<&str, Vec<&Node>> = BTreeMap::new();
    for n in &graph.nodes {
        by_path.entry(&n.path_key).or_default().push(n);
    }
    let tokens: Vec<_> = r
        .query
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .collect();
    let mut target_outcomes = vec![];
    let mut intent_gaps = BTreeSet::new();
    if let Some(intent) = intent.filter(|i| i.match_preference == "exact") {
        for target in &intent.targets {
            let mut matches = vec![];
            for node in &graph.nodes {
                seed_node_visits += 1;
                if cancel.load(Ordering::Relaxed) {
                    return Err(Error::cancelled());
                }
                if seed_node_visits > MAX_NODE_VISITS
                    || start.elapsed().as_millis() >= MAX_MS as u128
                {
                    seed_work_cut = true;
                    break;
                }
                let name_matches = target.name.as_ref().map_or(node.kind == "file", |name| {
                    node.kind != "file" && &node.name == name
                });
                let path_matches = match &target.path {
                    Some(path) => model::decode_path_key(&node.path_key)? == path.as_bytes(),
                    None => true,
                };
                if name_matches && path_matches {
                    matches.push(node.id.clone());
                }
            }
            let status = if seed_work_cut {
                "work_limit"
            } else if matches.len() == 1 {
                seedmap.insert(matches[0].clone(), 1.0);
                "resolved"
            } else if matches.is_empty() {
                "not_found"
            } else {
                "ambiguous"
            };
            if status != "resolved" {
                intent_gaps.insert(format!("target_{status}"));
            }
            target_outcomes.push(json!({"target":target,"status":status,"match_count":matches.len(),"node_id":if status == "resolved" {matches.first()} else {None}}));
        }
    }
    for hit in ranked
        .as_array()
        .unwrap()
        .iter()
        .filter(|_| intent.is_none_or(|i| i.match_preference != "exact"))
    {
        let key = hit["path_key"].as_str().unwrap();
        let a = hit["start_byte"].as_u64().unwrap_or(0) as usize;
        let b = hit["end_byte"].as_u64().unwrap_or(0) as usize;
        let file_nodes = by_path.get(key).cloned().unwrap_or_default();
        seed_node_visits += file_nodes.len();
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::cancelled());
        }
        if seed_node_visits > MAX_NODE_VISITS || start.elapsed().as_millis() >= MAX_MS as u128 {
            seed_work_cut = true;
            break;
        }
        let mut matches: Vec<_> = file_nodes
            .iter()
            .copied()
            .filter(|n| {
                n.path_key == key
                    && n.kind != "file"
                    && n.start_byte < b
                    && n.end_byte > a
                    && tokens.iter().any(|t| n.name.eq_ignore_ascii_case(t))
            })
            .collect();
        matches.sort_by(|a, b| {
            (a.end_byte - a.start_byte)
                .cmp(&(b.end_byte - b.start_byte))
                .then(a.id.cmp(&b.id))
        });
        if matches.is_empty()
            && let Some(n) = file_nodes
                .iter()
                .copied()
                .filter(|n| n.path_key == key && n.start_byte <= a && n.end_byte >= b)
                .min_by_key(|n| n.end_byte - n.start_byte)
        {
            matches.push(n);
        }
        for n in matches {
            if !seedmap.contains_key(&n.id) && seedmap.len() >= MAX_SEEDS {
                seed_cut = true;
                continue;
            }
            *seedmap.entry(n.id.clone()).or_default() += hit["score"].as_f64().unwrap_or(0.0);
        }
    }
    let sum: f64 = seedmap.values().sum();
    let seeds: Vec<_> = seedmap
        .into_iter()
        .map(|(id, w)| (id, if sum > 0.0 { w / sum } else { 0.0 }))
        .collect();
    let effective_seeds: Vec<_> = seeds
        .iter()
        .map(|(id, weight)| {
            let node = &graph.nodes[by_id[id]];
            json!({"node_id":id,"name":node.name,"path_key":node.path_key,"weight":weight})
        })
        .collect();
    let seed_digest = model::digest(&codec::canonical(&json!(seeds))?);
    let mut engine = Engine {
        collection: c,
        graph,
        by_id,
        outgoing,
        incoming,
        docs: documents
            .into_iter()
            .map(|d| (d.path_key.clone(), d))
            .collect(),
        denied_docs: BTreeSet::new(),
        bindings: BTreeMap::new(),
        work: Work {
            source_revalidation_bytes: generation_bytes,
            source_revalidation_nodes,
            source_revalidation_edges,
            source_revalidation_elapsed_ms,
            verified_document_reads: observed.len(),
            verified_document_bytes: generation_bytes,
            cache_bytes: generation_bytes,
            seed_node_visits,
            graph_nodes: projection.ids.len(),
            graph_edges: edges.len(),
            projection_node_visits: projection.ids.len(),
            projection_edge_visits: edges.len(),
            ranking_candidates: ranked.as_array().unwrap().len(),
            ranking_verified_reads: ranking["canonicalization"]["verified_files"]
                .as_u64()
                .unwrap_or(0) as usize,
            ranking_literal_verified_files: ranking["literal"]["verified_text_files"]
                .as_str()
                .and_then(|s| s.parse::<usize>().ok())
                .map(|text| {
                    text + ranking["literal"]["non_text_files"]
                        .as_str()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0)
                }),
            ..Work::default()
        },
        start,
        cancel,
    };
    let limits = diffusion::SpectralLimits {
        max_elapsed_ms: MAX_MS
            .saturating_sub(start.elapsed().as_millis() as u64)
            .max(1),
        max_samples: 16,
        ..Default::default()
    };
    let mut numeric = projection.spectral_trajectory(
        &seeds,
        diffusion::SpectralConfig::default(),
        limits.clone(),
        cancel,
    )?;
    let mut trajectory = vec![];
    let mut cached_pack = None;
    let mut reason = "no_window";
    let mut selected = None;
    let mut max_theta = None;
    let mut quality = "not_sampled".to_owned();
    let mut last = Candidate {
        ids: seeds.iter().map(|(id, _)| id.clone()).collect(),
        edges: BTreeSet::new(),
        imports: BTreeSet::new(),
        gaps: BTreeSet::new(),
        mass_omitted: 0.0,
        cut: false,
    };
    last.gaps.extend(intent_gaps);
    if seed_cut {
        last.cut = true;
        last.gaps.insert("seed_limit".into());
    }
    if seed_work_cut {
        last.cut = true;
        last.gaps.insert("seed_discovery_work_limit".into());
    }
    if ranking_cut {
        last.cut = true;
        last.gaps
            .insert("ranking_candidates_truncated_or_scan_partial".into());
    }
    // Measure the static closure first; trajectory attribution excludes these nodes.
    // Work spent on this diagnostic is charged to the same existing limits.
    let baseline = if intent.is_some() {
        Some(engine.close(last.clone(), &r.scope)?)
    } else {
        None
    };
    let seed_gaps = last.gaps.clone();
    let mut previous_interval: Option<(f64, f64)> = None;
    let mut rise = false;
    let mut fall = false;
    let mut peak = 0.0f64;
    let mut pending: Option<(f64, Candidate, usize)> = None;
    let scales: Vec<f64> = match strategy {
        "none" | "bfs" => vec![],
        "fixed_heat" => vec![fixed.unwrap_or(1.0)],
        _ => (0..15).map(|i| 2f64.powi(i - 6)).collect(),
    };
    if matches!(strategy, "none" | "bfs") {
        let mut h = vec![0.0; projection.ids.len()];
        for (id, w) in &seeds {
            if let Ok(i) = projection.ids.binary_search(id) {
                h[i] = *w;
            }
        }
        if strategy == "bfs" {
            let b = projection.bfs(
                &seeds.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(),
                2,
                MAX_REGION,
                100_000,
                cancel,
            )?;
            engine.work.bfs_edge_visits += b.edge_visits;
            for (i, _) in b.nodes {
                h[i] = 1.0;
            }
            if !b.complete {
                last.cut = true;
                last.gaps.insert("bfs_work_limit".into());
            }
        }
        let mut region = if strategy == "bfs" {
            engine.work.candidate_nodes += projection.ids.len();
            Candidate {
                ids: projection
                    .ids
                    .iter()
                    .zip(&h)
                    .filter(|(_, w)| **w > 0.0)
                    .map(|(id, _)| id.clone())
                    .collect(),
                edges: BTreeSet::new(),
                imports: BTreeSet::new(),
                gaps: BTreeSet::new(),
                mass_omitted: 0.0,
                cut: false,
            }
        } else {
            engine.region(&seeds, &projection, &h)?
        };
        region.gaps.extend(last.gaps.clone());
        region.cut |= last.cut;
        last = engine.close(region, &r.scope)?;
        reason = "fixed_policy";
    }
    for theta in scales {
        if !engine.check()? {
            reason = "work_limit";
            break;
        }
        let Some(sample) = numeric.sample(theta, cancel)? else {
            reason = "work_limit";
            break;
        };
        max_theta = Some(theta);
        engine.work.scales += 1;
        quality = sample.quality.clone();
        let mut region = engine.region(&seeds, &projection, &sample.h)?;
        region.gaps.extend(seed_gaps.clone());
        region.cut |= seed_cut || seed_work_cut || ranking_cut;
        let candidate = engine.close(region, &r.scope)?;
        let diagnostics = json!({"profile":PROFILE,"projection":PROJECTION,"projection_sha256":projection_digest,"scope":r.scope,"scope_supplied":true,"selected_theta":theta,"max_evaluated_theta":theta,"quality":quality});
        let trial = engine.materialize(&candidate, r, Some(theta), "no_window", diagnostics)?;
        let eligible = codec::canonical(&trial)?.len().saturating_add(2048) <= r.budget_bytes
            && !candidate.cut
            && candidate.gaps.is_empty()
            && trial["coverage"]["gaps"]
                .as_array()
                .is_some_and(Vec::is_empty)
            && quality == "estimated";
        if lab {
            let record = json!({"theta":theta,"c_q":sample.c_q,"c_q_interval_estimate":sample.c_q_interval_estimate,"quality":quality,"candidate_node_ids":candidate.ids,"candidate_relations":candidate.edges.len(),"lexical_import_witnesses":candidate.imports.len(),"evidence_intervals":trial["items"].as_array().unwrap().iter().map(|i|json!([i["path_key"],i["start_byte"],i["end_byte"]])).collect::<Vec<_>>(),"gaps":trial["coverage"]["gaps"],"window_eligible":eligible});
            engine.work.trajectory_bytes += codec::canonical(&record)?.len();
            trajectory.push(record);
        }
        cached_pack = Some(trial);
        if candidate.cut && !seed_cut {
            selected = Some(theta);
            last = candidate;
            reason = "work_limit";
            break;
        }
        if let Some(interval) = &sample.c_q_interval_estimate {
            if let Some((lo, hi)) = previous_interval {
                if interval.lower > hi {
                    rise = true;
                }
                if rise && interval.upper < lo {
                    fall = true;
                }
            }
            peak = peak.max(interval.lower);
            let low = rise && fall && interval.upper < LOW * peak && eligible;
            if low {
                match &mut pending {
                    Some((_, first, count))
                        if first.ids == candidate.ids
                            && first.edges == candidate.edges
                            && first.imports == candidate.imports =>
                    {
                        *count += 1;
                    }
                    _ => {
                        pending = Some((theta, candidate.clone(), 1));
                    }
                }
                if let Some((t, first, count)) = &pending
                    && *count >= PERSISTENCE
                {
                    selected = Some(*t);
                    last = first.clone();
                    reason = "spectral_window";
                    break;
                }
            } else {
                pending = None;
            }
            previous_interval = Some((interval.lower, interval.upper));
        } else {
            pending = None;
        }
        selected = Some(theta);
        last = candidate;
        if strategy == "fixed_heat" {
            reason = "fixed_policy";
        }
    }
    engine.work.numeric = numeric.work();
    if numeric.status() != "complete" {
        reason = "work_limit";
        last.gaps.insert(format!("numeric_{}", numeric.status()));
    }
    if seeds.is_empty() {
        reason = "no_evidence";
    } else if reason == "no_window" && quality != "estimated" {
        reason = "numerical_indeterminate";
        last.gaps.insert(format!("numeric_{}", quality));
    }
    if !engine.check()? {
        reason = "work_limit";
        last.gaps.insert("global_time_limit".into());
    }
    let diagnostics = json!({"profile":PROFILE,"projection":PROJECTION,"projection_sha256":projection_digest,"revalidation_producer_sha256":revalidation_producer,"revalidation_universe":"complete_generation","seed_policy":"ranked_chunk_exact_name_else_enclosing_v1","seed_count":seeds.len(),"seed_sha256":seed_digest,"seed_normalization":"l1_once","ranking":ranking_summary,"seed_limit":MAX_SEEDS,"seed_limit_hit":seed_cut,"scope":r.scope,"scope_supplied":true,"max_evaluated_theta":max_theta,"scales":engine.work.scales,"quality":quality,"profile_parameters":{"start_theta":0.015625,"max_theta":256.0,"ratio":2.0,"mass_fraction":MASS,"relative_curvature":LOW,"persistence":PERSISTENCE,"window_metadata_reserve_bytes":2048},"limits":{"elapsed_ms":MAX_MS,"region_nodes":MAX_REGION,"closure_depth":MAX_DEPTH,"cache_bytes":MAX_CACHE,"cache_documents":MAX_DOCS,"read_bytes":MAX_READ_BYTES,"generation_revalidation_bytes":MAX_READ_BYTES/3,"candidate_node_visits":MAX_NODE_VISITS,"closure_node_visits":MAX_NODE_VISITS,"closure_edge_visits":MAX_EDGE_VISITS,"numeric":limits},"work":engine.work});
    // Keep the already verified candidate payload when a later numeric/work limit
    // terminates the trajectory. Materialization is never restarted after expiry.
    let mut pack = if let Some(pack) = cached_pack {
        pack
    } else {
        engine.materialize(&last, r, selected, reason, diagnostics.clone())?
    };
    pack["diagnostics"] = diagnostics;
    pack["diagnostics"]["selected_theta"] = json!(selected);
    if let Some(intent) = intent {
        pack["query_intent"] = json!(intent);
        pack["diagnostics"]["seed_policy"] = json!("query_intent_v1");
        pack["diagnostics"]["effective_seeds"] = json!(effective_seeds);
        pack["diagnostics"]["target_outcomes"] = json!(target_outcomes);
        if let Some(baseline) = &baseline {
            pack["diagnostics"]["trajectory_contribution"] = json!({"baseline":"seed_static_closure_spans", "baseline_complete":!baseline.cut && baseline.gaps.is_empty(), "attribution":"novel_non_whitespace_bytes_per_delivered_item"});
            let mut spans: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
            for id in &baseline.ids {
                let node = &engine.graph.nodes[engine.by_id[id]];
                spans
                    .entry(node.path_key.clone())
                    .or_default()
                    .push((node.start_byte, node.end_byte));
            }
            for index in &baseline.imports {
                let witness = &engine.graph.edges[*index].evidence;
                spans
                    .entry(witness.path_key.clone())
                    .or_default()
                    .push((witness.start_byte, witness.end_byte));
            }
            for ranges in spans.values_mut() {
                ranges.sort_unstable();
            }
            for item in pack["items"].as_array_mut().unwrap() {
                let id = item["node_id"].as_str().unwrap_or("");
                let key = item["path_key"].as_str().unwrap();
                let start = item["start_byte"].as_u64().unwrap() as usize;
                let end = item["end_byte"].as_u64().unwrap() as usize;
                let ranges = spans.get(key).map(Vec::as_slice).unwrap_or(&[]);
                let mut uncovered = 0;
                let mut cursor = start;
                let bytes = &engine.docs[key].bytes;
                for &(a, b) in ranges {
                    if b <= cursor {
                        continue;
                    }
                    if a >= end {
                        break;
                    }
                    if a > cursor {
                        uncovered += bytes[cursor..a.min(end)]
                            .iter()
                            .filter(|b| !b.is_ascii_whitespace())
                            .count();
                    }
                    cursor = cursor.max(b).min(end);
                    if cursor == end {
                        break;
                    }
                }
                uncovered += bytes[cursor..end]
                    .iter()
                    .filter(|b| !b.is_ascii_whitespace())
                    .count();
                let incomplete = baseline.cut || !baseline.gaps.is_empty();
                let origin = if uncovered == 0 && seeds.iter().any(|(seed, _)| seed == id) {
                    "seed"
                } else if uncovered == 0 {
                    "static_closure"
                } else if incomplete {
                    "undetermined_baseline_incomplete"
                } else {
                    "trajectory_region_or_its_closure"
                };
                item["origin"] = json!(origin);
                item["trajectory_new_non_whitespace_bytes"] = if incomplete && uncovered > 0 {
                    Value::Null
                } else {
                    json!(uncovered)
                };
            }
        }
    }
    pack["acquisition"]["reason"] = json!(reason);
    pack["coverage"]["mass_omitted"] = json!(last.mass_omitted);
    for gap in &last.gaps {
        let gaps = pack["coverage"]["gaps"].as_array_mut().unwrap();
        if !gaps.contains(&json!(gap)) {
            gaps.push(json!(gap));
        }
    }
    if !pack["coverage"]["gaps"].as_array().unwrap().is_empty() {
        pack["structural"]["status"] = json!("gaps");
    }
    if strategy != POLICY {
        pack["acquisition"]["policy"] = json!(strategy);
    }
    if pack["coverage"]["gaps"]
        .as_array()
        .unwrap()
        .contains(&json!("global_time_limit"))
    {
        pack["acquisition"]["reason"] = json!("work_limit");
    }
    if pack["acquisition"]["reason"] == "spectral_window"
        && !pack["coverage"]["gaps"].as_array().unwrap().is_empty()
    {
        pack["acquisition"]["reason"] = json!("budget_limit");
    }
    engine.work.total_verified_document_reads = engine.work.ranking_verified_reads
        + engine.work.ranking_literal_verified_files.unwrap_or(0)
        + engine.work.verified_document_reads;
    engine.work.elapsed_ms = start.elapsed().as_millis() as u64;
    pack["diagnostics"]["work"] = json!(engine.work);
    while codec::canonical(&pack)?.len() > r.budget_bytes {
        if pack["items"].as_array_mut().unwrap().pop().is_none() {
            return Err(Error::invalid(
                "budget cannot fit spectral response metadata",
            ));
        }
        pack["coverage"]["omitted_units"] =
            json!(pack["coverage"]["omitted_units"].as_u64().unwrap_or(0) + 1);
        let gaps = pack["coverage"]["gaps"].as_array_mut().unwrap();
        if !gaps.contains(&json!("budget_unit_does_not_fit")) {
            gaps.push(json!("budget_unit_does_not_fit"));
        }
        pack["coverage"]["directed_relations_delivered"] = json!(0);
        pack["relations"] = json!([]);
        pack["structural"]["status"] = json!("gaps");
        if reason == "spectral_window" {
            pack["acquisition"]["reason"] = json!("work_limit");
        }
    }
    // Final payload is checked after every budget decision. Objects in this bounded cache
    // were hash-verified when read; no live filesystem contents replace pinned bytes.
    for item in pack["items"].as_array().unwrap() {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::cancelled());
        }
        let doc = &engine.docs[item["path_key"].as_str().unwrap()];
        let a = item["start_byte"].as_u64().unwrap() as usize;
        let b = item["end_byte"].as_u64().unwrap() as usize;
        if item["content_sha256"] != doc.content_sha256
            || doc.bytes.get(a..b) != item["text"].as_str().map(str::as_bytes)
        {
            return Err(Error::corrupt("spectral final payload verification failed"));
        }
    }
    for _ in 0..4 {
        let size = codec::canonical(&pack)?.len();
        if pack["payload_bytes"] == json!(size) {
            break;
        }
        pack["payload_bytes"] = json!(size);
    }
    if !engine.check()? {
        pack["acquisition"]["reason"] = json!("work_limit");
        let gaps = pack["coverage"]["gaps"].as_array_mut().unwrap();
        if !gaps.contains(&json!("global_time_limit")) {
            gaps.push(json!("global_time_limit"));
        }
        pack["structural"]["status"] = json!("gaps");
        if codec::canonical(&pack)?.len() > r.budget_bytes {
            return Err(Error::invalid(
                "budget cannot fit terminal time-limit metadata",
            ));
        }
        for _ in 0..4 {
            let size = codec::canonical(&pack)?.len();
            if pack["payload_bytes"] == json!(size) {
                break;
            }
            pack["payload_bytes"] = json!(size);
        }
    }
    Ok((
        pack,
        serde_json::to_value(engine.work).map_err(|e| Error::corrupt(e.to_string()))?,
        json!(trajectory),
    ))
}
