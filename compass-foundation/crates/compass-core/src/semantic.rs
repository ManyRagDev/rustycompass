//! Pinned, local CPU E5 encoder. Only `install` is permitted to use the network.
use crate::error::{Error, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokenizers::Tokenizer;

pub const MODEL_ID: &str = "intfloat/multilingual-e5-small";
pub const MODEL_REVISION: &str = "614241f622f53c4eeff9890bdc4f31cfecc418b3";
pub const DIMENSIONS: usize = 384;
pub const MAX_TOKENS: usize = 512;
pub const SEGMENTER_ID: &str = "syntax-boundaries-or-utf8-contiguous-halving-16384-e5passage512-v1";
const FILES: &[(&str, u64, &str)] = &[
    (
        "config.json",
        655,
        "69137736cab8b8903a07fe8afaafdda25aac55415a12a55d1bffa9f581abf959",
    ),
    (
        "tokenizer.json",
        17082730,
        "0b44a9d7b51c3c62626640cda0e2c2f70fdacdc25bbbd68038369d14ebdf4c39",
    ),
    (
        "model.safetensors",
        470641600,
        "1a55775f53449dac10a2bcbc312469fac40b96d53198c407081a831f81c98477",
    ),
    (
        "tokenizer_config.json",
        443,
        "a1d6bc8734a6f635dc158508bef000f8e2e5a759c7d92f984b2c86e5ff53425b",
    ),
    (
        "special_tokens_map.json",
        167,
        "d05497f1da52c5e09554c0cd874037a083e1dc1b9cfd48034d1c717f1afc07a7",
    ),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelIdentity {
    pub model_id: String,
    pub revision: String,
    pub fingerprint: String,
    pub dimensions: usize,
    pub max_tokens: usize,
    pub segmenter_id: String,
    pub backend: String,
}
impl ModelIdentity {
    pub fn pinned() -> Self {
        let mut h = Sha256::new();
        h.update(format!("{MODEL_ID}\n{MODEL_REVISION}\n{SEGMENTER_ID}\nquery: \npassage: \nmasked-mean-l2-f32-candle-0.9.2-cpu\n"));
        for (name, size, hash) in FILES {
            h.update(format!("{name}:{size}:{hash}\n"));
        }
        Self {
            model_id: MODEL_ID.into(),
            revision: MODEL_REVISION.into(),
            fingerprint: format!("{:x}", h.finalize()),
            dimensions: DIMENSIONS,
            max_tokens: MAX_TOKENS,
            segmenter_id: SEGMENTER_ID.into(),
            backend: "candle-0.9.2-cpu-f32".into(),
        }
    }
}

pub struct Document<'a> {
    pub path_key: &'a str,
    pub content_sha256: &'a str,
    pub bytes: &'a [u8],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub path_key: String,
    pub content_sha256: String,
    pub model_fingerprint: String,
    pub segmenter_id: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub start_line: u64,
    pub end_line: u64,
    pub vector: Vec<f32>,
}
fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::cancelled())
    } else {
        Ok(())
    }
}
fn failure(message: impl std::fmt::Display) -> Error {
    Error {
        code: "semantic_error",
        message: message.to_string(),
        exit_code: 1,
    }
}
fn sha(bytes: &[u8], cancel: &AtomicBool) -> Result<String> {
    let mut h = Sha256::new();
    for block in bytes.chunks(1024 * 1024) {
        check(cancel)?;
        h.update(block);
    }
    check(cancel)?;
    Ok(format!("{:x}", h.finalize()))
}
fn verified_bytes(
    dir: &Path,
    file: &str,
    size: u64,
    expected: &str,
    cancel: &AtomicBool,
) -> Result<Vec<u8>> {
    check(cancel)?;
    let path = dir.join(file);
    let mut f = fs::File::open(&path).map_err(|e| {
        Error::io(
            format!(
                "local model missing {}; run model-install explicitly",
                path.display()
            ),
            e,
        )
    })?;
    if f.metadata()
        .map_err(|e| Error::io("model metadata", e))?
        .len()
        != size
    {
        return Err(failure(format!("model integrity failure: {file} size")));
    }
    let mut bytes = Vec::with_capacity(size as usize);
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        check(cancel)?;
        let n = f
            .read(&mut buf)
            .map_err(|e| Error::io("read local model", e))?;
        if n == 0 {
            break;
        }
        if bytes.len() as u64 + n as u64 > size {
            return Err(failure("model changed while reading"));
        }
        bytes.extend_from_slice(&buf[..n]);
    }
    if bytes.len() as u64 != size || sha(&bytes, cancel)? != expected {
        return Err(failure(format!("model integrity failure: {file} SHA256")));
    }
    Ok(bytes)
}

