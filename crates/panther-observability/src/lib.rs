use panther_domain::entities::TraceEvent;
use panther_domain::ports::TelemetrySink;
use tracing::info;

pub fn init_logging() {
    // Simple env-based subscriber for now
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

pub struct LogSink;

impl TelemetrySink for LogSink {
    fn record(&self, event: TraceEvent) {
        info!(target: "panther.telemetry", name = event.name, message = event.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use panther_domain::ports::TelemetrySink;

    #[test]
    fn log_sink_records_without_panic() {
        init_logging();
        let sink = LogSink;
        sink.record(panther_domain::entities::TraceEvent {
            name: "test".into(),
            message: "ok".into(),
            timestamp_ms: 0,
            attributes: serde_json::json!({}),
        });
        // no assertion; ensures no panic and trait fulfills
    }
}
