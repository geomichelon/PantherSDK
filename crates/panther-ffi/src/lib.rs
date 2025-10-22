use ffi_support::rust_string_to_c;
use once_cell::sync::OnceCell;
use panther_core::Engine;
use panther_domain::entities::Prompt;
use panther_observability::{init_logging, LogSink};
use panther_providers::NullProvider;
use std::sync::Arc;
use panther_domain::ports::KeyValueStore;
use panthersdk as _; // ensure linkage to panthersdk for metrics/bias helpers
use std::ffi::CStr;
use std::os::raw::c_char;
use std::collections::{HashMap, HashSet};

static ENGINE: OnceCell<Engine> = OnceCell::new();
static LOGS: OnceCell<std::sync::Mutex<Vec<String>>> = OnceCell::new();
static STORAGE: OnceCell<Arc<dyn KeyValueStore>> = OnceCell::new();
#[cfg(feature = "metrics-prometheus")]
static PROM: OnceCell<Arc<panther_metrics::PrometheusMetrics>> = OnceCell::new();

#[no_mangle]
pub extern "C" fn panther_init() -> i32 {
    init_logging();
    let _ = LOGS.set(std::sync::Mutex::new(Vec::new()));
    let provider = Arc::new(NullProvider);
    let telemetry = Some(Arc::new(LogSink) as Arc<dyn panther_domain::ports::TelemetrySink>);
    let engine = Engine::new(provider, telemetry);

    // Conditionally augment the engine without mutable bindings by shadowing
    #[cfg(any(feature = "metrics-inmemory", feature = "metrics-prometheus"))]
    let engine = {
        #[cfg(feature = "metrics-prometheus")]
        let metrics = {
            let m = Arc::new(panther_metrics::PrometheusMetrics::default());
            let _ = PROM.set(m.clone());
            m as Arc<dyn panther_domain::ports::MetricsSink>
        };
        #[cfg(all(not(feature = "metrics-prometheus"), feature = "metrics-inmemory"))]
        let metrics = Arc::new(panther_metrics::InMemoryMetrics::default()) as Arc<dyn panther_domain::ports::MetricsSink>;
        engine.with_metrics(metrics)
    };
    #[cfg(not(any(feature = "metrics-inmemory", feature = "metrics-prometheus")))]
    let engine = engine;

    #[cfg(feature = "storage-inmemory")]
    let engine = {
        let store = Arc::new(panther_storage::InMemoryStore::default());
        let _ = STORAGE.set(store.clone());
        engine.with_storage(store)
    };
    #[cfg(not(feature = "storage-inmemory"))]
    let engine = engine;

    #[cfg(feature = "storage-sled")]
    let engine = {
        let path = std::env::var("PANTHER_SLED_PATH").unwrap_or_else(|_| "./panther_db".to_string());
        match panther_storage_sled::SledStore::open(&path) {
            Ok(store) => {
                let store = Arc::new(store);
                let _ = STORAGE.set(store.clone());
                engine.with_storage(store)
            }
            Err(_) => engine,
        }
    };
    #[cfg(not(feature = "storage-sled"))]
    let engine = engine;
    match ENGINE.set(engine) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

// ---------- Metrics FFI ----------
#[no_mangle]
pub extern "C" fn panther_metrics_bleu(reference: *const c_char, candidate: *const c_char) -> f64 {
    let r = unsafe { CStr::from_ptr(reference).to_string_lossy().into_owned() };
    let c = unsafe { CStr::from_ptr(candidate).to_string_lossy().into_owned() };
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_bleu".to_string())); }
    panthersdk::domain::metrics::evaluate_bleu(&r, &c)
}

#[no_mangle]
pub extern "C" fn panther_metrics_accuracy(expected: *const c_char, generated: *const c_char) -> f64 {
    let e = unsafe { CStr::from_ptr(expected).to_string_lossy().into_owned() };
    let g = unsafe { CStr::from_ptr(generated).to_string_lossy().into_owned() };
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_accuracy".to_string())); }
    panthersdk::domain::metrics::evaluate_accuracy(&e, &g)
}

#[no_mangle]
pub extern "C" fn panther_metrics_coherence(text: *const c_char) -> f64 {
    let t = unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() };
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_coherence".to_string())); }
    panthersdk::domain::metrics::evaluate_coherence(&t)
}

#[no_mangle]
pub extern "C" fn panther_metrics_diversity(samples_json: *const c_char) -> f64 {
    let s = unsafe { CStr::from_ptr(samples_json).to_string_lossy().into_owned() };
    let parsed: Result<Vec<String>, _> = serde_json::from_str(&s);
    match parsed {
        Ok(v) => {
            if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_diversity".to_string())); }
            panthersdk::domain::metrics::evaluate_diversity(&v)
        },
        Err(_) => 0.0,
    }
}

#[no_mangle]
pub extern "C" fn panther_metrics_fluency(text: *const c_char) -> f64 {
    let t = unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() };
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_fluency".to_string())); }
    panthersdk::domain::metrics::evaluate_fluency(&t)
}

#[no_mangle]
pub extern "C" fn panther_metrics_rouge_l(reference: *const c_char, candidate: *const c_char) -> f64 {
    let r = unsafe { CStr::from_ptr(reference).to_string_lossy().into_owned() };
    let c = unsafe { CStr::from_ptr(candidate).to_string_lossy().into_owned() };
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_rouge_l".to_string())); }
    panthersdk::domain::metrics::evaluate_rouge_l(&r, &c)
}

#[no_mangle]
pub extern "C" fn panther_metrics_fact_coverage(facts_json: *const c_char, candidate: *const c_char) -> f64 {
    let facts_s = unsafe { CStr::from_ptr(facts_json).to_string_lossy().into_owned() };
    let cand = unsafe { CStr::from_ptr(candidate).to_string_lossy().into_owned() };
    let facts: Vec<String> = serde_json::from_str(&facts_s).unwrap_or_default();
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_fact_coverage".to_string())); }
    panthersdk::domain::metrics::evaluate_fact_coverage(&facts, &cand)
}

#[no_mangle]
pub extern "C" fn panther_metrics_factcheck_adv(facts_json: *const c_char, candidate: *const c_char) -> f64 {
    let facts_s = unsafe { CStr::from_ptr(facts_json).to_string_lossy().into_owned() };
    let cand = unsafe { CStr::from_ptr(candidate).to_string_lossy().into_owned() };
    let facts: Vec<String> = serde_json::from_str(&facts_s).unwrap_or_default();
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_factcheck_adv".to_string())); }
    panthersdk::domain::metrics::evaluate_factcheck_adv(&facts, &cand)
}

#[no_mangle]
pub extern "C" fn panther_metrics_plagiarism(corpus_json: *const c_char, candidate: *const c_char) -> f64 {
    let corpus_s = unsafe { CStr::from_ptr(corpus_json).to_string_lossy().into_owned() };
    let cand = unsafe { CStr::from_ptr(candidate).to_string_lossy().into_owned() };
    let corpus: Vec<String> = serde_json::from_str(&corpus_s).unwrap_or_default();
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("metrics_plagiarism".to_string())); }
    panthersdk::domain::metrics::evaluate_plagiarism(&corpus, &cand)
}

