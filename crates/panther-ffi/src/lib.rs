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

static ENGINE: OnceCell<Engine> = OnceCell::new();
static LOGS: OnceCell<std::sync::Mutex<Vec<String>>> = OnceCell::new();
static STORAGE: OnceCell<Arc<dyn KeyValueStore>> = OnceCell::new();

#[no_mangle]
pub extern "C" fn panther_init() -> i32 {
    init_logging();
    let _ = LOGS.set(std::sync::Mutex::new(Vec::new()));
    let provider = Arc::new(NullProvider);
    let telemetry = Some(Arc::new(LogSink) as Arc<dyn panther_domain::ports::TelemetrySink>);
    let engine = Engine::new(provider, telemetry);

    // Conditionally augment the engine without mutable bindings by shadowing
    #[cfg(feature = "metrics-inmemory")]
    let engine = {
        let metrics = Arc::new(panther_metrics::InMemoryMetrics::default());
        engine.with_metrics(metrics)
    };
    #[cfg(not(feature = "metrics-inmemory"))]
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
        Ok((events, done, new_cursor)) => {
            let out = serde_json::json!({"events": events, "done": done, "cursor": new_cursor});
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
            _ => {}
        }
    }
    if providers.is_empty() { return rust_string_to_c("{\"error\":\"no providers configured\"}".to_string()); }
    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(_) => return rust_string_to_c("{\"error\":\"runtime init failed\"}".to_string()),
    };
    let res = rt.block_on(async move {
        let validator = panther_validation::LLMValidator::from_json_str(&guidelines_json, providers)?;
        let results = validator.validate(&prompt).await?;
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
        let validator = panther_validation::LLMValidator::from_json_str(&guidelines_json, providers)?;
        let results = validator.validate(&prompt).await?;
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
