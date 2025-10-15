//! Metrics domain: evaluation utilities (delegates) and metrics sink re-exports.

pub use panther_metrics::InMemoryMetrics;
pub use panther_domain::ports::{MetricsSink, ContentMetrics};
use once_cell::sync::OnceCell;

static CONTENT_METRICS_IMPL: OnceCell<Box<dyn ContentMetrics>> = OnceCell::new();

fn content_metrics() -> &'static dyn ContentMetrics {
    CONTENT_METRICS_IMPL.get_or_init(|| Box::new(panther_metrics_content::DefaultContentMetrics)).as_ref()
}

pub fn set_content_metrics_impl(impl_box: Box<dyn ContentMetrics>) -> bool {
    CONTENT_METRICS_IMPL.set(impl_box).is_ok()
}

pub fn evaluate_accuracy(expected: &str, generated: &str) -> f64 { content_metrics().accuracy(expected, generated) }
pub fn evaluate_bleu(reference: &str, candidate: &str) -> f64 {
    content_metrics().bleu(reference, candidate)
}
pub fn evaluate_coherence(text: &str) -> f64 {
    content_metrics().coherence(text)
}
pub fn evaluate_diversity(samples: &[String]) -> f64 {
    content_metrics().diversity(samples)
}
pub fn evaluate_fluency(text: &str) -> f64 {
    content_metrics().fluency(text)
}
pub fn evaluate_rouge_l(reference: &str, candidate: &str) -> f64 {
    content_metrics().rouge_l(reference, candidate)
}
pub fn evaluate_fact_coverage(facts: &[String], candidate: &str) -> f64 {
    content_metrics().fact_coverage(facts, candidate)
}

pub fn evaluate_factcheck_adv(facts: &[String], candidate: &str) -> f64 {
    content_metrics().factcheck_adv(facts, candidate)
}

// Plagiarism metrics (Jaccard of n-grams)
pub fn evaluate_plagiarism(corpus: &[String], candidate: &str) -> f64 {
    content_metrics().plagiarism(corpus, candidate)
}
pub fn evaluate_plagiarism_ngram(corpus: &[String], candidate: &str, n: usize) -> f64 {
    content_metrics().plagiarism_ngram(corpus, candidate, n)
}
