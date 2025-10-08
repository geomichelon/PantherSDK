pub fn generate_tests(domain: &str, examples: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, ex) in examples.iter().enumerate() {
        out.push(format!("[{}] {} :: {}", i + 1, domain, ex));
        out.push(format!("[{}a] {} :: {}?", i + 1, domain, ex));
    }
    out
}
