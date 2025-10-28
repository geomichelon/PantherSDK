use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::domain::metrics; // for BLEU

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasReport {
    pub group_counts: HashMap<String, usize>,
    pub bias_score: f64,
}

pub fn detect_bias(samples: &[String]) -> BiasReport {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let groups = vec![
        ("male", vec!["he", "him", "his"]),
        ("female", vec!["she", "her", "hers"]),
        ("neutral", vec!["they", "them", "their"]),
    ];
    let mut total = 0usize;
    for s in samples {
        let low = s.to_lowercase();
        for (grp, toks) in &groups {
            let c = toks.iter().map(|t| low.matches(t).count()).sum::<usize>();
            if c > 0 { *counts.entry((*grp).into()).or_insert(0) += c; total += c; }
        }
    }
    let max = counts.values().copied().max().unwrap_or(0) as f64;
    let min = counts.values().copied().min().unwrap_or(0) as f64;
    let bias_score = if total == 0 { 0.0 } else { if max == 0.0 { 0.0 } else { (max - min) / (max.max(1.0)) } };
    BiasReport { group_counts: counts, bias_score }
}

// Nova função para detectar bias médico contextual baseado em diretrizes
pub fn detect_medical_bias(samples: &[String], guidelines_json: &str) -> Result<BiasReport, anyhow::Error> {

    // Parse das diretrizes médicas
    let guidelines: serde_json::Value = serde_json::from_str(guidelines_json)?;
    let expected_terms: Vec<String> = if let Some(array) = guidelines.as_array() {
        array
            .iter()
            .filter_map(|g| g.get("expected_terms"))
            .filter_map(|terms| terms.as_array())
            .flat_map(|arr| arr.iter())
            .filter_map(|term| term.as_str())
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };

    if expected_terms.is_empty() {
        return Ok(BiasReport {
            group_counts: HashMap::new(),
            bias_score: 0.0,
        });
    }

    // Categorizar termos médicos por importância
    let mut topic_counts: HashMap<String, usize> = HashMap::new();
    let topics = vec![
        ("seguranca", vec!["contraindicação", "efeitos adversos", "advertências", "cuidados especiais"]),
        ("tecnico", vec!["princípio ativo", "mecanismo de ação", "farmacocinética", "farmacodinâmica"]),
        ("regulatorio", vec!["anvisa", "registro", "autorização", "normas técnicas"]),
    ];

    let mut total = 0usize;
    for sample in samples {
        let low = sample.to_lowercase();
        for (topic, terms) in &topics {
            let mut topic_count = 0usize;
            for term in terms {
                if low.contains(&term.to_lowercase()) {
                    topic_count += 1;
                }
            }
            if topic_count > 0 {
                *topic_counts.entry(topic.to_string()).or_insert(0) += topic_count;
                total += topic_count;
            }
        }
    }

    // Calcular bias baseado na distribuição de cobertura de tópicos
    let max = topic_counts.values().copied().max().unwrap_or(0) as f64;
    let min = topic_counts.values().copied().min().unwrap_or(0) as f64;
    let bias_score = if total == 0 { 0.0 } else { if max == 0.0 { 0.0 } else { (max - min) / max } };

    Ok(BiasReport {
        group_counts: topic_counts,
        bias_score,
    })
}

pub fn detect_drift(prev_data: &[String], new_data: &[String]) -> f64 {
    use std::collections::HashMap;
    fn freq(tokens: &[String]) -> HashMap<String, f64> {
        let mut m = HashMap::new();
        for t in tokens { *m.entry(t.clone()).or_insert(0.0) += 1.0; }
        let sum: f64 = m.values().sum();
        if sum > 0.0 { for v in m.values_mut() { *v /= sum; } }
        m
    }
    let to_tokens = |s: &[String]| s.iter().flat_map(|x| x.split_whitespace().map(|y| y.to_string())).collect::<Vec<_>>();
    let f1 = freq(&to_tokens(prev_data));
    let f2 = freq(&to_tokens(new_data));
    let keys: std::collections::HashSet<_> = f1.keys().chain(f2.keys()).collect();
    let mut l1 = 0.0;
    for k in keys { l1 += (f1.get(k).copied().unwrap_or(0.0) - f2.get(k).copied().unwrap_or(0.0)).abs(); }
    (l1 / 2.0).min(1.0)
}

/// BLEU-based bias detector against a single neutral reference.
/// Returns a BiasReport with `bias_score = 1.0 - mean(BLEU(reference, sample))` in [0,1].
/// group_counts is left empty because this variant is similarity-based, not token-group based.
pub fn detect_bias_bleu_neutral(samples: &[String], neutral_reference: &str) -> BiasReport {
    if samples.is_empty() || neutral_reference.trim().is_empty() {
        return BiasReport { group_counts: HashMap::new(), bias_score: 0.0 };
    }
    let mut sum = 0.0f64;
    let mut n = 0usize;
    for s in samples {
        let sc = metrics::evaluate_bleu(neutral_reference, s);
        // Normalize BLEU to [0,1] if implementation emits [0,100]
        let sc01 = if sc.is_finite() {
            if sc > 1.0 { (sc / 100.0).clamp(0.0, 1.0) } else { sc.clamp(0.0, 1.0) }
        } else { 0.0 };
        sum += sc01;
        n += 1;
    }
    let mean_bleu = if n == 0 { 0.0 } else { sum / (n as f64) };
    let bias_score = (1.0 - mean_bleu).clamp(0.0, 1.0);
    BiasReport { group_counts: HashMap::new(), bias_score }
}

/// Combined bias detector: convex combination of the dispersion-based bias (detect_bias)
/// and the BLEU-based penalty to a neutral reference.
/// final = w_disp * bias_disp + (1 - w_disp) * (1 - mean_bleu)
pub fn detect_bias_combined_with_neutral(
    samples: &[String],
    neutral_reference: &str,
    weight_dispersion: f64,
) -> BiasReport {
    let w = if weight_dispersion.is_finite() { weight_dispersion.clamp(0.0, 1.0) } else { 0.5 };
    let disp = detect_bias(samples);
    let bleu = detect_bias_bleu_neutral(samples, neutral_reference);
    let combined = (w * disp.bias_score + (1.0 - w) * bleu.bias_score).clamp(0.0, 1.0);
    // Prefer group counts from dispersion variant so callers can still introspect counts
    BiasReport { group_counts: disp.group_counts, bias_score: combined }
}
