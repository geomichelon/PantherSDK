//! Metrics domain: evaluation utilities (delegates) and metrics sink re-exports.

pub use panther_metrics::InMemoryMetrics;
pub use panther_domain::ports::MetricsSink;

pub fn evaluate_accuracy(expected: &str, generated: &str) -> f64 {
    panther_metrics_content::evaluate_accuracy(expected, generated)
}
pub fn evaluate_bleu(reference: &str, candidate: &str) -> f64 {
    panther_metrics_content::evaluate_bleu(reference, candidate)
}
pub fn evaluate_coherence(text: &str) -> f64 {
    panther_metrics_content::evaluate_coherence(text)
}
pub fn evaluate_diversity(samples: &[String]) -> f64 {
    panther_metrics_content::evaluate_diversity(samples)
}
pub fn evaluate_fluency(text: &str) -> f64 {
    panther_metrics_content::evaluate_fluency(text)
}
pub fn evaluate_rouge_l(reference: &str, candidate: &str) -> f64 {
    panther_metrics_content::evaluate_rouge_l(reference, candidate)
}
pub fn evaluate_fact_coverage(facts: &[String], candidate: &str) -> f64 {
    panther_metrics_content::evaluate_fact_coverage(facts, candidate)
}

pub fn evaluate_factcheck_adv(facts: &[String], candidate: &str) -> f64 {
    panther_metrics_content::evaluate_factcheck_adv_score(facts, candidate)
}

// Plagiarism metrics (Jaccard of n-grams)
pub fn evaluate_plagiarism(corpus: &[String], candidate: &str) -> f64 {
    panther_metrics_content::evaluate_plagiarism(corpus, candidate)
}
pub fn evaluate_plagiarism_ngram(corpus: &[String], candidate: &str, n: usize) -> f64 {
    panther_metrics_content::evaluate_plagiarism_ngram(corpus, candidate, n)
}
