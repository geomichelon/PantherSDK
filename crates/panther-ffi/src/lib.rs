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
    rust_string_to_c("0.1.0".to_string())
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