#[no_mangle]
pub extern "C" fn panther_metrics_plagiarism_ngram(
    corpus_json: *const c_char,
    candidate: *const c_char,
    ngram: i32,
) -> f64 {
    let corpus_s = unsafe { CStr::from_ptr(corpus_json).to_string_lossy().into_owned() };
    let cand = unsafe { CStr::from_ptr(candidate).to_string_lossy().into_owned() };
    let corpus: Vec<String> = serde_json::from_str(&corpus_s).unwrap_or_default();
    let n = if ngram <= 0 { 3usize } else { ngram as usize };
    if let Some(buf) = LOGS.get() {
        let _ = buf.lock().map(|mut v| v.push(format!("metrics_plagiarism_ngram:{}", n)));
    }
    panthersdk::domain::metrics::evaluate_plagiarism_ngram(&corpus, &cand, n)
}

#[no_mangle]
pub extern "C" fn panther_metrics_record(name: *const c_char, value: f64) -> i32 {
    if let Some(engine) = ENGINE.get() {
        let nm = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
        engine.record_metric(&nm, value);
        if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push(format!("record_metric:{}", nm))); }
        return 0;
    }
    1
}

// ---------- Bias FFI ----------
#[no_mangle]
pub extern "C" fn panther_bias_detect(samples_json: *const c_char) -> *mut std::os::raw::c_char {
    let s = unsafe { CStr::from_ptr(samples_json).to_string_lossy().into_owned() };
    let parsed: Result<Vec<String>, _> = serde_json::from_str(&s);
    match parsed {
        Ok(v) => {
            if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("bias_detect".to_string())); }
            let rep = panthersdk::domain::bias::detect_bias(&v);
            rust_string_to_c(serde_json::to_string(&rep).unwrap_or_else(|_| "{}".to_string()))
        }
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

// ---------- Storage FFI ----------
#[no_mangle]
pub extern "C" fn panther_storage_save_metric(name: *const c_char, value: f64, timestamp_ms: i64) -> i32 {
    if let Some(store) = STORAGE.get() {
        let s: &dyn KeyValueStore = &**store;
        let nm = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
        let r = panthersdk::domain::storage::save_metric(s, &nm, value, timestamp_ms);
        let rc = if r.is_ok() { 0 } else { 1 };
        if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push(format!("save_metric:{}", nm))); }
        return rc;
    }
    2
}

#[no_mangle]
pub extern "C" fn panther_storage_get_history(metric: *const c_char) -> *mut std::os::raw::c_char {
    if let Some(store) = STORAGE.get() {
        let s: &dyn KeyValueStore = &**store;
        let m = unsafe { CStr::from_ptr(metric).to_string_lossy().into_owned() };
        let r = panthersdk::domain::storage::get_history(s, &m);
        let out = match r {
            Ok(hist) => rust_string_to_c(serde_json::to_string(&hist).unwrap_or_else(|_| "[]".to_string())),
            Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
        };
        if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push(format!("get_history:{}", m))); }
        return out;
    }
    rust_string_to_c("[]".to_string())
}

#[no_mangle]
pub extern "C" fn panther_storage_export(format_c: *const c_char) -> *mut std::os::raw::c_char {
    if let Some(store) = STORAGE.get() {
        let s: &dyn KeyValueStore = &**store;
        let fmt = unsafe { CStr::from_ptr(format_c).to_string_lossy().into_owned() };
        let r = panthersdk::domain::storage::export_metrics(s, &fmt);
        let out = match r {
            Ok(s) => rust_string_to_c(s),
            Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
        };
        if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("export_metrics".to_string())); }
        return out;
    }
    rust_string_to_c("[]".to_string())
}

#[no_mangle]
pub extern "C" fn panther_storage_list_metrics() -> *mut std::os::raw::c_char {
    if let Some(store) = STORAGE.get() {
        let s: &dyn KeyValueStore = &**store;
        match panthersdk::domain::storage::list_metrics(s) {
            Ok(list) => rust_string_to_c(serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string())),
            Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
        }
    } else {
        rust_string_to_c("[]".to_string())
    }
}

#[no_mangle]
pub extern "C" fn panther_logs_get() -> *mut std::os::raw::c_char {
    if let Some(buf) = LOGS.get() {
        if let Ok(v) = buf.lock() {
            return rust_string_to_c(serde_json::to_string(&*v).unwrap_or_else(|_| "[]".to_string()));
        }
    }
    rust_string_to_c("[]".to_string())
}

#[no_mangle]
pub extern "C" fn panther_logs_get_recent() -> *mut std::os::raw::c_char {
    if let Some(buf) = LOGS.get() {
        if let Ok(v) = buf.lock() {
            let len = v.len();
            let start = len.saturating_sub(50);
            let slice = &v[start..];
            return rust_string_to_c(serde_json::to_string(slice).unwrap_or_else(|_| "[]".to_string()));
        }
    }
    rust_string_to_c("[]".to_string())
}

#[no_mangle]
pub extern "C" fn panther_version_string() -> *mut std::os::raw::c_char {
    // Return the crate version from Cargo metadata to avoid manual drift
    rust_string_to_c(env!("CARGO_PKG_VERSION").to_string())
}

// ---------- Token/Cost FFI ----------
#[no_mangle]
pub extern "C" fn panther_token_count(text: *const c_char) -> i32 {
    if text.is_null() { return 0; }
    let t = unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() };
    // Simple whitespace tokenizer heuristic (kept consistent with panther-core)
    t.split_whitespace().count() as i32
}

// ---------- Guidelines Similarity (MVP: BOW cosine) ----------
static GUIDELINES_INDEX: OnceCell<std::sync::Mutex<Vec<(String, HashMap<String, f64>, HashSet<String>)>>> = OnceCell::new();
static GUIDELINES_RAW: OnceCell<std::sync::Mutex<Vec<String>>> = OnceCell::new();

fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

fn vec_from_text(s: &str) -> HashMap<String, f64> {
    let mut map: HashMap<String, f64> = HashMap::new();
    for tok in tokenize(s) { *map.entry(tok).or_insert(0.0) += 1.0; }
    let norm = map.values().map(|v| v*v).sum::<f64>().sqrt();
    if norm > 0.0 { for v in map.values_mut() { *v /= norm; } }
    map
}

fn set_from_text(s: &str) -> HashSet<String> {
    tokenize(s).into_iter().collect()
}

fn cosine(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let mut dot = 0.0;
    for (k, va) in small.iter() {
        if let Some(vb) = large.get(k) { dot += va * vb; }
    }
    dot
}

