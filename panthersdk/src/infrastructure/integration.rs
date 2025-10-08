//! Integration helpers for Prometheus/Grafana exports.

use serde_json::json;

pub fn send_metrics_to_prometheus(metrics: &[(&str, f64)]) -> String {
    // Prometheus exposition format (text). One metric per line.
    let mut out = String::new();
    for (name, value) in metrics {
        out.push_str(&format!("{} {}\n", name, value));
    }
    out
}

pub fn export_to_grafana_json() -> String {
    // Minimal dashboard stub
    let dash = json!({
        "dashboard": {
            "title": "PantherSDK Metrics",
            "panels": [
                {"type": "graph", "title": "LLM Calls", "targets": [{"expr": "panther_generate_calls"}]}
            ]
        }
    });
    dash.to_string()
}

