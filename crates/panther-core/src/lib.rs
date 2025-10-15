use panther_domain::entities::{Completion, Prompt};
use panther_domain::ports::{KeyValueStore, LlmProvider, LlmProviderAsync, MetricsSink, TelemetrySink};
use std::sync::Arc;
use tracing::info;

pub struct Engine {
    provider: Arc<dyn LlmProvider>,
    provider_async: Option<Arc<dyn LlmProviderAsync>>,
    telemetry: Option<Arc<dyn TelemetrySink>>, 
    metrics: Option<Arc<dyn MetricsSink>>,
    storage: Option<Arc<dyn KeyValueStore>>,
}

impl Engine {
    pub fn new(provider: Arc<dyn LlmProvider>, telemetry: Option<Arc<dyn TelemetrySink>>) -> Self {
        Self { provider, provider_async: None, telemetry, metrics: None, storage: None }
    }

    pub fn with_metrics(mut self, metrics: Arc<dyn MetricsSink>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_storage(mut self, storage: Arc<dyn KeyValueStore>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_async_provider(mut self, provider: Arc<dyn LlmProviderAsync>) -> Self {
        self.provider_async = Some(provider);
        self
    }

    pub fn generate(&self, prompt: Prompt) -> anyhow::Result<Completion> {
        info!(target: "panther", provider = self.provider.name(), "generating");
        let start_ms = chrono::Utc::now().timestamp_millis();
        if let Some(m) = &self.metrics { m.inc_counter("panther.generate.calls", 1.0); }
        let result = self.provider.generate(&prompt);
        let end_ms = chrono::Utc::now().timestamp_millis();

        if let Some(sink) = &self.telemetry {
            if let Ok(c) = &result {
                let evt = panther_domain::entities::TraceEvent {
                    name: "completion".into(),
                    message: c.text.clone(),
                    timestamp_ms: chrono::Utc::now().timestamp_millis(),
                    attributes: serde_json::json!({"model": c.model}),
                };
                sink.record(evt);
            }
        }
        if let Ok(c) = &result {
            let latency_ms = (end_ms - start_ms).max(0) as f64;
            let input_tokens = token_count(&prompt.text) as f64;
            let output_tokens = token_count(&c.text) as f64;
            let total_tokens = input_tokens + output_tokens;

            if let Some(m) = &self.metrics {
                m.observe_histogram("panther.latency_ms", latency_ms);
                m.observe_histogram("panther.tokens.input", input_tokens);
                m.observe_histogram("panther.tokens.output", output_tokens);
                m.observe_histogram("panther.tokens.total", total_tokens);
            }

            if let Some(store) = &self.storage {
                // store last model for quick inspection
                let _ = store.set("panther.last_model", c.model.clone().unwrap_or_default());
                // append time-series samples
                let _ = save_metric(store.as_ref(), "panther.latency_ms", latency_ms, end_ms);
                let _ = save_metric(store.as_ref(), "panther.tokens.input", input_tokens, end_ms);
                let _ = save_metric(store.as_ref(), "panther.tokens.output", output_tokens, end_ms);
                let _ = save_metric(store.as_ref(), "panther.tokens.total", total_tokens, end_ms);
            }
        }
        result
    }

    pub async fn generate_async(&self, prompt: Prompt) -> anyhow::Result<Completion> {
        if let Some(p) = &self.provider_async {
            let start_ms = chrono::Utc::now().timestamp_millis();
            if let Some(m) = &self.metrics { m.inc_counter("panther.generate.calls", 1.0); }
            let result = p.generate(&prompt).await;
            let end_ms = chrono::Utc::now().timestamp_millis();

            if let Some(sink) = &self.telemetry {
                if let Ok(c) = &result {
                    let evt = panther_domain::entities::TraceEvent {
                        name: "completion".into(),
                        message: c.text.clone(),
                        timestamp_ms: chrono::Utc::now().timestamp_millis(),
                        attributes: serde_json::json!({"model": c.model}),
                    };
                    sink.record(evt);
                }
            }
            if let Ok(c) = &result {
                let latency_ms = (end_ms - start_ms).max(0) as f64;
                let input_tokens = token_count(&prompt.text) as f64;
                let output_tokens = token_count(&c.text) as f64;
                let total_tokens = input_tokens + output_tokens;

                if let Some(m) = &self.metrics {
                    m.observe_histogram("panther.latency_ms", latency_ms);
                    m.observe_histogram("panther.tokens.input", input_tokens);
                    m.observe_histogram("panther.tokens.output", output_tokens);
                    m.observe_histogram("panther.tokens.total", total_tokens);
                }

                if let Some(store) = &self.storage {
                    let _ = store.set("panther.last_model", c.model.clone().unwrap_or_default());
                    let _ = save_metric(store.as_ref(), "panther.latency_ms", latency_ms, end_ms);
                    let _ = save_metric(store.as_ref(), "panther.tokens.input", input_tokens, end_ms);
                    let _ = save_metric(store.as_ref(), "panther.tokens.output", output_tokens, end_ms);
                    let _ = save_metric(store.as_ref(), "panther.tokens.total", total_tokens, end_ms);
                }
            }
            result
        } else {
            // Fallback: run blocking provider on a blocking thread
            let provider = self.provider.clone();
            let telemetry = self.telemetry.clone();
            let metrics = self.metrics.clone();
            let storage = self.storage.clone();
            tokio::task::spawn_blocking(move || {
                generate_with_parts(provider, telemetry, metrics, storage, prompt)
            })
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("join error: {}", e)))
        }
    }