#[no_mangle]
pub extern "C" fn panther_guidelines_ingest_json(guidelines_json_c: *const c_char) -> i32 {
    if guidelines_json_c.is_null() { return 0; }
    let s = unsafe { CStr::from_ptr(guidelines_json_c).to_string_lossy().into_owned() };
    let mut items: Vec<(String, HashMap<String, f64>, HashSet<String>)> = Vec::new();
    let mut raws: Vec<String> = Vec::new();
    // Accept array of objects with {topic, expected_terms[]} or array of strings
    let v: Result<serde_json::Value, _> = serde_json::from_str(&s);
    match v {
        Ok(serde_json::Value::Array(arr)) => {
            for it in arr {
                match it {
                    serde_json::Value::Object(obj) => {
                        let topic = obj.get("topic").and_then(|x| x.as_str()).unwrap_or("");
                        let expected = obj.get("expected_terms").and_then(|x| x.as_array()).map(|a| {
                            a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" ")
                        }).unwrap_or_default();
                        let combined = if expected.is_empty() { topic.to_string() } else { format!("{} {}", topic, expected) };
                        let vec = vec_from_text(&combined);
                        let setv = set_from_text(&combined);
                        items.push((topic.to_string(), vec, setv));
                        raws.push(combined);
                    }
                    serde_json::Value::String(txt) => {
                        let vec = vec_from_text(&txt);
                        let setv = set_from_text(&txt);
                        raws.push(txt.clone());
                        items.push((txt, vec, setv));
                    }
                    _ => {}
                }
            }
        }
        _ => { return 0; }
    }
    let m = GUIDELINES_INDEX.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    if let Ok(mut guard) = m.lock() {
        *guard = items;
        if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("guidelines_ingest".to_string())); }
        // store raws in parallel OnceCell
        let raw_m = GUIDELINES_RAW.get_or_init(|| std::sync::Mutex::new(Vec::new()));
        if let Ok(mut r) = raw_m.lock() { *r = raws; }
        return guard.len() as i32;
    }
    0
}

#[no_mangle]
pub extern "C" fn panther_guidelines_similarity(query_c: *const c_char, top_k: i32, method_c: *const c_char) -> *mut std::os::raw::c_char {
    if query_c.is_null() { return rust_string_to_c("[]".to_string()); }
    let method = unsafe { if method_c.is_null() { "bow".to_string() } else { CStr::from_ptr(method_c).to_string_lossy().into_owned() } };
    let q = unsafe { CStr::from_ptr(query_c).to_string_lossy().into_owned() };
    let qv = vec_from_text(&q);
    let qs = set_from_text(&q);
    let k = if top_k <= 0 { 5 } else { top_k as usize };
    if method == "embed-openai" || method == "embed-ollama" {
        // Embedding-based path
        return crate::embed::similarity_embed(&q, k, &method);
    }
    if let Some(mtx) = GUIDELINES_INDEX.get() {
        if let Ok(idx) = mtx.lock() {
            let mut outv: Vec<(String, f64, f64, f64)> = Vec::new();
            for (id, vec, setv) in idx.iter() {
                let bow = cosine(&qv, vec);
                let inter = qs.intersection(setv).count() as f64;
                let union = qs.union(setv).count() as f64;
                let jacc = if union > 0.0 { inter / union } else { 0.0 };
                let hybrid = 0.6 * bow + 0.4 * jacc;
                let score = match method.as_str() { "bow" => bow, "jaccard" => jacc, "hybrid" => hybrid, _ => bow };
                outv.push((id.clone(), score, bow, jacc));
            }
            outv.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let out: Vec<serde_json::Value> = outv.into_iter().take(k).map(|(id,score,bow,jacc)| serde_json::json!({"topic": id, "score": score, "bow": bow, "jaccard": jacc})).collect();
            if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("guidelines_similarity".to_string())); }
            return rust_string_to_c(serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string()));
        }
    }
    rust_string_to_c("[]".to_string())
}

// Persist raw guidelines JSON for reuse
#[no_mangle]
pub extern "C" fn panther_guidelines_save_json(name_c: *const c_char, json_c: *const c_char) -> i32 {
    if name_c.is_null() || json_c.is_null() { return 1; }
    if let Some(store) = STORAGE.get() {
        let s: &dyn KeyValueStore = &**store;
        let name = unsafe { CStr::from_ptr(name_c).to_string_lossy().into_owned() };
        let json = unsafe { CStr::from_ptr(json_c).to_string_lossy().into_owned() };
        let key = format!("guidelines:{}", name);
        match s.set(&key, json) { Ok(_) => 0, Err(_) => 1 }
    } else { 2 }
}

#[no_mangle]
pub extern "C" fn panther_guidelines_load(name_c: *const c_char) -> i32 {
    if name_c.is_null() { return 0; }
    if let Some(store) = STORAGE.get() {
        let s: &dyn KeyValueStore = &**store;
        let name = unsafe { CStr::from_ptr(name_c).to_string_lossy().into_owned() };
        let key = format!("guidelines:{}", name);
        match s.get(&key) {
            Ok(Some(json)) => panther_guidelines_ingest_json(std::ffi::CString::new(json).unwrap().as_ptr()),
            _ => 0,
        }
    } else { 0 }
}

// ---------- Embeddings (optional) ----------
mod embed {
    use super::*;
    use once_cell::sync::OnceCell;

    struct EmbedIndex { method: String, vectors: Vec<Vec<f32>> }
    static EMBED_INDEX: OnceCell<std::sync::Mutex<Option<EmbedIndex>>> = OnceCell::new();

    fn normalize(v: &mut Vec<f32>) {
        let norm = (v.iter().map(|x| (*x as f64)*(*x as f64)).sum::<f64>()).sqrt();
        if norm > 0.0 {
            let inv = 1.0 / norm as f32;
            for x in v.iter_mut() { *x *= inv; }
        }
    }
    fn dot(a: &[f32], b: &[f32]) -> f64 {
        let (small, large) = if a.len() <= b.len() { (a,b) } else { (b,a) };
        let mut s = 0.0f64;
        for (i, x) in small.iter().enumerate() { s += (*x as f64) * (large[i] as f64); }
        s
    }

