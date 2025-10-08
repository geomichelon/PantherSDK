use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
