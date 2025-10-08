use once_cell::sync::OnceCell;
use panther_core::Engine;
use panther_domain::entities::Prompt;
use panther_domain::ports::KeyValueStore;
use panther_observability::{init_logging, LogSink};
use panther_providers::NullProvider;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::sync::Arc;

static ENGINE: OnceCell<Engine> = OnceCell::new();
static STORAGE: OnceCell<Arc<dyn KeyValueStore>> = OnceCell::new();

#[pyfunction]
fn init() -> PyResult<()> {
    init_logging();
    let provider = Arc::new(NullProvider);
    let telemetry = Some(Arc::new(LogSink) as Arc<dyn panther_domain::ports::TelemetrySink>);
    // Immutable builder via shadowing to avoid unused_mut
    let engine = Engine::new(provider, telemetry);
    #[cfg(feature = "metrics-inmemory")]
    let engine = {
        let metrics = Arc::new(panther_metrics::InMemoryMetrics::default());
        engine.with_metrics(metrics)
    };
    #[cfg(not(feature = "metrics-inmemory"))]
    let engine = engine;

    // Determine storage adapter (if any) and keep a handle for Python-level access
    let mut store_opt: Option<Arc<dyn KeyValueStore>> = None;
    #[cfg(feature = "storage-inmemory")]
    {
        store_opt = Some(Arc::new(panther_storage::InMemoryStore::default()));
    }
    #[cfg(all(not(feature = "storage-inmemory"), feature = "storage-sled"))]
    {
        let path = std::env::var("PANTHER_SLED_PATH").unwrap_or_else(|_| "./panther_db".to_string());
        if let Ok(store_impl) = panther_storage_sled::SledStore::open(&path) {
            store_opt = Some(Arc::new(store_impl));
        }
    }
    let engine = if let Some(store) = &store_opt { engine.with_storage(store.clone()) } else { engine };

    ENGINE
        .set(engine)
        .map_err(|_| PyRuntimeError::new_err("Engine already initialized"))?;
    if let Some(store) = store_opt { let _ = STORAGE.set(store); }
    Ok(())
}

#[pyfunction]
fn generate(_py: Python<'_>, prompt: &str) -> PyResult<String> {
    let engine = ENGINE
        .get()
        .ok_or_else(|| PyRuntimeError::new_err("pantherpy.init() not called"))?;
    let res = engine.generate(Prompt { text: prompt.to_string() });
    Ok(match res {
        Ok(c) => serde_json::to_string(&c).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    })
}

#[pyfunction]
fn evaluate_bleu_py(_py: Python<'_>, reference: &str, candidate: &str) -> PyResult<f64> {
    Ok(panthersdk::domain::metrics::evaluate_bleu(reference, candidate))
}

#[pyfunction]
fn get_history_py(_py: Python<'_>, metric: &str) -> PyResult<String> {
    if let Some(store) = STORAGE.get() {
        let hist = panthersdk::domain::storage::get_history(store.as_ref(), metric)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        return Ok(serde_json::to_string(&hist).unwrap_or_else(|_| "[]".to_string()));
    }
    Ok("[]".to_string())
}

#[pyfunction]
fn detect_bias_py(_py: Python<'_>, samples: Vec<String>) -> PyResult<String> {
    let rep = panthersdk::domain::bias::detect_bias(&samples);
    Ok(serde_json::to_string(&rep).unwrap_or_else(|_| "{}".to_string()))
}

#[pymodule]
fn pantherpy(m: &pyo3::prelude::Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_bleu_py, m)?)?;
    m.add_function(wrap_pyfunction!(get_history_py, m)?)?;
    m.add_function(wrap_pyfunction!(detect_bias_py, m)?)?;
    Ok(())
}