    #[cfg(feature = "guidelines-embed-openai")]
    async fn embed_openai(texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let base = std::env::var("PANTHER_OPENAI_BASE").unwrap_or_else(|_| "https://api.openai.com".to_string());
        let key = std::env::var("PANTHER_OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("missing PANTHER_OPENAI_API_KEY"))?;
        let model = std::env::var("PANTHER_OPENAI_EMBED_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());
        let client = reqwest::Client::new();
        let url = format!("{}/v1/embeddings", base.trim_end_matches('/'));
        #[derive(serde::Serialize)] struct Req<'a> { input: &'a [String], model: String }
        let resp = client.post(&url)
            .bearer_auth(key)
            .json(&Req { input: texts, model })
            .send().await?;
        let val: serde_json::Value = resp.json().await?;
        let mut out: Vec<Vec<f32>> = Vec::new();
        if let Some(arr) = val.get("data").and_then(|v| v.as_array()) {
            for d in arr {
                let vec = d.get("embedding").and_then(|e| e.as_array()).map(|a| {
                    a.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect::<Vec<f32>>()
                }).unwrap_or_default();
                out.push(vec);
            }
        }
        Ok(out)
    }

    #[cfg(not(feature = "guidelines-embed-openai"))]
    async fn embed_openai(_texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> { Err(anyhow::anyhow!("openai embed feature not enabled")) }

    #[cfg(feature = "guidelines-embed-ollama")]
    async fn embed_ollama(texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let base = std::env::var("PANTHER_OLLAMA_BASE").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        let model = std::env::var("PANTHER_OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
        let client = reqwest::Client::new();
        let url = format!("{}/api/embeddings", base.trim_end_matches('/'));
        let mut out: Vec<Vec<f32>> = Vec::new();
        for t in texts {
            let resp = client.post(&url).json(&serde_json::json!({"model": model, "prompt": t})).send().await?;
            let val: serde_json::Value = resp.json().await?;
            let vec = val.get("embedding").and_then(|e| e.as_array()).map(|a| a.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect::<Vec<f32>>()).unwrap_or_default();
            out.push(vec);
        }
        Ok(out)
    }

    #[cfg(not(feature = "guidelines-embed-ollama"))]
    async fn embed_ollama(_texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> { Err(anyhow::anyhow!("ollama embed feature not enabled")) }

    fn build_rt() -> Result<tokio::runtime::Runtime, ()> {
        tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|_| ())
    }

    pub fn similarity_embed(query: &str, k: usize, method: &str) -> *mut std::os::raw::c_char {
        // Build index if missing
        let mtx = EMBED_INDEX.get_or_init(|| std::sync::Mutex::new(None));
        // Ensure we have raw texts
        let raws_m = super::GUIDELINES_RAW.get();
        let raws: Vec<String> = if let Some(r) = raws_m { r.lock().ok().map(|g| g.clone()).unwrap_or_default() } else { Vec::new() };
        if raws.is_empty() {
            return rust_string_to_c("[]".to_string());
        }
        // Build if empty or mismatched method
        let need_build = {
            if let Ok(guard) = mtx.lock() {
                match &*guard { Some(ei) if ei.method == method => false, _ => true }
            } else { true }
        };
        if need_build {
            if let Ok(rt) = build_rt() {
                let built = rt.block_on(async {
                    let vecs = match method {
                        "embed-openai" => embed_openai(&raws).await,
                        "embed-ollama" => embed_ollama(&raws).await,
                        _ => Err(anyhow::anyhow!("unknown method")),
                    }?;
                    let mut vecs = vecs;
                    for v in vecs.iter_mut() { normalize(v); }
                    Ok::<Vec<Vec<f32>>, anyhow::Error>(vecs)
                });
                if let Ok(vecs) = built {
                    if let Ok(mut guard) = mtx.lock() { *guard = Some(EmbedIndex { method: method.to_string(), vectors: vecs }); }
                } else {
                    return rust_string_to_c("[]".to_string());
                }
            } else {
                return rust_string_to_c("[]".to_string());
            }
        }
        // Compute query embedding
        let q_vec: Vec<f32> = if let Ok(rt) = build_rt() {
            let out = rt.block_on(async {
                let v = match method {
                    "embed-openai" => embed_openai(&vec![query.to_string()]).await,
                    "embed-ollama" => embed_ollama(&vec![query.to_string()]).await,
                    _ => Err(anyhow::anyhow!("unknown method")),
                }?;
                Ok::<Vec<f32>, anyhow::Error>(v.into_iter().next().unwrap_or_default())
            });
            match out { Ok(mut v) => { normalize(&mut v); v }, Err(_) => Vec::new() }
        } else { Vec::new() };
        if q_vec.is_empty() { return rust_string_to_c("[]".to_string()); }
        // Score
        if let Ok(guard) = mtx.lock() {
            if let Some(ei) = &*guard {
                let ids: Vec<String> = if let Some(idx) = super::GUIDELINES_INDEX.get() {
                    idx.lock().ok().map(|v| v.iter().map(|(id,_,_)| id.clone()).collect()).unwrap_or_default()
                } else { Vec::new() };
                let mut scored: Vec<(String, f64)> = Vec::new();
                for (i, v) in ei.vectors.iter().enumerate() {
                    let s = dot(&q_vec, v);
                    let id = ids.get(i).cloned().unwrap_or_else(|| format!("item:{}", i));
                    scored.push((id, s));
                }
                scored.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let out: Vec<serde_json::Value> = scored.into_iter().take(k).map(|(id,score)| serde_json::json!({"topic": id, "score": score})).collect();
                return rust_string_to_c(serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string()));
            }
        }
        rust_string_to_c("[]".to_string())
    }

    #[no_mangle]
    pub extern "C" fn panther_guidelines_embeddings_build(method_c: *const c_char) -> i32 {
        if method_c.is_null() { return 0; }
        let method = unsafe { CStr::from_ptr(method_c).to_string_lossy().into_owned() };
        let raws_m = super::GUIDELINES_RAW.get();
        let raws: Vec<String> = if let Some(r) = raws_m { r.lock().ok().map(|g| g.clone()).unwrap_or_default() } else { Vec::new() };
        if raws.is_empty() { return 0; }
        if let Ok(rt) = build_rt() {
            let built = rt.block_on(async {
                let vecs = match method.as_str() {
                    "embed-openai" => embed_openai(&raws).await,
                    "embed-ollama" => embed_ollama(&raws).await,
                    _ => Err(anyhow::anyhow!("unknown method")),
                }?;
                let mut vecs = vecs;
                for v in vecs.iter_mut() { normalize(v); }
                Ok::<Vec<Vec<f32>>, anyhow::Error>(vecs)
            });
            if let Ok(vecs) = built {
                let mtx = EMBED_INDEX.get_or_init(|| std::sync::Mutex::new(None));
                if let Ok(mut guard) = mtx.lock() { *guard = Some(EmbedIndex { method: method.to_string(), vectors: vecs }); }
                return raws.len() as i32;
            }
        }
        0
    }
}

#[derive(serde::Deserialize, Debug)]
struct CostRuleCompat {
    // Accept both keys: `match` (used by samples) and `provider` (used by tools)
    #[serde(default)]
    r#match: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    usd_per_1k_in: Option<f64>,
    #[serde(default)]
    usd_per_1k_out: Option<f64>,
    // Optional alternative: allow prices per million tokens for convenience
    #[serde(default)]
    usd_per_1m_in: Option<f64>,
    #[serde(default)]
    usd_per_1m_out: Option<f64>,
}

fn select_rule<'a>(prov: &str, rules: &'a [CostRuleCompat]) -> Option<&'a CostRuleCompat> {
    // Prefer the longest matching prefix; support exact, prefix with trailing ':', and wildcard '*'
    let mut best: Option<&CostRuleCompat> = None;
    let mut best_len: usize = 0;
    for r in rules {
        let key = r.r#match.as_ref().or(r.provider.as_ref());
        if let Some(k) = key {
            if k == prov {
                return Some(r);
            }
            if k == "*" {
                if best.is_none() { best = Some(r); best_len = 1; }
                continue;
            }
            if k.ends_with(':') && prov.starts_with(k) {
                let len = k.len();
                if len > best_len { best = Some(r); best_len = len; }
            }
        }
    }
    best
}

#[no_mangle]
pub extern "C" fn panther_calculate_cost(
    tokens_in: i32,
    tokens_out: i32,
    provider_name: *const c_char,
    cost_rules_json: *const c_char,
) -> f64 {
    if provider_name.is_null() || cost_rules_json.is_null() { return 0.0; }
    let prov = unsafe { CStr::from_ptr(provider_name).to_string_lossy().into_owned() };
    let rules_s = unsafe { CStr::from_ptr(cost_rules_json).to_string_lossy().into_owned() };
    let mut rules_vec: Vec<CostRuleCompat> = Vec::new();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&rules_s) {
        if let Some(arr) = v.as_array() {
            rules_vec = arr.iter().filter_map(|x| serde_json::from_value::<CostRuleCompat>(x.clone()).ok()).collect();
        } else if let Some(arr) = v.get("rules").and_then(|x| x.as_array()) {
            rules_vec = arr.iter().filter_map(|x| serde_json::from_value::<CostRuleCompat>(x.clone()).ok()).collect();
        }
    }
    if let Some(rule) = select_rule(&prov, &rules_vec) {
        let cin = match (rule.usd_per_1k_in, rule.usd_per_1m_in) {
            (Some(v), _) => v,
            (None, Some(m)) => m / 1000.0,
            _ => 0.0,
        };
        let cout = match (rule.usd_per_1k_out, rule.usd_per_1m_out) {
            (Some(v), _) => v,
            (None, Some(m)) => m / 1000.0,
            _ => 0.0,
        };
        let ti = (tokens_in.max(0) as f64) / 1000.0;
        let to = (tokens_out.max(0) as f64) / 1000.0;
        return ti * cin + to * cout;
    }
    0.0
}

