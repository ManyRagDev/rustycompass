//! Generation-bound derived indexes and budgeted, verified context retrieval.
use crate::{
    Error, Result, codec,
    collection::{self, Collection, PrepareRequest},
    diffusion, model, semantic, structure,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
const VERSION: &str = "retrieval-v1-rrf60-chunks80";
const MAX_DERIVED: u64 = 128 * 1024 * 1024;
const CANDIDATES: usize = 200;
#[derive(Serialize, Deserialize)]
struct Derived {
    version: String,
    generation: String,
    graph: structure::Graph,
    model: Option<semantic::ModelIdentity>,
    model_dir: Option<PathBuf>,
    segments: Vec<semantic::Segment>,
    semantic_skipped: Vec<String>,
    reused_semantic_files: usize,
}
fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::cancelled())
    } else {
        Ok(())
    }
}
fn failure(e: impl std::fmt::Display) -> Error {
    Error::corrupt(e.to_string())
}
fn load(c: &Collection) -> Result<Derived> {
    let mut f = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(c.pinned_generation_dir().join("derived.json"))
        .map_err(|e| {
            Error::io(
                "derived indexes unavailable; prepare/update the collection",
                e,
            )
        })?;
    if !f.metadata().map_err(failure)?.is_file() {
        return Err(failure("derived index must be regular"));
    }
    let mut bytes = vec![];
    (&mut f)
        .take(MAX_DERIVED + 1)
        .read_to_end(&mut bytes)
        .map_err(failure)?;
    if bytes.len() as u64 > MAX_DERIVED {
        return Err(failure("derived index size limit"));
    }
    let envelope: Value = serde_json::from_slice(&bytes).map_err(failure)?;
    let data = &envelope["data"];
    if envelope["sha256"] != model::digest(&codec::canonical(data)?) {
        return Err(failure("derived index digest mismatch"));
    }
    let d: Derived = serde_json::from_value(data.clone()).map_err(failure)?;
    if d.version != VERSION
        || d.generation != c.generation_id()
        || d.graph.generation != c.generation_id()
    {
        return Err(failure("derived producer/generation mismatch"));
    }
    Ok(d)
}
// Acquisition reuses the verified, generation-bound graph loader. Keep the
// legacy retrieval producer and wire format unchanged.
pub(crate) fn acquisition_graph(c: &Collection) -> Result<structure::Graph> {
    Ok(load(c)?.graph)
}
fn derive(
    c: &Collection,
    previous: Option<&Collection>,
    model_dir: Option<&Path>,
    cancel: &AtomicBool,
) -> Result<Value> {
    check(cancel)?;
    let documents = c.verified_documents(cancel)?;
    let docs: Vec<_> = documents
        .into_iter()
        .map(|d| structure::Document {
            path: model::decode_path_key(&d.path_key)
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default(),
            path_key: d.path_key,
            content_sha256: d.content_sha256,
            bytes: d.bytes,
        })
        .collect();
    let graph = structure::build(&docs, c.generation_id().to_owned(), cancel)?;
    let previous = previous.and_then(|p| load(p).ok());
    let mut segments = vec![];
    let mut skipped = vec![];
    let mut reused = 0;
    let mut encoder = model_dir
        .map(|p| semantic::Encoder::load(p, cancel))
        .transpose()?;
    let identity = encoder.as_ref().map(|e| e.model_identity().clone());
    if let Some(encoder) = encoder.as_mut() {
        for doc in &docs {
            check(cancel)?;
            if std::str::from_utf8(&doc.bytes).is_err() || doc.bytes.contains(&0) {
                skipped.push(doc.path_key.clone());
                continue;
            }
            let prior = previous
                .as_ref()
                .filter(|p| p.model == identity && p.graph.producer == graph.producer)
                .map(|p| {
                    p.segments
                        .iter()
                        .filter(|s| {
                            s.path_key == doc.path_key && s.content_sha256 == doc.content_sha256
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if !prior.is_empty() {
                segments.extend(prior);
                reused += 1;
                continue;
            }
            let mut boundaries = vec![0, doc.bytes.len()];
            for node in &graph.nodes {
                if node.path_key == doc.path_key && node.kind != "file" && node.scope.is_empty() {
                    boundaries.extend([node.start_byte, node.end_byte]);
                }
            }
            boundaries.sort_unstable();
            boundaries.dedup();
            segments.extend(encoder.embed_document_ranges(
                semantic::Document {
                    path_key: &doc.path_key,
                    content_sha256: &doc.content_sha256,
                    bytes: &doc.bytes,
                },
                &boundaries,
                cancel,
            )?);
        }
    }
    let d = Derived {
        version: VERSION.into(),
        generation: c.generation_id().into(),
        graph,
        model: identity,
        model_dir: model_dir
            .map(|p| p.canonicalize().map_err(failure))
            .transpose()?,
        segments,
        semantic_skipped: skipped,
        reused_semantic_files: reused,
    };
    let data = serde_json::to_value(&d).map_err(failure)?;
    let bytes =
        codec::canonical(&json!({"sha256":model::digest(&codec::canonical(&data)?),"data":data}))?;
    if bytes.len() as u64 > MAX_DERIVED {
        return Err(Error::invalid(
            "derived index exceeds 128MiB per generation",
        ));
    }
    c.with_writer(cancel, || {
        c.reserve_storage(bytes.len() as u64)?;
        let mut temp =
            tempfile::NamedTempFile::new_in(c.pinned_generation_dir()).map_err(failure)?;
        temp.write_all(&bytes).map_err(failure)?;
        temp.as_file().sync_all().map_err(failure)?;
        check(cancel)?;
        temp.persist_noclobber(c.pinned_generation_dir().join("derived.json"))
            .map_err(failure)?;
        fs::File::open(c.pinned_generation_dir())
            .and_then(|f| f.sync_all())
            .map_err(failure)?;
        Ok(())
    })?;
    Ok(
        json!({"structure":{ "nodes":d.graph.nodes.len(),"relations":d.graph.edges.len(),"coverage":d.graph.coverage },"semantic":{"enabled":d.model.is_some(),"model":d.model,"segments":d.segments.len(),"skipped_files":d.semantic_skipped,"reused_files":reused},"derived_bytes":bytes.len()}),
    )
}
pub fn prepare(
    request: &PrepareRequest,
    model_dir: Option<&Path>,
    cancel: &AtomicBool,
) -> Result<Value> {
    let previous = Collection::open(&request.store, None).ok();
    let mut receipt = collection::prepare_staged(request, cancel)?;
    let generation = receipt["generation"]
        .as_str()
        .ok_or_else(|| failure("missing generation"))?
        .to_owned();
    let c = Collection::open(&request.store, Some(&generation))?;
    receipt["derived"] = derive(&c, previous.as_ref(), model_dir, cancel)?;
    collection::publish_current_expected(
        &request.store,
        &generation,
        previous.as_ref().map(Collection::generation_id),
        cancel,
    )?;
    Ok(receipt)
}
pub fn update(store: &Path, cancel: &AtomicBool) -> Result<Value> {
    let previous = Collection::open(store, None)?;
    let d = load(&previous)?;
    let mut receipt = collection::update_staged(store, cancel)?;
    let generation = receipt["generation"].as_str().unwrap().to_owned();
    let c = Collection::open(store, Some(&generation))?;
    receipt["derived"] = derive(&c, Some(&previous), d.model_dir.as_deref(), cancel)?;
    collection::publish_current_expected(
        store,
        &generation,
        Some(previous.generation_id()),
        cancel,
    )?;
    Ok(receipt)
}
pub fn status(store: &Path) -> Result<Value> {
    let mut v = collection::status(store)?;
    let c = Collection::open(store, None)?;
    match load(&c) {
        Ok(d) => {
            v["derived"] = json!({"status":"available","producer":d.version,"graph_nodes":d.graph.nodes.len(),"graph_producer":d.graph.producer,"graph_relations":d.graph.edges.len(),"structure_coverage":d.graph.coverage,"semantic_model":d.model,"semantic_segments":d.segments.len(),"semantic_skipped":d.semantic_skipped,"semantic_reused_files":d.reused_semantic_files});
        }
        Err(e) => v["derived"] = json!({"status":"unavailable","reason":e.message}),
    };
    Ok(v)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Candidate {
    path_key: String,
    content_sha256: String,
    start_line: u64,
    end_line: u64,
    start_byte: u64,
    end_byte: u64,
    score: f64,
    methods: Vec<String>,
    expanded: bool,
    node_id: Option<String>,
}
fn number(v: &Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}
fn candidate(v: &Value, method: &str) -> Result<Candidate> {
    Ok(Candidate {
        path_key: v["path_key"]
            .as_str()
            .ok_or_else(|| failure("invalid candidate path"))?
            .into(),
        content_sha256: v["content_sha256"]
            .as_str()
            .ok_or_else(|| failure("invalid candidate revision"))?
            .into(),
        start_line: number(&v["start_line"]).max(number(&v["line"])).max(1),
        end_line: number(&v["end_line"]).max(number(&v["line"])).max(1),
        start_byte: number(&v["start_byte"]),
        end_byte: number(&v["end_byte"]),
        score: 0.0,
        methods: vec![method.into()],
        expanded: false,
        node_id: None,
    })
}
struct VerifiedSpans {
    content_sha256: String,
    spans: Vec<(u64, u64, u64, u64)>,
}
type SpanCache = BTreeMap<String, VerifiedSpans>;
fn canonicalize(
    c: &Collection,
    mut x: Candidate,
    cache: &mut SpanCache,
    cancel: &AtomicBool,
) -> Result<Candidate> {
    if !cache.contains_key(&x.path_key) {
        let d = c.verified_document(&x.path_key, cancel)?;
        let mut spans = Vec::new();
        let (mut first, mut last, mut start, mut offset) = (1u64, 0u64, 0u64, 0u64);
        for line in d.bytes.split_inclusive(|b| *b == b'\n') {
            last += 1;
            offset += line.len() as u64;
            if last % 80 == 0 {
                check(cancel)?;
                spans.push((first, last, start, offset));
                first = last + 1;
                start = offset;
            }
        }
        if start < offset {
            spans.push((first, last, start, offset));
        }
        if d.bytes.is_empty() {
            spans.push((1, 1, 0, 0));
        }
        cache.insert(
            x.path_key.clone(),
            VerifiedSpans {
                content_sha256: d.content_sha256,
                spans,
            },
        );
    }
    let verified = &cache[&x.path_key];
    if verified.content_sha256 != x.content_sha256 {
        return Err(failure("candidate revision mismatch"));
    }
    let first = (x.start_line.saturating_sub(1) / 80) * 80 + 1;
    let &(start_line, end_line, start_byte, end_byte) = verified
        .spans
        .iter()
        .find(|span| span.0 == first)
        .ok_or_else(|| failure("candidate line outside verified object"))?;
    x.start_line = start_line;
    x.end_line = end_line;
    x.start_byte = start_byte;
    x.end_byte = end_byte;
    Ok(x)
}
fn ranks(
    c: &Collection,
    d: &Derived,
    query: &str,
    mode: &str,
    cancel: &AtomicBool,
) -> Result<(Vec<Candidate>, Value)> {
    if query.trim().is_empty() || query.len() > 4096 {
        return Err(Error::invalid("query requires 1..4096 UTF8 bytes"));
    }
    if !["literal", "lexical", "semantic", "hybrid"].contains(&mode) {
        return Err(Error::invalid(
            "mode must be literal, lexical, semantic or hybrid",
        ));
    }
    let mut channels = vec![];
    let mut coverage = json!({"candidate_limit_per_channel":CANDIDATES,"semantic":"not_requested"});
    let literal_unsupported = if query.len() > 256 {
        Some("query_exceeds_256_bytes")
    } else if query.bytes().any(|b| matches!(b, 0 | b'\r' | b'\n')) {
        Some("query_contains_nul_or_line_break")
    } else {
        None
    };
    if mode == "hybrid" && literal_unsupported.is_some() {
        coverage["literal"] = json!({"status":"unsupported","reason":literal_unsupported.unwrap()});
    }
    if mode == "literal" || (mode == "hybrid" && literal_unsupported.is_none()) {
        let o = crate::search::search(
            &crate::search::Request {
                snapshot: c.snapshot_path().to_owned(),
                source: c.source_path().to_owned(),
                query: query.into(),
                max_matches: CANDIDATES as u64,
                limits: crate::search::Limits::default(),
            },
            cancel,
        )
        .map_err(|e| Error::invalid(e.to_string()))?;
        if o.value["snapshot"]["snapshot_id"] != c.generation_id()
            || o.value["snapshot"]["snapshot_file_sha256"]
                != model::digest(&codec::canonical(&c.snapshot)?)
        {
            return Err(failure(
                "literal source does not match pinned collection generation",
            ));
        }
        coverage["literal"] = o.value["scan"].clone();
        channels.push(
            o.value["matches"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| candidate(v, "literal"))
                .collect::<Result<Vec<_>>>()?,
        );
    }
    if mode == "lexical" || mode == "hybrid" {
        let o = c.query_cancel(query, CANDIDATES, 0, cancel)?;
        coverage["lexical"] = json!({"candidate_count":o["results"].as_array().unwrap().len(),"potentially_truncated":o["results"].as_array().unwrap().len()==CANDIDATES});
        channels.push(
            o["results"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| candidate(v, "lexical"))
                .collect::<Result<Vec<_>>>()?,
        );
    }
    if mode == "semantic" || (mode == "hybrid" && d.model.is_some()) {
        let model_dir = d.model_dir.as_ref().ok_or_else(|| {
            Error::invalid("semantic index unavailable; prepare with --semantic-model")
        })?;
        if d.model.as_ref() != Some(&semantic::ModelIdentity::pinned()) {
            return Err(failure("semantic producer mismatch"));
        }
        let mut encoder = semantic::Encoder::load(model_dir, cancel)?;
        let q = encoder.embed_query(query, cancel)?;
        let mut scored = vec![];
        for block in d.segments.chunks(64) {
            check(cancel)?;
            for s in block {
                if s.model_fingerprint != encoder.model_identity().fingerprint {
                    return Err(failure("mixed semantic model"));
                }
                scored.push((semantic::cosine(&q, &s.vector)?, s));
            }
        }
        scored.sort_by(|(a, x), (b, y)| {
            b.total_cmp(a)
                .then(x.path_key.cmp(&y.path_key))
                .then(x.start_byte.cmp(&y.start_byte))
        });
        channels.push(
            scored
                .iter()
                .take(CANDIDATES)
                .map(|(_, s)| Candidate {
                    path_key: s.path_key.clone(),
                    content_sha256: s.content_sha256.clone(),
                    start_line: s.start_line,
                    end_line: s.end_line,
                    start_byte: s.start_byte,
                    end_byte: s.end_byte,
                    score: 0.0,
                    methods: vec!["semantic".into()],
                    expanded: false,
                    node_id: None,
                })
                .collect(),
        );
        coverage["semantic"] = json!({"status":"available","segments":d.segments.len(),"skipped_files":d.semantic_skipped,"potentially_truncated":d.segments.len()>CANDIDATES});
    } else if mode == "hybrid" {
        coverage["semantic"] =
            json!({"status":"unavailable","reason":"not_enabled_for_collection"});
    }
    let mut merged: BTreeMap<(String, u64), Candidate> = BTreeMap::new();
    let mut span_cache = SpanCache::new();
    for channel in channels {
        let mut seen = BTreeSet::new();
        for x in channel {
            check(cancel)?;
            let mut x = canonicalize(c, x, &mut span_cache, cancel)?;
            let key = (x.path_key.clone(), x.start_line);
            if !seen.insert(key.clone()) {
                continue;
            }
            let rank = seen.len();
            let score = 1.0 / (60.0 + rank as f64);
            if let Some(existing) = merged.get_mut(&key) {
                existing.score += score;
                existing.methods.extend(x.methods);
            } else {
                x.score = score;
                merged.insert(key, x);
            }
        }
    }
    coverage["canonicalization"] =
        json!({"verified_files":span_cache.len(),"cache":"per_query_verified_spans"});
    let mut out: Vec<_> = merged.into_values().collect();
    out.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then(a.path_key.cmp(&b.path_key))
            .then(a.start_byte.cmp(&b.start_byte))
    });
    Ok((out, coverage))
}
/// One generation-bound discovery pass for the experimental acquisition policy.
pub(crate) fn acquisition_discover(
    c: &Collection,
    query: &str,
    mode: &str,
    cancel: &AtomicBool,
) -> Result<(structure::Graph, Value, Value)> {
    let d = load(c)?;
    let (ranked, coverage) = ranks(c, &d, query, mode, cancel)?;
    Ok((
        d.graph,
        serde_json::to_value(ranked).map_err(failure)?,
        coverage,
    ))
}
fn excerpt(c: &Collection, x: &Candidate, cancel: &AtomicBool) -> Result<Value> {
    let d = c.verified_document(&x.path_key, cancel)?;
    if d.content_sha256 != x.content_sha256 {
        return Err(failure("selected revision mismatch"));
    }
    let raw = d
        .bytes
        .get(x.start_byte as usize..x.end_byte as usize)
        .ok_or_else(|| failure("invalid selected span"))?;
    let text = std::str::from_utf8(raw).map_err(|_| failure("selected span is not UTF8"))?;
    let mut value = serde_json::to_value(x).map_err(failure)?;
    value["text"] = json!(text);
    value["verification"] = json!("full_object_sha256");
    Ok(value)
}
pub fn query(
    c: &Collection,
    query: &str,
    mode: &str,
    limit: usize,
    offset: usize,
    cancel: &AtomicBool,
) -> Result<Value> {
    if limit == 0 || limit > 1000 {
        return Err(Error::invalid("limit must be 1..1000"));
    }
    let d = load(c)?;
    let (ranked, coverage) = ranks(c, &d, query, mode, cancel)?;
    let results = ranked
        .iter()
        .skip(offset)
        .take(limit)
        .map(|x| excerpt(c, x, cancel))
        .collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"generation":c.generation_id(),"producer":VERSION,"mode":mode,"coverage":coverage,"results":results,"total_candidates":ranked.len(),"offset":offset,"next_offset":(offset.saturating_add(limit)<ranked.len()).then_some(offset.saturating_add(limit))}),
    )
}
pub fn neighbors(
    c: &Collection,
    node: Option<&str>,
    query: Option<&str>,
    depth: usize,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Value> {
    let d = load(c)?;
    let mut v = if let Some(id) = node {
        serde_json::to_value(d.graph.neighbors(id, depth, limit, cancel)?).map_err(failure)?
    } else {
        if limit == 0 || limit > 1000 {
            return Err(Error::invalid("limit must be 1..1000"));
        }
        let q = query.unwrap_or("").to_lowercase();
        let found: Vec<_> = d
            .graph
            .nodes
            .iter()
            .filter(|n| n.name.to_lowercase().contains(&q) || n.path.to_lowercase().contains(&q))
            .take(limit + 1)
            .collect();
        json!({"generation":c.generation_id(),"nodes":found.iter().take(limit).collect::<Vec<_>>(),"truncated":found.len()>limit,"edges":[]})
    };
    let mut checked = BTreeSet::new();
    for n in v["nodes"].as_array().into_iter().flatten() {
        let key = n["path_key"]
            .as_str()
            .ok_or_else(|| failure("invalid graph evidence"))?;
        if checked.insert(key.to_owned()) {
            let doc = c.verified_document(key, cancel)?;
            if n["content_sha256"] != doc.content_sha256 {
                return Err(failure("graph evidence mismatch"));
            }
        }
    }
    v["coverage"] = serde_json::to_value(&d.graph.coverage).map_err(failure)?;
    v["graph_producer"] = json!(d.graph.producer);
    Ok(v)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrieveRequest {
    pub query: String,
    pub mode: String,
    pub expansion: String,
    pub theta: f64,
    pub budget_bytes: usize,
    pub cursor: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Cursor {
    generation: String,
    request_hash: String,
    offset: usize,
    #[serde(default)]
    next_start_line: Option<u64>,
    #[serde(default)]
    expanded_offset: usize,
    #[serde(default)]
    expanded_next_start_line: Option<u64>,
}
#[derive(Clone, Copy, Default)]
struct QueueState {
    offset: usize,
    line: Option<u64>,
}
#[derive(Clone, Copy, Default)]
struct PageState {
    direct: QueueState,
    expanded: QueueState,
}
fn request_hash(r: &RetrieveRequest) -> Result<String> {
    Ok(model::digest(&codec::canonical(&json!([
        VERSION,
        "pack-v2-prefix-reserved-expansion",
        r.query,
        r.mode,
        r.expansion,
        r.theta,
        r.budget_bytes
    ]))?))
}
fn cursor(c: &Collection, hash: &str, state: PageState) -> Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(codec::canonical(&Cursor {
        generation: c.generation_id().into(),
        request_hash: hash.into(),
        offset: state.direct.offset,
        next_start_line: state.direct.line,
        expanded_offset: state.expanded.offset,
        expanded_next_start_line: state.expanded.line,
    })?))
}
fn page_cursor(
    c: &Collection,
    hash: &str,
    state: PageState,
    totals: (usize, usize),
) -> Result<Value> {
    if state.direct.offset < totals.0 || state.expanded.offset < totals.1 {
        Ok(json!(cursor(c, hash, state)?))
    } else {
        Ok(Value::Null)
    }
}
pub fn cursor_generation(raw: &str) -> Result<String> {
    if raw.len() > 4096 {
        return Err(Error::invalid("cursor too large"));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(raw)
        .map_err(|_| Error::invalid("invalid cursor"))?;
    let c: Cursor = serde_json::from_slice(&bytes).map_err(|_| Error::invalid("invalid cursor"))?;
    Ok(c.generation)
}
// Prefixes end at complete original line boundaries. CRLF, BOM and Unicode
// remain byte-identical; the remainder is both a read reference and a cursor.
fn prefix_item(full: &Value, lines: usize) -> Result<Value> {
    let text = full["text"]
        .as_str()
        .ok_or_else(|| failure("missing excerpt text"))?;
    let lengths: Vec<_> = text.split_inclusive('\n').map(str::len).collect();
    if lines == 0 || lines > lengths.len() {
        return Err(failure("invalid excerpt prefix"));
    }
    let bytes: usize = lengths[..lines].iter().sum();
    let mut item = full.clone();
    item["text"] = json!(&text[..bytes]);
    item["end_byte"] = json!(number(&full["start_byte"]) + bytes as u64);
    item["end_line"] = json!(number(&full["start_line"]) + lines as u64 - 1);
    let truncated = bytes < text.len();
    item["truncated"] = json!(truncated);
    item["next_start_line"] = if truncated {
        json!(number(&item["end_line"]) + 1)
    } else {
        Value::Null
    };
    if truncated {
        item["remainder"] = json!({"path_key":full["path_key"],"content_sha256":full["content_sha256"],"start_line":number(&item["end_line"])+1,"end_line":full["end_line"],"start_byte":item["end_byte"],"end_byte":full["end_byte"]});
    }
    Ok(item)
}
fn resume_item(mut full: Value, line: Option<u64>) -> Result<Value> {
    if let Some(line) = line {
        let first = number(&full["start_line"]);
        let last = number(&full["end_line"]);
        if line < first || line > last {
            return Err(Error::invalid("cursor line outside candidate"));
        }
        let text = full["text"]
            .as_str()
            .ok_or_else(|| failure("missing excerpt text"))?
            .to_owned();
        let skip: usize = text
            .split_inclusive('\n')
            .take((line - first) as usize)
            .map(str::len)
            .sum();
        full["start_line"] = json!(line);
        full["start_byte"] = json!(number(&full["start_byte"]) + skip as u64);
        full["text"] = json!(&text[skip..]);
    }
    Ok(full)
}
struct PageBudget<'a> {
    collection: &'a Collection,
    hash: &'a str,
    totals: (usize, usize),
    budget: usize,
    cancel: &'a AtomicBool,
}
fn fill_group(
    page: &PageBudget<'_>,
    queue: &[Candidate],
    expanded: bool,
    state: &mut PageState,
    pack: &mut Value,
    quota: usize,
    returned: &mut usize,
) -> Result<usize> {
    let mut used = 0usize;
    while *returned < 20 {
        check(page.cancel)?;
        let current = if expanded {
            state.expanded
        } else {
            state.direct
        };
        if current.offset >= queue.len() {
            break;
        }
        let candidate = &queue[current.offset];
        let full = resume_item(
            excerpt(page.collection, candidate, page.cancel)?,
            current.line,
        )?;
        let lines = full["text"].as_str().unwrap().split_inclusive('\n').count();
        let fit = |item: Value,
                   progress: QueueState,
                   omitted: bool|
         -> Result<Option<(Value, PageState, usize)>> {
            let cost = codec::canonical(&item)?.len();
            if used.saturating_add(cost) > quota {
                return Ok(None);
            }
            let mut next = *state;
            if expanded {
                next.expanded = progress;
            } else {
                next.direct = progress;
            }
            let mut trial = pack.clone();
            trial[if omitted { "omitted" } else { "items" }]
                .as_array_mut()
                .unwrap()
                .push(item);
            trial["next_cursor"] = page_cursor(page.collection, page.hash, next, page.totals)?;
            trial["payload_bytes"] = json!(page.budget);
            Ok((codec::canonical(&trial)?.len() <= page.budget).then_some((trial, next, cost)))
        };
        let mut whole = full.clone();
        whole["truncated"] = json!(false);
        whole["next_start_line"] = Value::Null;
        if let Some((trial, next, cost)) = fit(
            whole,
            QueueState {
                offset: current.offset + 1,
                line: None,
            },
            false,
        )? {
            *pack = trial;
            *state = next;
            used += cost;
            *returned += 1;
            continue;
        }
        let (mut low, mut high, mut best) = (1, lines.saturating_sub(1), None);
        while low <= high {
            check(page.cancel)?;
            let middle = low + (high - low) / 2;
            let item = prefix_item(&full, middle)?;
            let line = number(&item["next_start_line"]);
            if let Some(result) = fit(
                item,
                QueueState {
                    offset: current.offset,
                    line: Some(line),
                },
                false,
            )? {
                best = Some(result);
                low = middle + 1;
            } else {
                high = middle - 1;
            }
        }
        if let Some((trial, next, cost)) = best {
            *pack = trial;
            *state = next;
            used += cost;
            *returned += 1;
            break;
        }
        // Defer to the next page when this group already used its allocation.
        if used > 0
            || !pack["items"].as_array().unwrap().is_empty()
            || !pack["omitted"].as_array().unwrap().is_empty()
        {
            break;
        }
        // An allocation constraint is not evidence that a line cannot fit a
        // fresh page. Keep its cursor until the other queue releases room.
        let mut empty = pack.clone();
        empty["items"] = json!([]);
        empty["omitted"] = json!([]);
        let mut single = full.clone();
        single["text"] = json!(
            full["text"]
                .as_str()
                .unwrap()
                .split_inclusive('\n')
                .next()
                .unwrap_or("")
        );
        single["truncated"] = json!(true);
        single["next_start_line"] = json!(number(&full["start_line"]) + 1);
        empty["items"].as_array_mut().unwrap().push(single);
        empty["payload_bytes"] = json!(page.budget);
        if codec::canonical(&empty)?.len() <= page.budget {
            break;
        }
        let mut reference = full;
        reference.as_object_mut().unwrap().remove("text");
        reference.as_object_mut().unwrap().remove("verification");
        reference["reason"] = json!("budget");
        reference["detail"] = json!("budget_cannot_fit_complete_line");
        if let Some((trial, next, cost)) = fit(
            reference,
            QueueState {
                offset: current.offset + 1,
                line: None,
            },
            true,
        )? {
            *pack = trial;
            *state = next;
            used += cost;
            *returned += 1;
            continue;
        }
        break;
    }
    Ok(used)
}
pub fn retrieve(c: &Collection, r: &RetrieveRequest, cancel: &AtomicBool) -> Result<Value> {
    if !(2048..=1048576).contains(&r.budget_bytes)
        || !["none", "bfs", "heat"].contains(&r.expansion.as_str())
        || !r.theta.is_finite()
        || !(0.0..=10.0).contains(&r.theta)
    {
        return Err(Error::invalid("invalid retrieval budget/expansion/theta"));
    }
    let hash = request_hash(r)?;
    let mut page_state = if let Some(raw) = &r.cursor {
        if raw.len() > 4096 {
            return Err(Error::invalid("cursor too large"));
        }
        let x: Cursor = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(raw).map_err(failure)?)
            .map_err(failure)?;
        if x.generation != c.generation_id() || x.request_hash != hash {
            return Err(Error::invalid(
                "cursor belongs to another generation/request",
            ));
        }
        PageState {
            direct: QueueState {
                offset: x.offset,
                line: x.next_start_line,
            },
            expanded: QueueState {
                offset: x.expanded_offset,
                line: x.expanded_next_start_line,
            },
        }
    } else {
        PageState::default()
    };
    let d = load(c)?;
    let (mut direct, coverage) = ranks(c, &d, &r.query, &r.mode, cancel)?;
    let mut expansion = json!({"method":r.expansion});
    let mut expanded = vec![];
    let mut span_cache = SpanCache::new();
    if r.expansion != "none" && !direct.is_empty() {
        let projection = diffusion::Graph::new(
            d.graph.nodes.iter().map(|n| n.id.clone()).collect(),
            &d.graph.projection_pairs(),
        )?;
        let mut seeds: BTreeMap<String, f64> = BTreeMap::new();
        for hit in &mut direct {
            let node = d
                .graph
                .nodes
                .iter()
                .filter(|n| {
                    n.path_key == hit.path_key
                        && n.start_byte as u64 <= hit.start_byte
                        && n.end_byte as u64 >= hit.end_byte
                })
                .min_by_key(|n| n.end_byte - n.start_byte)
                .or_else(|| {
                    d.graph
                        .nodes
                        .iter()
                        .find(|n| n.path_key == hit.path_key && n.kind == "file")
                });
            if let Some(n) = node {
                hit.node_id = Some(n.id.clone());
                *seeds.entry(n.id.clone()).or_default() += hit.score;
            }
        }
        let scores: Vec<(usize, f64)> = if r.expansion == "heat" {
            let h = projection.heat(
                &seeds
                    .iter()
                    .map(|(id, s)| (id.clone(), *s))
                    .collect::<Vec<_>>(),
                r.theta,
                20_000_000,
                cancel,
            )?;
            expansion = json!({"method":"heat","projection":"resolved_contains_imports_calls_unit_v1","status":h.status,"theta":h.theta,"max_degree":h.max_degree,"iterations":h.iterations,"edge_visits":h.edge_visits,"truncation_tail_bound":h.truncation_tail_bound,"floating_point_error":h.floating_point_error});
            h.scores
                .into_iter()
                .enumerate()
                .filter(|(_, s)| *s > 0.0)
                .collect()
        } else {
            let b = projection.bfs(
                &seeds.keys().cloned().collect::<Vec<_>>(),
                2,
                1000,
                100000,
                cancel,
            )?;
            expansion =
                json!({"method":"bfs","complete":b.complete,"depth":2,"edge_visits":b.edge_visits});
            b.nodes
                .into_iter()
                .map(|(i, depth)| (i, 1.0 / (1.0 + depth as f64)))
                .collect()
        };
        let mut seen: BTreeSet<_> = direct
            .iter()
            .map(|x| (x.path_key.clone(), x.start_line))
            .collect();
        let mut scores = scores;
        scores.sort_by(|(i, a), (j, b)| b.total_cmp(a).then(i.cmp(j)));
        for (i, score) in scores {
            if expanded.len() >= CANDIDATES {
                expansion["candidates_truncated"] = json!(true);
                break;
            }
            let id = &projection.ids[i];
            if seeds.contains_key(id) {
                continue;
            }
            let node = d
                .graph
                .nodes
                .iter()
                .find(|n| &n.id == id)
                .ok_or_else(|| failure("projection node absent"))?;
            let mut x = canonicalize(
                c,
                Candidate {
                    path_key: node.path_key.clone(),
                    content_sha256: node.content_sha256.clone(),
                    start_line: node.start_line as u64,
                    end_line: node.end_line as u64,
                    start_byte: node.start_byte as u64,
                    end_byte: node.end_byte as u64,
                    score,
                    methods: vec![r.expansion.clone()],
                    expanded: true,
                    node_id: Some(id.clone()),
                },
                &mut span_cache,
                cancel,
            )?;
            if seen.insert((x.path_key.clone(), x.start_line)) {
                x.expanded = true;
                expanded.push(x);
            }
        }
    }
    let totals = (direct.len(), expanded.len());
    for (state, total) in [
        (page_state.direct, totals.0),
        (page_state.expanded, totals.1),
    ] {
        if state.offset > total || (state.offset == total && state.line.is_some()) {
            return Err(Error::invalid("cursor offset outside candidate set"));
        }
    }
    let initial = page_state;
    let mut pack = json!({"schema_version":"1","report_type":"evidence_pack","generation":c.generation_id(),"producer":VERSION,"graph_producer":d.graph.producer,"query":r.query,"mode":r.mode,"coverage":coverage,"expansion":expansion,"budget_bytes":r.budget_bytes,"payload_bytes":r.budget_bytes,"items":[],"omitted":[],"total_candidates":totals.0+totals.1,"direct_candidates":totals.0,"next_cursor":page_cursor(c,&hash,page_state,totals)?});
    let header = codec::canonical(&pack)?.len();
    if header > r.budget_bytes {
        return Err(Error::invalid("budget cannot fit response metadata"));
    }
    let available = r.budget_bytes - header;
    let has_expansion = page_state.expanded.offset < totals.1;
    let page = PageBudget {
        collection: c,
        hash: &hash,
        totals,
        budget: r.budget_bytes,
        cancel,
    };
    let mut returned = 0;
    // Reserve a quarter of the initial payload room for expansion. Direct
    // evidence goes first; either group can reclaim the other's unused room
    // after that group is exhausted. The exact JSON envelope is always fitted.
    let direct_quota = if has_expansion {
        available * 3 / 4
    } else {
        available
    };
    fill_group(
        &page,
        &direct,
        false,
        &mut page_state,
        &mut pack,
        direct_quota,
        &mut returned,
    )?;
    let direct_pending = page_state.direct.offset < totals.0;
    let expansion_quota = if direct_pending {
        available / 4
    } else {
        r.budget_bytes
    };
    fill_group(
        &page,
        &expanded,
        true,
        &mut page_state,
        &mut pack,
        expansion_quota,
        &mut returned,
    )?;
    if page_state.expanded.offset >= totals.1 && page_state.direct.offset < totals.0 {
        fill_group(
            &page,
            &direct,
            false,
            &mut page_state,
            &mut pack,
            r.budget_bytes,
            &mut returned,
        )?;
    }
    pack["next_cursor"] = page_cursor(c, &hash, page_state, totals)?;
    if returned == 0 && (initial.direct.offset < totals.0 || initial.expanded.offset < totals.1) {
        return Err(Error::invalid(
            "budget cannot fit candidate metadata; increase budget_bytes",
        ));
    }
    for _ in 0..4 {
        let len = codec::canonical(&pack)?.len();
        if pack["payload_bytes"] == len {
            break;
        }
        pack["payload_bytes"] = json!(len);
    }
    if codec::canonical(&pack)?.len() > r.budget_bytes {
        return Err(Error::invalid("budget cannot fit response metadata"));
    }
    check(cancel)?;
    Ok(pack)
}