/// Explicit network operation; downloads fixed public files, verifies, then atomically publishes each.
/// Existing valid files are reused. Cancellation/failure never promotes partial bytes.
pub fn install(dir: &Path, cancel: &AtomicBool) -> Result<ModelIdentity> {
    check(cancel)?;
    fs::create_dir_all(dir).map_err(|e| Error::io("create model directory", e))?;
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(900))
        .build()
        .map_err(failure)?;
    for &(name, size, expected) in FILES {
        check(cancel)?;
        match verified_bytes(dir, name, size, expected, cancel) {
            Ok(_) => continue,
            Err(e) if e.code == "cancelled" => return Err(e),
            Err(_) => {}
        }
        let url = format!("https://huggingface.co/{MODEL_ID}/resolve/{MODEL_REVISION}/{name}");
        let mut response = client
            .get(url)
            .send()
            .map_err(failure)?
            .error_for_status()
            .map_err(failure)?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)
            .map_err(|e| Error::io("model temporary file", e))?;
        let mut h = Sha256::new();
        let mut total = 0u64;
        let mut buf = vec![0u8; 1024 * 1024];
        loop {
            check(cancel)?;
            let n = response
                .read(&mut buf)
                .map_err(|e| Error::io("download public model", e))?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > size {
                return Err(failure(format!(
                    "model download exceeds pinned size: {name}"
                )));
            }
            h.update(&buf[..n]);
            tmp.write_all(&buf[..n])
                .map_err(|e| Error::io("write model", e))?;
        }
        if total != size || format!("{:x}", h.finalize()) != expected {
            return Err(failure(format!("download integrity failure: {name}")));
        }
        check(cancel)?;
        tmp.as_file()
            .sync_all()
            .map_err(|e| Error::io("sync model", e))?;
        tmp.persist(dir.join(name))
            .map_err(|e| Error::io("publish model", e.error))?;
    }
    check(cancel)?;
    Ok(ModelIdentity::pinned())
}