    pub fn record_metric(&self, name: &str, value: f64) {
        if let Some(m) = &self.metrics { m.inc_counter(name, value); }
    }
}

// Minimal dependency without chrono feature creep; implement tiny wrapper
mod chrono {
    pub use std::time::{SystemTime, UNIX_EPOCH};
    pub struct Utc;
    impl Utc {
        pub fn now() -> Now { Now }
    }
    pub struct Now;
    impl Now {
        pub fn timestamp_millis(&self) -> i64 {
            let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            dur.as_millis() as i64
        }
    }
}

// Runtime metrics helpers
fn token_count(s: &str) -> usize { s.split_whitespace().count() }

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct MetricSample { name: String, value: f64, timestamp_ms: i64 }

const INDEX_KEY: &str = "panther.metrics.index";

fn save_metric(store: &dyn KeyValueStore, name: &str, value: f64, timestamp_ms: i64) -> anyhow::Result<()> {
    let key = format!("metric:{}", name);
    let mut hist: Vec<MetricSample> = store
        .get(&key)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    hist.push(MetricSample { name: name.to_string(), value, timestamp_ms });
    store.set(&key, serde_json::to_string(&hist)?)?;

    let mut idx: Vec<String> = store
        .get(INDEX_KEY)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    if !idx.iter().any(|m| m == name) { idx.push(name.to_string()); }
    store.set(INDEX_KEY, serde_json::to_string(&idx)?)?;
    Ok(())
}

// Helper used by the async fallback to avoid borrowing `&self` into a 'static closure
fn generate_with_parts(
    provider: Arc<dyn LlmProvider>,
    telemetry: Option<Arc<dyn TelemetrySink>>,
    metrics: Option<Arc<dyn MetricsSink>>,
    storage: Option<Arc<dyn KeyValueStore>>,
    prompt: Prompt,
) -> anyhow::Result<Completion> {
    info!(target: "panther", provider = provider.name(), "generating");
    let start_ms = chrono::Utc::now().timestamp_millis();
    if let Some(m) = &metrics { m.inc_counter("panther.generate.calls", 1.0); }
    let result = provider.generate(&prompt);
    let end_ms = chrono::Utc::now().timestamp_millis();

    if let Some(sink) = &telemetry {
        if let Ok(c) = &result {
            let evt = panther_domain::entities::TraceEvent {
                name: "completion".into(),
                message: c.text.clone(),
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                attributes: serde_json::json!({"model": c.model}),
            };
            sink.record(evt);
        }
    }
    if let Ok(c) = &result {
        let latency_ms = (end_ms - start_ms).max(0) as f64;
        let input_tokens = token_count(&prompt.text) as f64;
        let output_tokens = token_count(&c.text) as f64;
        let total_tokens = input_tokens + output_tokens;

        if let Some(m) = &metrics {
            m.observe_histogram("panther.latency_ms", latency_ms);
            m.observe_histogram("panther.tokens.input", input_tokens);
            m.observe_histogram("panther.tokens.output", output_tokens);
            m.observe_histogram("panther.tokens.total", total_tokens);
        }

        if let Some(store) = &storage {
            let _ = store.set("panther.last_model", c.model.clone().unwrap_or_default());
            let _ = save_metric(store.as_ref(), "panther.latency_ms", latency_ms, end_ms);
            let _ = save_metric(store.as_ref(), "panther.tokens.input", input_tokens, end_ms);
            let _ = save_metric(store.as_ref(), "panther.tokens.output", output_tokens, end_ms);
            let _ = save_metric(store.as_ref(), "panther.tokens.total", total_tokens, end_ms);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use panther_domain::entities::{Completion, TraceEvent};
    use panther_domain::ports::{LlmProvider, TelemetrySink};
    use std::sync::{Arc, Mutex};

    struct MockProvider;

    impl LlmProvider for MockProvider {
        fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
            Ok(Completion { text: format!("mock: {}", prompt.text), model: Some("mock".into()) })
        }
        fn name(&self) -> &'static str { "mock" }
    }

    #[derive(Default)]
    struct MockSink { events: Mutex<Vec<TraceEvent>> }

    impl TelemetrySink for MockSink {
        fn record(&self, event: TraceEvent) { self.events.lock().unwrap().push(event); }
    }

    impl MockSink {
        fn count(&self) -> usize { self.events.lock().unwrap().len() }
    }

    #[test]
    fn engine_generates_and_records_telemetry() {
        let provider: Arc<dyn LlmProvider> = Arc::new(MockProvider);
        let sink_inner = Arc::new(MockSink::default());
        let telemetry: Option<Arc<dyn TelemetrySink>> = Some(sink_inner.clone());

        let engine = Engine::new(provider, telemetry);
        let out = engine.generate(Prompt { text: "hello".into() }).unwrap();

        assert_eq!(out.text, "mock: hello");
        assert_eq!(out.model.as_deref(), Some("mock"));
        assert_eq!(sink_inner.count(), 1, "should record one telemetry event");
    }
}
