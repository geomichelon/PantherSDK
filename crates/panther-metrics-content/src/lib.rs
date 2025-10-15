fn tokenize(s: &str) -> Vec<&str> { s.split_whitespace().collect() }

pub fn evaluate_accuracy(expected: &str, generated: &str) -> f64 {
    let e = tokenize(expected);
    let g = tokenize(generated);
    if e.is_empty() && g.is_empty() { return 1.0; }
    let n = e.len().max(g.len()) as f64;
    let matches = e.iter().zip(g.iter()).filter(|(a, b)| a == b).count() as f64;
    matches / n
}

pub fn evaluate_bleu(reference: &str, candidate: &str) -> f64 {
    // Simple BLEU-1 with brevity penalty
    let r = tokenize(reference);
    let c = tokenize(candidate);
    if c.is_empty() { return 0.0; }
    use std::collections::HashMap;
    let mut ref_counts = HashMap::new();
    for w in r { *ref_counts.entry(w).or_insert(0u32) += 1; }
    let mut match_count = 0u32;
    let mut cand_counts = HashMap::new();
    for w in c.iter() { *cand_counts.entry(*w).or_insert(0u32) += 1; }
    for (w, cc) in cand_counts { let rc = *ref_counts.get(w).unwrap_or(&0); match_count += cc.min(rc); }
    let precision = (match_count as f64) / (c.len() as f64);
    let bp = if c.len() > 0 && reference.len() > 0 {
        let r_len = tokenize(reference).len() as f64;
        let c_len = c.len() as f64;
        if c_len > r_len { 1.0 } else { (-((r_len / c_len) - 1.0)).exp() }
    } else { 0.0 };
    (precision.max(0.0).min(1.0) * bp).max(0.0).min(1.0)
}

pub fn evaluate_coherence(text: &str) -> f64 {
    // Naive coherence: penalize repeated bigrams
    let tokens = tokenize(text);
    if tokens.len() < 2 { return 1.0; }
    use std::collections::HashSet;
    let mut bigrams = HashSet::new();
    let mut repeats = 0u32;
    for i in 0..tokens.len()-1 { let bg = (tokens[i], tokens[i+1]); if !bigrams.insert(bg) { repeats += 1; } }
    1.0 - (repeats as f64 / (tokens.len().saturating_sub(1) as f64))
}

pub fn evaluate_diversity(samples: &[String]) -> f64 {
    // Type-token ratio across all samples
    use std::collections::HashSet;
    let mut types = HashSet::new();
    let mut tokens = 0usize;
    for s in samples { for t in tokenize(s) { types.insert(t.to_string()); tokens += 1; } }
    if tokens == 0 { return 0.0; }
    (types.len() as f64) / (tokens as f64)
}

pub fn evaluate_fluency(text: &str) -> f64 {
    // Naive fluency: ratio of tokens containing a vowel, capped [0,1]
    let tokens = tokenize(text);
    if tokens.is_empty() { return 0.0; }
    let vowels = ["a","e","i","o","u","A","E","I","O","U"];
    let good = tokens.iter().filter(|t| vowels.iter().any(|v| t.contains(v))).count();
    (good as f64) / (tokens.len() as f64)
}

// ROUGE-L F1 (LCS-based) — simplified implementation
pub fn evaluate_rouge_l(reference: &str, candidate: &str) -> f64 {
    let r: Vec<&str> = tokenize(reference);
    let c: Vec<&str> = tokenize(candidate);
    if r.is_empty() || c.is_empty() { return 0.0; }
    // LCS length via DP O(n*m)
    let n = r.len();
    let m = c.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in 0..n {
        for j in 0..m {
            dp[i + 1][j + 1] = if r[i] == c[j] { dp[i][j] + 1 } else { dp[i + 1][j].max(dp[i][j + 1]) };
        }
    }
    let lcs = dp[n][m] as f64;
    let prec = lcs / (m as f64);
    let rec = lcs / (n as f64);
    if prec + rec == 0.0 { 0.0 } else { (2.0 * prec * rec) / (prec + rec) }
}

// Fact-check coverage: percentage of expected facts/terms present in candidate
pub fn evaluate_fact_coverage(facts: &[String], candidate: &str) -> f64 {
    if facts.is_empty() { return 1.0; }
    let low = candidate.to_ascii_lowercase();
    let mut hits = 0usize;
    for f in facts { if !f.is_empty() && low.contains(&f.to_ascii_lowercase()) { hits += 1; } }
    (hits as f64) / (facts.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rouge_l_basic_overlap() {
        let r = "a b c";
        let c = "a x c";
        let s = evaluate_rouge_l(r, c);
        assert!(s > 0.0 && s <= 1.0, "score out of range: {}", s);
    }

    #[test]
    fn fact_coverage_counts_terms() {
        let facts = vec!["insulin".to_string(), "hba1c".to_string(), "pancreas".to_string()];
        let text = "Insulin regulates glucose. HbA1c is a marker.";
        let s = evaluate_fact_coverage(&facts, text);
        // 2 out of 3 terms
        assert!((s - (2.0/3.0)).abs() < 1e-6, "expected ~0.666, got {}", s);
    }

    #[test]
    fn coherence_repeats_penalize() {
        let good = evaluate_coherence("a b c d e");
        let bad = evaluate_coherence("a a a a a");
        assert!(good >= bad, "good {} should be >= bad {}", good, bad);
    }

    #[test]
    fn diversity_type_token_ratio() {
        let s = vec!["a a a".to_string(), "b c".to_string()];
        let d = evaluate_diversity(&s);
        assert!(d > 0.0 && d < 1.0);
    }

    #[test]
    fn fluency_vowels_present() {
        let f = evaluate_fluency("aaaa bbbb cccc");
        assert!(f > 0.0);
        let z = evaluate_fluency("");
        assert_eq!(z, 0.0);
    }

    #[test]
    fn accuracy_and_bleu_basics() {
        let acc = evaluate_accuracy("a b c", "a b d");
        assert!(acc >= 0.0 && acc <= 1.0);
        let bleu = evaluate_bleu("a b c", "a b c");
        assert!(bleu > 0.0);
    }
}
