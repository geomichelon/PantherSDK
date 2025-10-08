//! Storage domain: key-value backed metrics storage helpers and re-exports.

use anyhow::Result;
use panther_domain::ports::KeyValueStore;
use serde::{Deserialize, Serialize};

pub use panther_storage::InMemoryStore;
pub use panther_storage_sled::SledStore;

const INDEX_KEY: &str = "panther.metrics.index";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub name: String,
    pub value: f64,
    pub timestamp_ms: i64,
}

pub fn save_metric(store: &dyn KeyValueStore, name: &str, value: f64, timestamp_ms: i64) -> Result<()> {
    let key = format!("metric:{}", name);
    let mut hist: Vec<MetricSample> = store
        .get(&key)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    hist.push(MetricSample { name: name.to_string(), value, timestamp_ms });
    store.set(&key, serde_json::to_string(&hist)?)?;
    // update index
    let mut idx: Vec<String> = store
        .get(INDEX_KEY)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    if !idx.iter().any(|m| m == name) { idx.push(name.to_string()); }
    store.set(INDEX_KEY, serde_json::to_string(&idx)?)?;
    Ok(())
}

pub fn get_history(store: &dyn KeyValueStore, metric: &str) -> Result<Vec<MetricSample>> {
    let key = format!("metric:{}", metric);
    Ok(store
        .get(&key)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default())
}

pub fn export_metrics(store: &dyn KeyValueStore, format: &str) -> Result<String> {
    let idx: Vec<String> = store
        .get(INDEX_KEY)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let mut all: Vec<MetricSample> = Vec::new();
    for m in idx {
        all.extend(get_history(store, &m)?);
    }
    match format.to_lowercase().as_str() {
        "json" => Ok(serde_json::to_string(&all)?),
        "csv" => {
            let mut wtr = String::from("name,value,timestamp_ms\n");
            for s in &all {
                wtr.push_str(&format!("{},{},{}\n", s.name, s.value, s.timestamp_ms));
            }
            Ok(wtr)
        }
        _ => Ok(String::new()),
    }
}

pub fn list_metrics(store: &dyn KeyValueStore) -> Result<Vec<String>> {
    let idx: Vec<String> = store
        .get(INDEX_KEY)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Ok(idx)
}
