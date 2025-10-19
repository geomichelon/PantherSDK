use panther_domain::ports::MetricsSink;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct InMemoryMetrics {
    counters: Mutex<HashMap<String, f64>>,
    histograms: Mutex<HashMap<String, Vec<f64>>>,
}

impl MetricsSink for InMemoryMetrics {
    fn inc_counter(&self, name: &str, value: f64) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_insert(0.0) += value;
    }
    fn observe_histogram(&self, name: &str, value: f64) {
        let mut hist = self.histograms.lock().unwrap();
        hist.entry(name.to_string()).or_default().push(value);
    }
}

impl InMemoryMetrics {
    pub fn counter_value(&self, name: &str) -> f64 {
        self.counters.lock().unwrap().get(name).copied().unwrap_or(0.0)
    }
    pub fn histogram_values(&self, name: &str) -> Vec<f64> {
        self.histograms.lock().unwrap().get(name).cloned().unwrap_or_default()
    }
}

// Prometheus exporter (optional)
#[cfg(feature = "prometheus-exporter")]
pub struct PrometheusMetrics {
    registry: prometheus::Registry,
    counters: Mutex<HashMap<String, prometheus::IntCounter>>, 
    histograms: Mutex<HashMap<String, prometheus::Histogram>>, 
}

#[cfg(feature = "prometheus-exporter")]
impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self { registry: prometheus::Registry::new(), counters: Mutex::new(HashMap::new()), histograms: Mutex::new(HashMap::new()) }
    }
}

#[cfg(feature = "prometheus-exporter")]
impl MetricsSink for PrometheusMetrics {
    fn inc_counter(&self, name: &str, value: f64) {
        let mut map = self.counters.lock().unwrap();
        let ctr = map.entry(name.to_string()).or_insert_with(|| {
            let c = prometheus::IntCounter::new(name.to_string(), name.to_string()).unwrap();
            self.registry.register(Box::new(c.clone())).ok();
            c
        });
        // IntCounter only supports i64 increments; round value safely
        let v = value.round() as i64;
        if v > 0 { ctr.inc_by(v as u64); }
    }
    fn observe_histogram(&self, name: &str, value: f64) {
        let mut map = self.histograms.lock().unwrap();
        let h = map.entry(name.to_string()).or_insert_with(|| {
            let opts = prometheus::HistogramOpts::new(name.to_string(), name.to_string());
            let h = prometheus::Histogram::with_opts(opts).unwrap();
            self.registry.register(Box::new(h.clone())).ok();
            h
        });
        h.observe(value);
    }
}

#[cfg(feature = "prometheus-exporter")]
impl PrometheusMetrics {
    pub fn scrape(&self) -> String {
        let metric_families = self.registry.gather();
        let mut buf = Vec::new();
        let enc = prometheus::TextEncoder::new();
        enc.encode(&metric_families, &mut buf).ok();
        String::from_utf8_lossy(&buf).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_metrics_accumulates() {
        let m = InMemoryMetrics::default();
        m.inc_counter("a", 1.0);
        m.inc_counter("a", 2.0);
        m.observe_histogram("h", 0.5);
        assert_eq!(m.counter_value("a"), 3.0);
        assert_eq!(m.histogram_values("h"), vec![0.5]);
    }
}
