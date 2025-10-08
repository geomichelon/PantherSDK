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