pub struct Encoder {
    tokenizer: Tokenizer,
    model: BertModel,
    identity: ModelIdentity,
}
impl Encoder {
    /// Strictly offline; verifies the exact bytes passed to the tokenizer and weight loader.
    pub fn load(dir: &Path, cancel: &AtomicBool) -> Result<Self> {
        let config_bytes = verified_bytes(dir, FILES[0].0, FILES[0].1, FILES[0].2, cancel)?;
        let tokenizer_bytes = verified_bytes(dir, FILES[1].0, FILES[1].1, FILES[1].2, cancel)?;
        let weights = verified_bytes(dir, FILES[2].0, FILES[2].1, FILES[2].2, cancel)?;
        let config: Config = serde_json::from_slice(&config_bytes).map_err(failure)?;
        let mut tokenizer = Tokenizer::from_bytes(&tokenizer_bytes).map_err(failure)?;
        tokenizer.with_truncation(None).map_err(failure)?;
        tokenizer.with_padding(None);
        check(cancel)?;
        let vb = VarBuilder::from_buffered_safetensors(weights, DType::F32, &Device::Cpu)
            .map_err(failure)?;
        let model = BertModel::load(vb, &config).map_err(failure)?;
        check(cancel)?;
        Ok(Self {
            tokenizer,
            model,
            identity: ModelIdentity::pinned(),
        })
    }
    pub fn model_identity(&self) -> &ModelIdentity {
        &self.identity
    }
    fn encode(&self, text: &str) -> Result<tokenizers::Encoding> {
        self.tokenizer.encode(text, true).map_err(failure)
    }
    fn embed_prefixed(&self, text: &str, cancel: &AtomicBool) -> Result<Vec<f32>> {
        check(cancel)?;
        let encoding = self.encode(text)?;
        if encoding.len() > MAX_TOKENS {
            return Err(Error::invalid(format!(
                "semantic input has {} tokens; maximum is {MAX_TOKENS}; no truncation performed",
                encoding.len()
            )));
        }
        let ids = Tensor::new(encoding.get_ids(), &Device::Cpu)
            .and_then(|x| x.unsqueeze(0))
            .map_err(failure)?;
        let types = Tensor::new(encoding.get_type_ids(), &Device::Cpu)
            .and_then(|x| x.unsqueeze(0))
            .map_err(failure)?;
        let mask = Tensor::new(encoding.get_attention_mask(), &Device::Cpu)
            .and_then(|x| x.unsqueeze(0))
            .map_err(failure)?;
        check(cancel)?;
        let hidden = self
            .model
            .forward(&ids, &types, Some(&mask))
            .map_err(failure)?;
        check(cancel)?;
        let mask = mask
            .to_dtype(DType::F32)
            .and_then(|x| x.unsqueeze(2))
            .map_err(failure)?;
        let sum = hidden
            .broadcast_mul(&mask)
            .and_then(|x| x.sum(1))
            .map_err(failure)?;
        let count = mask.sum(1).map_err(failure)?;
        let pooled = sum
            .broadcast_div(&count)
            .and_then(|x| x.squeeze(0))
            .and_then(|x| x.to_vec1::<f32>())
            .map_err(failure)?;
        normalize(pooled)
    }
    pub fn embed_query(&mut self, query: &str, cancel: &AtomicBool) -> Result<Vec<f32>> {
        check(cancel)?;
        if query.trim().is_empty() || query.len() > 32768 {
            return Err(Error::invalid(
                "semantic query must be nonempty and at most 32768 UTF8 bytes",
            ));
        }
        self.embed_prefixed(&format!("query: {query}"), cancel)
    }
    /// Contiguous original byte ranges, no overlap and no omitted or silently truncated tokens.
    pub fn segment_spans(&self, text: &str, cancel: &AtomicBool) -> Result<Vec<(usize, usize)>> {
        segment_spans(&self.tokenizer, text, cancel)
    }
    pub fn embed_document(
        &mut self,
        doc: Document<'_>,
        cancel: &AtomicBool,
    ) -> Result<Vec<Segment>> {
        let boundaries = if doc.bytes.is_empty() {
            vec![0]
        } else {
            vec![0, doc.bytes.len()]
        };
        self.embed_document_ranges(doc, &boundaries, cancel)
    }
    /// Caller-supplied syntax boundaries partition the whole original document. Each unit
    /// is split further only when needed for the token/byte budget. Hash identity stays
    /// that of the whole document; callers must include the boundary producer in cache keys.
    pub fn embed_document_ranges(
        &mut self,
        doc: Document<'_>,
        boundaries: &[usize],
        cancel: &AtomicBool,
    ) -> Result<Vec<Segment>> {
        check(cancel)?;
        if doc.bytes.len() > 64 * 1024 * 1024 {
            return Err(Error::invalid("semantic document exceeds 64 MiB"));
        }
        if sha(doc.bytes, cancel)? != doc.content_sha256 {
            return Err(Error::corrupt(
                "semantic document SHA256 does not match bytes",
            ));
        }
        let text = std::str::from_utf8(doc.bytes)
            .map_err(|_| Error::invalid("semantic documents require UTF8"))?;
        if text.contains('\0') {
            return Err(Error::invalid("semantic document contains NUL"));
        }
        if boundaries.first() != Some(&0)
            || boundaries.last() != Some(&text.len())
            || boundaries.windows(2).any(|p| p[0] >= p[1])
            || boundaries.iter().any(|&p| !text.is_char_boundary(p))
        {
            return Err(Error::invalid(
                "semantic boundaries must strictly partition original UTF8 bytes from 0 to EOF",
            ));
        }
        let mut spans = Vec::new();
        for pair in boundaries.windows(2) {
            check(cancel)?;
            for (start, end) in self.segment_spans(&text[pair[0]..pair[1]], cancel)? {
                spans.push((pair[0] + start, pair[0] + end));
            }
        }
        let mut segments = Vec::new();
        let mut line = 1u64;
        for (start, end) in spans {
            check(cancel)?;
            let part = &text[start..end];
            let newlines = part.bytes().filter(|&b| b == b'\n').count() as u64;
            let end_line = line + newlines - u64::from(part.ends_with('\n'));
            let vector = self.embed_prefixed(&format!("passage: {part}"), cancel)?;
            segments.push(Segment {
                path_key: doc.path_key.into(),
                content_sha256: doc.content_sha256.into(),
                model_fingerprint: self.identity.fingerprint.clone(),
                segmenter_id: SEGMENTER_ID.into(),
                start_byte: start as u64,
                end_byte: end as u64,
                start_line: line,
                end_line,
                vector,
            });
            line += newlines;
        }
        check(cancel)?;
        Ok(segments)
    }
}
fn segment_spans(
    tokenizer: &Tokenizer,
    text: &str,
    cancel: &AtomicBool,
) -> Result<Vec<(usize, usize)>> {
    let mut spans = Vec::new();
    let mut start = 0;
    while start < text.len() {
        check(cancel)?;
        let mut end = (start + 16384).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        loop {
            check(cancel)?;
            let n = tokenizer
                .encode(format!("passage: {}", &text[start..end]), true)
                .map_err(failure)?
                .len();
            if n <= MAX_TOKENS {
                break;
            }
            let mut cut = start + (end - start) / 2;
            while cut > start && !text.is_char_boundary(cut) {
                cut -= 1;
            }
            if cut == start {
                return Err(failure(
                    "a single UTF8 character exceeds model token budget",
                ));
            }
            end = cut;
        }
        spans.push((start, end));
        start = end;
    }
    Ok(spans)
}
fn normalize(mut v: Vec<f32>) -> Result<Vec<f32>> {
    if v.len() != DIMENSIONS || v.iter().any(|x| !x.is_finite()) {
        return Err(failure("invalid embedding dimensions or values"));
    }
    let norm = v.iter().map(|&x| (x as f64).powi(2)).sum::<f64>().sqrt();
    if norm <= 0.0 || !norm.is_finite() {
        return Err(failure("invalid embedding norm"));
    }
    for x in &mut v {
        *x = (*x as f64 / norm) as f32;
    }
    Ok(v)
}
/// Exact cosine, bounded accumulation blocks; rejects malformed persisted vectors.
pub fn cosine(a: &[f32], b: &[f32]) -> Result<f32> {
    if a.len() != DIMENSIONS || b.len() != DIMENSIONS {
        return Err(Error::corrupt("semantic vector dimensions mismatch"));
    }
    let (mut dot, mut aa, mut bb) = (0f64, 0f64, 0f64);
    for (ac, bc) in a.chunks(64).zip(b.chunks(64)) {
        for (&x, &y) in ac.iter().zip(bc) {
            if !x.is_finite() || !y.is_finite() {
                return Err(Error::corrupt("nonfinite semantic vector"));
            }
            dot += x as f64 * y as f64;
            aa += (x as f64).powi(2);
            bb += (y as f64).powi(2);
        }
    }
    if aa == 0.0 || bb == 0.0 {
        return Err(Error::corrupt("zero semantic vector"));
    }
    Ok((dot / (aa * bb).sqrt()).clamp(-1., 1.) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_cosine_rejects_malformed() {
        let a = vec![1.; DIMENSIONS];
        let b = vec![-1.; DIMENSIONS];
        assert_eq!(cosine(&a, &a).unwrap(), 1.);
        assert_eq!(cosine(&a, &b).unwrap(), -1.);
        assert!(cosine(&a, &[]).is_err());
        assert!(cosine(&a, &vec![0.; DIMENSIONS]).is_err());
        let mut c = a.clone();
        c[3] = f32::NAN;
        assert!(cosine(&a, &c).is_err());
    }
    #[test]
    fn cancel_before_install_or_load() {
        let c = AtomicBool::new(true);
        let t = tempfile::tempdir().unwrap();
        assert_eq!(install(t.path(), &c).unwrap_err().code, "cancelled");
        assert!(matches!(Encoder::load(t.path(),&c),Err(e) if e.code=="cancelled"));
        assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
    }
    #[test]
    fn tampered_config_rejected_before_loading_weights() {
        let t = tempfile::tempdir().unwrap();
        fs::write(t.path().join("config.json"), vec![b' '; 655]).unwrap();
        assert!(
            matches!(Encoder::load(t.path(), &AtomicBool::new(false)), Err(e) if e.code == "semantic_error" && e.message.contains("SHA256"))
        );
    }
    #[test]
    fn pinned_identity_is_stable_and_explicit() {
        let id = ModelIdentity::pinned();
        assert_eq!(id.dimensions, 384);
        assert_eq!(id.fingerprint.len(), 64);
        assert_eq!(id.revision, MODEL_REVISION);
        assert_eq!(id.segmenter_id, SEGMENTER_ID);
    }
}

#[cfg(test)]
mod model_tests {
    use super::*;
    #[test]
    #[ignore = "requires explicitly installed pinned local model and COMPASS_SEMANTIC_MODEL"]
    fn independent_transformers_embeddings_and_full_span_coverage() {
        let dir = std::env::var_os("COMPASS_SEMANTIC_MODEL")
            .expect("set COMPASS_SEMANTIC_MODEL to installed model");
        let cancel = AtomicBool::new(false);
        let installed = install(Path::new(&dir), &cancel).unwrap();
        let mut encoder = Encoder::load(Path::new(&dir), &cancel).unwrap();
        assert_eq!(encoder.model_identity(), &installed);
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../verification/semantic/reference.json"
        ))
        .unwrap();
        let tolerance = fixture["absolute_tolerance"].as_f64().unwrap();
        let mut max_error = 0f64;
        for case in fixture["cases"].as_array().unwrap() {
            let text = case["text"].as_str().unwrap();
            let kind = case["kind"].as_str().unwrap();
            let prefixed = format!("{kind}: {text}");
            let encoded = encoder.encode(&prefixed).unwrap();
            let expected_ids: Vec<u32> = serde_json::from_value(case["input_ids"].clone()).unwrap();
            assert_eq!(encoded.get_ids(), expected_ids);
            let vector = encoder.embed_prefixed(&prefixed, &cancel).unwrap();
            let expected: Vec<f32> = serde_json::from_value(case["vector"].clone()).unwrap();
            for (got, want) in vector.iter().zip(&expected) {
                let delta = (*got as f64 - *want as f64).abs();
                max_error = max_error.max(delta);
                assert!(
                    delta <= tolerance,
                    "independent embedding differs: {kind} {delta}"
                );
            }
            assert!(cosine(&vector, &expected).unwrap() > 0.99999);
        }
        // A single very long line, non-ASCII and CRLF expose silent truncation and byte/char confusion.
        let text = format!(
            "\u{feff}início\r\n{}\nfinal sem LF",
            "café 東京 ☕ abc ".repeat(180)
        );
        let spans = encoder.segment_spans(&text, &cancel).unwrap();
        assert!(spans.len() > 1);
        let mut next = 0;
        for &(start, end) in &spans {
            assert_eq!(start, next);
            assert!(end > start);
            assert!(text.is_char_boundary(start) && text.is_char_boundary(end));
            assert!(
                encoder
                    .encode(&format!("passage: {}", &text[start..end]))
                    .unwrap()
                    .len()
                    <= MAX_TOKENS
            );
            next = end;
        }
        assert_eq!(next, text.len());
        let digest = sha(text.as_bytes(), &cancel).unwrap();
        let segments = encoder
            .embed_document(
                Document {
                    path_key: "YS50eHQ",
                    content_sha256: &digest,
                    bytes: text.as_bytes(),
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(segments.len(), spans.len());
        for (s, (start, end)) in segments.iter().zip(spans) {
            assert_eq!(s.start_byte, start as u64);
            assert_eq!(s.end_byte, end as u64);
            assert_eq!(
                s.start_line,
                1 + text[..start].bytes().filter(|&b| b == b'\n').count() as u64
            );
            assert_eq!(
                s.end_line,
                1 + text[..end - 1].bytes().filter(|&b| b == b'\n').count() as u64
            );
            assert_eq!(s.vector.len(), DIMENSIONS);
        }
        assert!(
            encoder
                .embed_query(&"palavra ".repeat(700), &cancel)
                .is_err()
        );
        assert!(
            encoder
                .embed_document(
                    Document {
                        path_key: "a",
                        content_sha256: &digest,
                        bytes: b"alterado"
                    },
                    &cancel
                )
                .is_err()
        );
        let syntax_text = "fn café() {}\r\n// fim sem LF";
        let boundary = syntax_text.find("//").unwrap();
        let syntax_hash = sha(syntax_text.as_bytes(), &cancel).unwrap();
        let doc = || Document {
            path_key: "c3ludGF4LnJz",
            content_sha256: &syntax_hash,
            bytes: syntax_text.as_bytes(),
        };
        let syntax_segments = encoder
            .embed_document_ranges(doc(), &[0, boundary, syntax_text.len()], &cancel)
            .unwrap();
        assert_eq!(syntax_segments.len(), 2);
        assert_eq!(
            (
                syntax_segments[0].start_byte,
                syntax_segments[0].end_byte,
                syntax_segments[0].start_line,
                syntax_segments[0].end_line
            ),
            (0, boundary as u64, 1, 1)
        );
        assert_eq!(
            (
                syntax_segments[1].start_byte,
                syntax_segments[1].end_byte,
                syntax_segments[1].start_line,
                syntax_segments[1].end_line
            ),
            (boundary as u64, syntax_text.len() as u64, 2, 2)
        );
        assert!(
            syntax_segments
                .iter()
                .all(|s| s.content_sha256 == syntax_hash)
        );
        for invalid in [
            vec![1, syntax_text.len()],
            vec![0, syntax_text.len() - 1],
            vec![0, boundary, boundary, syntax_text.len()],
            vec![0, syntax_text.find('é').unwrap() + 1, syntax_text.len()],
        ] {
            assert!(
                encoder
                    .embed_document_ranges(doc(), &invalid, &cancel)
                    .is_err()
            );
        }
        let result = serde_json::json!({"model":encoder.model_identity(),"producer_build":crate::BUILD_ID,"semantic_source_sha256":sha(include_bytes!("semantic.rs"), &cancel).unwrap(),"oracle_sha256":sha(include_bytes!("../../../verification/semantic/reference.json"), &cancel).unwrap(),"cases":fixture["cases"].as_array().unwrap().len(),"dimensions":DIMENSIONS,"max_abs_error":max_error,"absolute_tolerance":tolerance,"long_document_spans":segments.len(),"syntax_spans":syntax_segments.len(),"token_ids_exact":true,"cosine_minimum_assertion":0.99999,"checks":["pinned_install_reuse","independent_embeddings","contiguous_utf8_bytes","original_lines","syntax_boundaries","no_silent_truncation","hash_mismatch","cancel"]});
        eprintln!("SEMANTIC_COMPARISON={result}");
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            encoder.embed_query("sessão", &cancel).unwrap_err().code,
            "cancelled"
        );
        eprintln!(
            "independent PyTorch max_abs_error={max_error}; tolerance={tolerance}; spans={}",
            segments.len()
        );
    }
}