#[no_mangle]
pub extern "C" fn panther_get_token_metrics() -> *mut std::os::raw::c_char {
    // Minimal stub: return an empty object to satisfy linkage for samples that might call this.
    // Token histories can be retrieved via storage APIs if enabled.
    rust_string_to_c("{}".to_string())
}

// ---------- Agents (optional Stage 6) ----------
#[cfg(feature = "agents")]
#[no_mangle]
pub extern "C" fn panther_agent_run(plan_json_c: *const c_char, input_json_c: *const c_char) -> *mut std::os::raw::c_char {
    let plan_json = unsafe { CStr::from_ptr(plan_json_c).to_string_lossy().into_owned() };
    let input_json = unsafe { CStr::from_ptr(input_json_c).to_string_lossy().into_owned() };
    match panther_agents::run_plan(&plan_json, &input_json) {
        Ok(res) => {
            let s = serde_json::to_string(&res).unwrap_or_else(|_| "{}".to_string());
            rust_string_to_c(s)
        }
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

// Incremental agents API
#[cfg(feature = "agents")]
#[no_mangle]
pub extern "C" fn panther_agent_start(plan_json_c: *const c_char, input_json_c: *const c_char) -> *mut std::os::raw::c_char {
    let plan_json = unsafe { CStr::from_ptr(plan_json_c).to_string_lossy().into_owned() };
    let input_json = unsafe { CStr::from_ptr(input_json_c).to_string_lossy().into_owned() };
    match panther_agents::agent_start(&plan_json, &input_json) {
        Ok(run_id) => rust_string_to_c(serde_json::json!({"run_id": run_id}).to_string()),
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[cfg(feature = "agents")]
#[no_mangle]
pub extern "C" fn panther_agent_poll(run_id_c: *const c_char, cursor_c: *const c_char) -> *mut std::os::raw::c_char {
    let run_id = unsafe { CStr::from_ptr(run_id_c).to_string_lossy().into_owned() };
    let cursor_s = unsafe { CStr::from_ptr(cursor_c).to_string_lossy().into_owned() };
    let cursor = cursor_s.parse::<usize>().unwrap_or(0);
    match panther_agents::agent_poll(&run_id, cursor) {
        Ok((events, done, new_cursor, status)) => {
            let out = serde_json::json!({"events": events, "done": done, "cursor": new_cursor, "status": status});
            rust_string_to_c(out.to_string())
        }
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[cfg(feature = "agents")]
#[no_mangle]
pub extern "C" fn panther_agent_status(run_id_c: *const c_char) -> *mut std::os::raw::c_char {
    let run_id = unsafe { CStr::from_ptr(run_id_c).to_string_lossy().into_owned() };
    match panther_agents::agent_status(&run_id) {
        Ok((status, done)) => {
            let out = serde_json::json!({"status": status, "done": done});
            rust_string_to_c(out.to_string())
        }
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[cfg(feature = "agents")]
#[no_mangle]
pub extern "C" fn panther_agent_result(run_id_c: *const c_char) -> *mut std::os::raw::c_char {
    let run_id = unsafe { CStr::from_ptr(run_id_c).to_string_lossy().into_owned() };
    match panther_agents::agent_result(&run_id) {
        Ok(Some(outcome)) => rust_string_to_c(serde_json::to_string(&outcome).unwrap_or_else(|_| "{}".to_string())),
        Ok(None) => rust_string_to_c("null".to_string()),
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[no_mangle]
pub extern "C" fn panther_generate(prompt_c: *const c_char) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let engine = ENGINE.get().expect("panther_init not called");
    let res = engine.generate(Prompt { text: prompt });
    let out = match res {
        Ok(c) => rust_string_to_c(serde_json::to_string(&c).unwrap_or_default()),
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    };
    if let Some(buf) = LOGS.get() { let _ = buf.lock().map(|mut v| v.push("generate called".to_string())); }
    out
}

#[no_mangle]
pub extern "C" fn panther_free_string(s: *mut std::os::raw::c_char) {
    unsafe {
        if !s.is_null() {
            let _ = std::ffi::CString::from_raw(s);
        }
    }
}

// ---------- Validation (optional) ----------
#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_validation_run_custom_with_proof(
    prompt_c: *const c_char,
    providers_json_c: *const c_char,
    guidelines_json_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let providers_json = unsafe { CStr::from_ptr(providers_json_c).to_string_lossy().into_owned() };
    let guidelines_json = unsafe { CStr::from_ptr(guidelines_json_c).to_string_lossy().into_owned() };

    #[derive(serde::Deserialize)]
    struct ProviderCfg { #[serde(rename = "type")] ty: String, base_url: Option<String>, model: Option<String>, api_key: Option<String> }
    let cfgs: Result<Vec<ProviderCfg>, _> = serde_json::from_str(&providers_json);
    if let Err(e) = cfgs { return rust_string_to_c(format!("{{\"error\":\"providers json invalid: {}\"}}", e)); }
    let cfgs = cfgs.unwrap();

    // Try async providers first
    #[cfg(feature = "validation-async")]
    {
        let mut providers_async: Vec<(String, Arc<dyn panther_domain::ports::LlmProviderAsync>)> = Vec::new();
        for c in &cfgs {
            match c.ty.as_str() {
                #[cfg(feature = "validation-openai-async")]
                "openai" => {
                    if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                        let base = c.base_url.clone().unwrap_or_else(|| "https://api.openai.com".to_string());
                        let p = panther_providers::openai_async::OpenAiProviderAsync { api_key: key, model: model.clone(), base_url: base, timeout_secs: 30, retries: 2 };
                        providers_async.push((format!("openai:{}", model), Arc::new(p)));
                    }
                }
                #[cfg(feature = "validation-ollama-async")]
                "ollama" => {
                    if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                        let p = panther_providers::ollama_async::OllamaProviderAsync { base_url: base, model: model.clone(), timeout_secs: 30, retries: 2 };
                        providers_async.push((format!("ollama:{}", model), Arc::new(p)));
                    }
                }
                #[cfg(feature = "validation-anthropic-async")]
                "anthropic" => {
                    if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                        let base = c.base_url.clone().unwrap_or_else(|| "https://api.anthropic.com".to_string());
                        let p = panther_providers::anthropic_async::AnthropicProviderAsync {
                            api_key: key,
                            model: model.clone(),
                            base_url: base,
                            version: "2023-06-01".to_string(),
                            timeout_secs: 30,
                            retries: 2,
                        };
                        providers_async.push((format!("anthropic:{}", model), Arc::new(p)));
                    }
                }
                _ => {}
            }
        }
        if !providers_async.is_empty() {
            let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
                Ok(r) => r,
                Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
            };
            let prompt_clone = prompt.clone();
            let providers_json_clone = providers_json.clone();
            let guidelines_json_clone = guidelines_json.clone();
            let res = rt.block_on(async move {
                let validator = panther_validation::LLMValidatorAsync::from_json_str(&guidelines_json_clone, providers_async)?;
                let results = validator.validate(&prompt_clone).await?;
                Ok::<Vec<panther_validation::ValidationResult>, anyhow::Error>(results)
            });
            return match res {
                Ok(results) => {
                    let results_json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
                    let ctx = panther_validation::proof::ProofContext { sdk_version: env!("CARGO_PKG_VERSION").to_string(), salt: None };
                    let proof = match panther_validation::proof::compute_proof(&prompt, &providers_json_clone, &guidelines_json, &results_json, &ctx) {
                        Ok(p) => p,
                        Err(e) => return rust_string_to_c(format!("{{\"error\":\"compute proof failed: {}\"}}", e)),
                    };
                    let out = serde_json::json!({ "results": results, "proof": proof });
                    rust_string_to_c(serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_string()))
                }
                Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
            };
        }
    }

    // Fallback to synchronous providers
    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    for c in cfgs {
        match c.ty.as_str() {
            #[cfg(feature = "validation-openai")]
            "openai" => {
                if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                    let base = c.base_url.unwrap_or_else(|| "https://api.openai.com".to_string());
                    let p = panther_providers::openai::OpenAiProvider { api_key: key, model: model.clone(), base_url: base };
                    providers.push((format!("openai:{}", model), Arc::new(p)));
                }
            }
            #[cfg(feature = "validation-ollama")]
            "ollama" => {
                if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                    let p = panther_providers::ollama::OllamaProvider { base_url: base, model: model.clone() };
                    providers.push((format!("ollama:{}", model), Arc::new(p)));
                }
            }
            #[cfg(feature = "validation-anthropic")]
            "anthropic" => {
                if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                    let base = c.base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string());
                    let p = panther_providers::anthropic::AnthropicProvider { api_key: key, model: model.clone(), base_url: base, version: "2023-06-01".to_string() };
                    providers.push((format!("anthropic:{}", model), Arc::new(p)));
                }
            }
            _ => {}
        }
    }
    if providers.is_empty() { return rust_string_to_c("{\"error\":\"no providers configured\"}".to_string()); }
    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let prompt_clone2 = prompt.clone();
    let guidelines_json_clone2 = guidelines_json.clone();
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(&guidelines_json_clone2, providers)?;
        let results = validator.validate(&prompt_clone2).await?;
        Ok::<Vec<panther_validation::ValidationResult>, anyhow::Error>(results)
    });
    match res {
        Ok(results) => {
            let results_json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
            let ctx = panther_validation::proof::ProofContext { sdk_version: env!("CARGO_PKG_VERSION").to_string(), salt: None };
            let proof = match panther_validation::proof::compute_proof(&prompt, &providers_json, &guidelines_json, &results_json, &ctx) {
                Ok(p) => p,
                Err(e) => return rust_string_to_c(format!("{{\"error\":\"compute proof failed: {}\"}}", e)),
            };
            let out = serde_json::json!({ "results": results, "proof": proof });
            rust_string_to_c(serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_string()))
        }
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}
#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_validation_run_default(prompt_c: *const c_char) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let guidelines_json: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../panther-validation/guidelines/anvisa.json"));

    // Build providers from env
    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    #[cfg(feature = "validation-openai")]
    if let Ok(p) = panther_validation::ProviderFactory::openai_from_env() { providers.push(p); }
    #[cfg(feature = "validation-ollama")]
    if let Ok(p) = panther_validation::ProviderFactory::ollama_from_env() { providers.push(p); }
    if providers.is_empty() {
        return rust_string_to_c("{\"error\":\"no providers configured\"}".to_string());
    }

    // Run async validator on a fresh runtime
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build();
    if rt.is_err() {
        return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string());
    }
    let rt = rt.unwrap();
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(guidelines_json, providers)?;
        let results = validator.validate(&prompt).await?;
        Ok::<String, anyhow::Error>(serde_json::to_string(&results)? )
    });
    match res {
        Ok(s) => rust_string_to_c(s),
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

// With proof: returns { "results": [...], "proof": {..} }
#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_validation_run_multi_with_proof(
    prompt_c: *const c_char,
    providers_json_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let providers_json = unsafe { CStr::from_ptr(providers_json_c).to_string_lossy().into_owned() };
    let guidelines_json: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../panther-validation/guidelines/anvisa.json"));

    // Build providers per JSON using same logic as run_multi
    #[derive(serde::Deserialize)]
    struct ProviderCfg { #[serde(rename = "type")] ty: String, base_url: Option<String>, model: Option<String>, api_key: Option<String> }
    let cfgs: Result<Vec<ProviderCfg>, _> = serde_json::from_str(&providers_json);
    if let Err(e) = cfgs { return rust_string_to_c(format!("{{\"error\":\"providers json invalid: {}\"}}", e)); }
    let cfgs = cfgs.unwrap();

    // If async validation features are enabled, try building async providers first
    #[cfg(feature = "validation-async")]
    {
        let mut providers_async: Vec<(String, Arc<dyn panther_domain::ports::LlmProviderAsync>)> = Vec::new();
        for c in &cfgs {
            match c.ty.as_str() {
                #[cfg(feature = "validation-openai-async")]
                "openai" => {
                    if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                        let base = c.base_url.clone().unwrap_or_else(|| "https://api.openai.com".to_string());
                        let p = panther_providers::openai_async::OpenAiProviderAsync { api_key: key, model: model.clone(), base_url: base, timeout_secs: 30, retries: 2 };
                        providers_async.push((format!("openai:{}", model), Arc::new(p)));
                    }
                }
                #[cfg(feature = "validation-ollama-async")]
                "ollama" => {
                    if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                        let p = panther_providers::ollama_async::OllamaProviderAsync { base_url: base, model: model.clone(), timeout_secs: 30, retries: 2 };
                        providers_async.push((format!("ollama:{}", model), Arc::new(p)));
                    }
                }
                _ => {}
            }
        }
        if !providers_async.is_empty() {
            let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
                Ok(r) => r,
                Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
            };
            let prompt_clone = prompt.clone();
            let providers_json_clone = providers_json.clone();
            let res = rt.block_on(async move {
                let validator = panther_validation::LLMValidatorAsync::from_json_str(guidelines_json, providers_async)?;
                let results = validator.validate(&prompt_clone).await?;
                Ok::<Vec<panther_validation::ValidationResult>, anyhow::Error>(results)
            });
            return match res {
                Ok(results) => {
                    let results_json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
                    let ctx = panther_validation::proof::ProofContext { sdk_version: env!("CARGO_PKG_VERSION").to_string(), salt: None };
                    let proof = match panther_validation::proof::compute_proof(&prompt, &providers_json_clone, guidelines_json, &results_json, &ctx) {
                        Ok(p) => p,
                        Err(e) => return rust_string_to_c(format!("{{\"error\":\"compute proof failed: {}\"}}", e)),
                    };
                    let out = serde_json::json!({ "results": results, "proof": proof });
                    rust_string_to_c(serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_string()))
                }
                Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
            };
        }
    }

    // Fallback to synchronous providers if no async provider was built
    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    for c in cfgs {
        match c.ty.as_str() {
            #[cfg(feature = "validation-openai")]
            "openai" => {
                if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                    let base = c.base_url.unwrap_or_else(|| "https://api.openai.com".to_string());
                    let p = panther_providers::openai::OpenAiProvider { api_key: key, model: model.clone(), base_url: base };
                    providers.push((format!("openai:{}", model), Arc::new(p)));
                }
            }
            #[cfg(feature = "validation-ollama")]
            "ollama" => {
                if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                    let p = panther_providers::ollama::OllamaProvider { base_url: base, model: model.clone() };
                    providers.push((format!("ollama:{}", model), Arc::new(p)));
                }
            }
            #[cfg(feature = "validation-anthropic")]
            "anthropic" => {
                if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                    let base = c.base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string());
                    let p = panther_providers::anthropic::AnthropicProvider { api_key: key, model: model.clone(), base_url: base, version: "2023-06-01".to_string() };
                    providers.push((format!("anthropic:{}", model), Arc::new(p)));
                }
            }
            _ => {}
        }
    }
    if providers.is_empty() { return rust_string_to_c("{\"error\":\"no providers configured\"}".to_string()); }

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let prompt_clone2 = prompt.clone();
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(guidelines_json, providers)?;
        let results = validator.validate(&prompt_clone2).await?;
        Ok::<Vec<panther_validation::ValidationResult>, anyhow::Error>(results)
    });
    match res {
        Ok(results) => {
            let results_json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
            let ctx = panther_validation::proof::ProofContext { sdk_version: env!("CARGO_PKG_VERSION").to_string(), salt: None };
            let proof = match panther_validation::proof::compute_proof(&prompt, &providers_json, guidelines_json, &results_json, &ctx) {
                Ok(p) => p,
                Err(e) => return rust_string_to_c(format!("{{\"error\":\"compute proof failed: {}\"}}", e)),
            };
            let out = serde_json::json!({ "results": results, "proof": proof });
            rust_string_to_c(serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_string()))
        }
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

