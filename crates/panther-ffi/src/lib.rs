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