// (Removed duplicate panther_validation_run_custom_with_proof implementation)

#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_proof_compute(
    prompt_c: *const c_char,
    providers_json_c: *const c_char,
    guidelines_json_c: *const c_char,
    results_json_c: *const c_char,
    salt_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let providers_json = unsafe { CStr::from_ptr(providers_json_c).to_string_lossy().into_owned() };
    let guidelines_json = unsafe { CStr::from_ptr(guidelines_json_c).to_string_lossy().into_owned() };
    let results_json = unsafe { CStr::from_ptr(results_json_c).to_string_lossy().into_owned() };
    let salt = unsafe { if salt_c.is_null() { None } else { Some(CStr::from_ptr(salt_c).to_string_lossy().into_owned()) } };
    let ctx = panther_validation::proof::ProofContext { sdk_version: env!("CARGO_PKG_VERSION").to_string(), salt };
    let proof = panther_validation::proof::compute_proof(&prompt, &providers_json, &guidelines_json, &results_json, &ctx);
    match proof {
        Ok(p) => rust_string_to_c(serde_json::to_string(&p).unwrap_or_else(|_| "{}".to_string())),
        Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)),
    }
}

#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_proof_verify_local(
    prompt_c: *const c_char,
    providers_json_c: *const c_char,
    guidelines_json_c: *const c_char,
    results_json_c: *const c_char,
    salt_c: *const c_char,
    proof_json_c: *const c_char,
) -> i32 {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let providers_json = unsafe { CStr::from_ptr(providers_json_c).to_string_lossy().into_owned() };
    let guidelines_json = unsafe { CStr::from_ptr(guidelines_json_c).to_string_lossy().into_owned() };
    let results_json = unsafe { CStr::from_ptr(results_json_c).to_string_lossy().into_owned() };
    let salt = unsafe { if salt_c.is_null() { None } else { Some(CStr::from_ptr(salt_c).to_string_lossy().into_owned()) } };
    let proof_json = unsafe { CStr::from_ptr(proof_json_c).to_string_lossy().into_owned() };
    let parsed: Result<panther_validation::proof::Proof, _> = serde_json::from_str(&proof_json);
    if let Ok(proof) = parsed {
        let ok = panther_validation::proof::verify_proof_local(&proof, &prompt, &providers_json, &guidelines_json, &results_json, salt);
        return if ok { 1 } else { 0 };
    }
    0
}

// ---------- Blockchain (optional) ----------
#[cfg(feature = "blockchain-eth")]
#[no_mangle]
pub extern "C" fn panther_proof_anchor_eth(
    proof_hash_hex_c: *const c_char,
    rpc_url_c: *const c_char,
    contract_addr_c: *const c_char,
    privkey_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let proof_hash_hex = unsafe { CStr::from_ptr(proof_hash_hex_c).to_string_lossy().into_owned() };
    let rpc_url = unsafe { CStr::from_ptr(rpc_url_c).to_string_lossy().into_owned() };
    let contract_addr = unsafe { CStr::from_ptr(contract_addr_c).to_string_lossy().into_owned() };
    let privkey = unsafe { CStr::from_ptr(privkey_c).to_string_lossy().into_owned() };

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let res = rt.block_on(async move {
        let result = panther_validation::anchor_eth::anchor_proof(&proof_hash_hex, &rpc_url, &contract_addr, &privkey).await?;
        Ok::<String, anyhow::Error>(serde_json::to_string(&result)? )
    });
    match res { Ok(s) => rust_string_to_c(s), Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)), }
}

#[cfg(feature = "blockchain-eth")]
#[no_mangle]
pub extern "C" fn panther_proof_check_eth(
    proof_hash_hex_c: *const c_char,
    rpc_url_c: *const c_char,
    contract_addr_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let proof_hash_hex = unsafe { CStr::from_ptr(proof_hash_hex_c).to_string_lossy().into_owned() };
    let rpc_url = unsafe { CStr::from_ptr(rpc_url_c).to_string_lossy().into_owned() };
    let contract_addr = unsafe { CStr::from_ptr(contract_addr_c).to_string_lossy().into_owned() };

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let res = rt.block_on(async move {
        let anchored = panther_validation::anchor_eth::is_anchored(&proof_hash_hex, &rpc_url, &contract_addr).await?;
        Ok::<String, anyhow::Error>(serde_json::to_string(&serde_json::json!({"anchored": anchored}))? )
    });
    match res { Ok(s) => rust_string_to_c(s), Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)), }
}
#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_validation_run_openai(
    prompt_c: *const c_char,
    api_key_c: *const c_char,
    model_c: *const c_char,
    base_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let api_key = unsafe { CStr::from_ptr(api_key_c).to_string_lossy().into_owned() };
    let model = unsafe { CStr::from_ptr(model_c).to_string_lossy().into_owned() };
    let base = unsafe { CStr::from_ptr(base_c).to_string_lossy().into_owned() };
    let guidelines_json: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../panther-validation/guidelines/anvisa.json"));

    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    #[cfg(feature = "validation-openai")]
    {
        let p = panther_providers::openai::OpenAiProvider { api_key, model: model.clone(), base_url: base };
        providers.push((format!("openai:{}", model), Arc::new(p)));
    }
    if providers.is_empty() {
        return rust_string_to_c("{\"error\":\"openai feature not enabled\"}".to_string());
    }

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(guidelines_json, providers)?;
        let results = validator.validate(&prompt).await?;
        Ok::<String, anyhow::Error>(serde_json::to_string(&results)? )
    });
    match res { Ok(s) => rust_string_to_c(s), Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)), }
}

#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_validation_run_ollama(
    prompt_c: *const c_char,
    base_c: *const c_char,
    model_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let base = unsafe { CStr::from_ptr(base_c).to_string_lossy().into_owned() };
    let model = unsafe { CStr::from_ptr(model_c).to_string_lossy().into_owned() };
    let guidelines_json: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../panther-validation/guidelines/anvisa.json"));

    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    #[cfg(feature = "validation-ollama")]
    {
        let p = panther_providers::ollama::OllamaProvider { base_url: base, model: model.clone() };
        providers.push((format!("ollama:{}", model), Arc::new(p)));
    }
    if providers.is_empty() {
        return rust_string_to_c("{\"error\":\"ollama feature not enabled\"}".to_string());
    }

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(guidelines_json, providers)?;
        let results = validator.validate(&prompt).await?;
        Ok::<String, anyhow::Error>(serde_json::to_string(&results)? )
    });
    match res { Ok(s) => rust_string_to_c(s), Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)), }
}

#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_validation_run_multi(
    prompt_c: *const c_char,
    providers_json_c: *const c_char,
) -> *mut std::os::raw::c_char {
    #[derive(serde::Deserialize)]
    struct Cfg {
        #[serde(rename = "type")] ty: String,
        #[serde(default)] api_key: Option<String>,
        base_url: Option<String>,
        model: Option<String>,
    }

    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let cfg_json = unsafe { CStr::from_ptr(providers_json_c).to_string_lossy().into_owned() };
    let guidelines_json: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../panther-validation/guidelines/anvisa.json"));

    let cfgs: Result<Vec<Cfg>, _> = serde_json::from_str(&cfg_json);
    if let Err(e) = cfgs {
        return rust_string_to_c(format!("{{\"error\":\"providers json invalid: {}\"}}", e));
    }
    let cfgs = cfgs.unwrap();

    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    for c in cfgs {
        match c.ty.as_str() {
            #[cfg(feature = "validation-openai")]
            "openai" => {
                if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                    let base = c.base_url.unwrap_or_else(|| "https://api.openai.com".to_string());
                    let p = panther_providers::openai::OpenAiProvider { api_key: key, model: model.clone(), base_url: base };
                    providers.push((format!("openai:{}", model), Arc::new(p)));
                }
            }
            #[cfg(feature = "validation-ollama")]
            "ollama" => {
                if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                    let p = panther_providers::ollama::OllamaProvider { base_url: base, model: model.clone() };
                    providers.push((format!("ollama:{}", model), Arc::new(p)));
                }
            }
            _ => {}
        }
    }
    if providers.is_empty() { return rust_string_to_c("{\"error\":\"no providers configured\"}".to_string()); }

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(guidelines_json, providers)?;
        let results = validator.validate(&prompt).await?;
        Ok::<String, anyhow::Error>(serde_json::to_string(&results)? )
    });
    match res { Ok(s) => rust_string_to_c(s), Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)), }
}

#[cfg(feature = "validation")]
#[no_mangle]
pub extern "C" fn panther_validation_run_custom(
    prompt_c: *const c_char,
    providers_json_c: *const c_char,
    guidelines_json_c: *const c_char,
) -> *mut std::os::raw::c_char {
    let prompt = unsafe { CStr::from_ptr(prompt_c).to_string_lossy().into_owned() };
    let providers_json = unsafe { CStr::from_ptr(providers_json_c).to_string_lossy().into_owned() };
    let guidelines_json = unsafe { CStr::from_ptr(guidelines_json_c).to_string_lossy().into_owned() };

    let cfgs: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&providers_json);
    if let Err(e) = cfgs {
        return rust_string_to_c(format!("{{\"error\":\"providers json invalid: {}\"}}", e));
    }
    let cfgs = cfgs.unwrap();

    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    for entry in cfgs {
        if let Some(ty) = entry.get("type").and_then(|v| v.as_str()) {
            match ty {
                #[cfg(feature = "validation-openai")]
                "openai" => {
                    if let (Some(base), Some(model)) = (
                        entry.get("base_url").and_then(|v| v.as_str()),
                        entry.get("model").and_then(|v| v.as_str()),
                    ) {
                        if let Some(api_key) = entry.get("api_key").and_then(|v| v.as_str()) {
                            let prov = panther_providers::openai::OpenAiProvider {
                                api_key: api_key.to_string(),
                                model: model.to_string(),
                                base_url: base.to_string(),
                            };
                            providers.push((format!("openai:{}", model), Arc::new(prov)));
                        }
                    }
                }
                #[cfg(feature = "validation-ollama")]
                "ollama" => {
                    if let (Some(base), Some(model)) = (
                        entry.get("base_url").and_then(|v| v.as_str()),
                        entry.get("model").and_then(|v| v.as_str()),
                    ) {
                        let prov = panther_providers::ollama::OllamaProvider {
                            base_url: base.to_string(),
                            model: model.to_string(),
                        };
                        providers.push((format!("ollama:{}", model), Arc::new(prov)));
                    }
                }
                #[cfg(feature = "validation-anthropic")]
                "anthropic" => {
                    if let (Some(base), Some(model)) = (
                        entry.get("base_url").and_then(|v| v.as_str()),
                        entry.get("model").and_then(|v| v.as_str()),
                    ) {
                        if let Some(api_key) = entry.get("api_key").and_then(|v| v.as_str()) {
                            let prov = panther_providers::anthropic::AnthropicProvider {
                                api_key: api_key.to_string(),
                                model: model.to_string(),
                                base_url: base.to_string(),
                                version: "2023-06-01".to_string(),
                            };
                            providers.push((format!("anthropic:{}", model), Arc::new(prov)));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if providers.is_empty() {
        return rust_string_to_c("{\"error\":\"no providers configured\"}".to_string());
    }

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(&guidelines_json, providers)?;
        let results = validator.validate(&prompt).await?;
        Ok::<String, anyhow::Error>(serde_json::to_string(&results)? )
    });
    match res { Ok(s) => rust_string_to_c(s), Err(e) => rust_string_to_c(format!("{{\"error\":\"{}\"}}", e)), }
}
// Prometheus scrape (Rust exporter)
#[cfg(feature = "metrics-prometheus")]
#[no_mangle]
pub extern "C" fn panther_prometheus_scrape() -> *mut std::os::raw::c_char {
    if let Some(prom) = PROM.get() {
        let s = prom.scrape();
        return rust_string_to_c(s);
    }
    rust_string_to_c(String::new())
}
