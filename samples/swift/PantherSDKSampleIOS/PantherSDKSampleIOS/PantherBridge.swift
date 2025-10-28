import Foundation
import Darwin

enum PantherBridge {
    // MARK: - Public API
    static func tokenCount(_ s: String) -> Int {
        s.withCString { Int(panther_token_count($0)) }
    }

    static func calculateCost(tokensIn: Int, tokensOut: Int, provider: String, rules: String) -> Double {
        provider.withCString { pn in rules.withCString { r in panther_calculate_cost(Int32(tokensIn), Int32(tokensOut), pn, r) } }
    }

    static func validateDefault(prompt: String) -> String { str { panther_validation_run_default($0) }(prompt) }

    static func validateOpenAI(prompt: String, apiKey: String, model: String, base: String) -> String {
        prompt.withCString { p in apiKey.withCString { a in model.withCString { m in base.withCString { b in strPtr(panther_validation_run_openai(p,a,m,b)) } } } }
    }

    static func validateOllama(prompt: String, base: String, model: String) -> String {
        prompt.withCString { p in base.withCString { b in model.withCString { m in strPtr(panther_validation_run_ollama(p,b,m)) } } }
    }

    static func validateMulti(prompt: String, providersJSON: String) -> String {
        prompt.withCString { p in providersJSON.withCString { j in strPtr(panther_validation_run_multi(p,j)) } }
    }

    static func validateMultiWithProof(prompt: String, providersJSON: String) -> String {
        prompt.withCString { p in providersJSON.withCString { j in strPtr(panther_validation_run_multi_with_proof(p,j)) } }
    }

    static func validateCustom(prompt: String, providersJSON: String, guidelinesJSON: String) -> String {
        prompt.withCString { p in providersJSON.withCString { j in guidelinesJSON.withCString { g in strPtr(panther_validation_run_custom(p,j,g)) } } }
    }

    static func validateCustomWithProof(prompt: String, providersJSON: String, guidelinesJSON: String) -> String {
        prompt.withCString { p in providersJSON.withCString { j in guidelinesJSON.withCString { g in strPtr(panther_validation_run_custom_with_proof(p,j,g)) } } }
    }

    static func biasDetect(samples: [String]) -> String {
        guard let data = try? JSONSerialization.data(withJSONObject: samples),
              let json = String(data: data, encoding: .utf8) else { return "{}" }
        return json.withCString { j in strPtr(panther_bias_detect(j)) }
    }

    static func biasDetectNeutralBLEU(samples: [String], neutralReference: String, weightDispersion: Double = 0.5) -> String {
        guard let data = try? JSONSerialization.data(withJSONObject: samples),
              let json = String(data: data, encoding: .utf8) else { return "{}" }
        return json.withCString { j in neutralReference.withCString { n in
            if let out = PantherBridge._dyn_bias_detect_neutral_bleu(j, n, weightDispersion) {
                return strPtr(out)
            } else {
                // Fallback to baseline bias detection when the new symbol isn't present in the framework
                return strPtr(panther_bias_detect(j))
            }
        } }
    }

    // Dynamic lookup to avoid link errors when the framework doesn't yet export the new symbol
    private static func _dyn_bias_detect_neutral_bleu(
        _ samplesJson: UnsafePointer<CChar>,
        _ neutralReference: UnsafePointer<CChar>,
        _ weightDispersion: Double
    ) -> UnsafeMutablePointer<CChar>? {
        typealias Fn = @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>, Double) -> UnsafeMutablePointer<CChar>?
        // Use RTLD_DEFAULT to search all loaded images for the symbol
        let handle = UnsafeMutableRawPointer(bitPattern: -2) // RTLD_DEFAULT
        guard let sym = dlsym(handle, "panther_bias_detect_neutral_bleu") else { return nil }
        let fn = unsafeBitCast(sym, to: Fn.self)
        return fn(samplesJson, neutralReference, weightDispersion)
    }

    // MARK: - Metrics API
    static func metricsBLEU(reference: String, candidate: String) -> Double {
        reference.withCString { r in candidate.withCString { c in panther_metrics_bleu(r, c) } }
    }

    static func metricsROUGEL(reference: String, candidate: String) -> Double {
        reference.withCString { r in candidate.withCString { c in panther_metrics_rouge_l(r, c) } }
    }

    static func metricsCoherence(text: String) -> Double {
        text.withCString { t in panther_metrics_coherence(t) }
    }

    static func metricsFluency(text: String) -> Double {
        text.withCString { t in panther_metrics_fluency(t) }
    }

    static func metricsAccuracy(expected: String, generated: String) -> Double {
        expected.withCString { e in generated.withCString { g in panther_metrics_accuracy(e, g) } }
    }

    static func metricsDiversity(samples: [String]) -> Double {
        guard let data = try? JSONSerialization.data(withJSONObject: samples),
              let json = String(data: data, encoding: .utf8) else { return 0.0 }
        return json.withCString { j in panther_metrics_diversity(j) }
    }

    static func metricsFactCoverage(facts: [String], candidate: String) -> Double {
        guard let data = try? JSONSerialization.data(withJSONObject: facts),
              let json = String(data: data, encoding: .utf8) else { return 0.0 }
        return json.withCString { f in candidate.withCString { c in panther_metrics_fact_coverage(f, c) } }
    }

    static func metricsFactCheckAdv(facts: [String], candidate: String) -> Double {
        guard let data = try? JSONSerialization.data(withJSONObject: facts),
              let json = String(data: data, encoding: .utf8) else { return 0.0 }
        return json.withCString { f in candidate.withCString { c in panther_metrics_factcheck_adv(f, c) } }
    }

    static func metricsPlagiarism(corpus: [String], candidate: String) -> Double {
        guard let data = try? JSONSerialization.data(withJSONObject: corpus),
              let json = String(data: data, encoding: .utf8) else { return 0.0 }
        return json.withCString { c in candidate.withCString { ca in panther_metrics_plagiarism(c, ca) } }
    }

    static func metricsPlagiarismNgram(corpus: [String], candidate: String, ngram: Int32) -> Double {
        guard let data = try? JSONSerialization.data(withJSONObject: corpus),
              let json = String(data: data, encoding: .utf8) else { return 0.0 }
        return json.withCString { c in candidate.withCString { ca in panther_metrics_plagiarism_ngram(c, ca, ngram) } }
    }

    static func loadGuidelinesFromURL(_ url: String, completion: @escaping (String?) -> Void) {
        guard let u = URL(string: url) else { completion(nil); return }
        URLSession.shared.dataTask(with: u) { data, _, _ in
            guard let data = data, let s = String(data: data, encoding: .utf8) else { completion(nil); return }
            completion(s)
        }.resume()
    }

    static func anchorProof(apiBase: String, hash: String, completion: @escaping (String) -> Void) {
        guard let base = URL(string: apiBase) else { return }
        var req = URLRequest(url: base.appendingPathComponent("proof/anchor"))
        req.httpMethod = "POST"
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try? JSONSerialization.data(withJSONObject: ["hash": hash])
        URLSession.shared.dataTask(with: req) { data, _, _ in
            let s = data.flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
            completion(s)
        }.resume()
    }

    static func checkProofStatus(apiBase: String, hash: String, completion: @escaping (String) -> Void) {
        guard var comps = URLComponents(string: apiBase) else { return }
        comps.path = "/proof/status"; comps.queryItems = [URLQueryItem(name: "hash", value: hash)]
        guard let url = comps.url else { return }
        URLSession.shared.dataTask(with: url) { data, _, _ in
            let s = data.flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
            completion(s)
        }.resume()
    }

    // MARK: - Helpers
    private static func str(_ f: @escaping (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?) -> (String) -> String {
        { input in input.withCString { c in strPtr(f(c)) } }
    }

    private static func strPtr(_ ptr: UnsafeMutablePointer<CChar>?) -> String {
        guard let ptr else { return "" }
        let s = String(cString: ptr)
        panther_free_string(ptr)
        return s
    }
}

// MARK: - FFI C symbols
@_silgen_name("panther_free_string")
private func panther_free_string(_ s: UnsafeMutablePointer<CChar>?)
@_silgen_name("panther_token_count")
private func panther_token_count(_ text: UnsafePointer<CChar>) -> Int32
@_silgen_name("panther_calculate_cost")
private func panther_calculate_cost(_ tokensIn: Int32, _ tokensOut: Int32, _ providerName: UnsafePointer<CChar>, _ costRulesJson: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_validation_run_default")
private func panther_validation_run_default(_ prompt: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_validation_run_openai")
private func panther_validation_run_openai(_ prompt: UnsafePointer<CChar>, _ apiKey: UnsafePointer<CChar>, _ model: UnsafePointer<CChar>, _ baseUrl: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_validation_run_ollama")
private func panther_validation_run_ollama(_ prompt: UnsafePointer<CChar>, _ baseUrl: UnsafePointer<CChar>, _ model: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_validation_run_multi")
private func panther_validation_run_multi(_ prompt: UnsafePointer<CChar>, _ providersJson: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_validation_run_multi_with_proof")
private func panther_validation_run_multi_with_proof(_ prompt: UnsafePointer<CChar>, _ providersJson: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_validation_run_custom")
private func panther_validation_run_custom(_ prompt: UnsafePointer<CChar>, _ providersJson: UnsafePointer<CChar>, _ guidelinesJson: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_validation_run_custom_with_proof")
private func panther_validation_run_custom_with_proof(_ prompt: UnsafePointer<CChar>, _ providersJson: UnsafePointer<CChar>, _ guidelinesJson: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_bias_detect")
private func panther_bias_detect(_ samplesJson: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

// MARK: - Metrics C symbols
@_silgen_name("panther_metrics_bleu")
private func panther_metrics_bleu(_ reference: UnsafePointer<CChar>, _ candidate: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_rouge_l")
private func panther_metrics_rouge_l(_ reference: UnsafePointer<CChar>, _ candidate: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_coherence")
private func panther_metrics_coherence(_ text: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_fluency")
private func panther_metrics_fluency(_ text: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_accuracy")
private func panther_metrics_accuracy(_ expected: UnsafePointer<CChar>, _ generated: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_diversity")
private func panther_metrics_diversity(_ samplesJson: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_fact_coverage")
private func panther_metrics_fact_coverage(_ factsJson: UnsafePointer<CChar>, _ candidate: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_factcheck_adv")
private func panther_metrics_factcheck_adv(_ factsJson: UnsafePointer<CChar>, _ candidate: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_plagiarism")
private func panther_metrics_plagiarism(_ corpusJson: UnsafePointer<CChar>, _ candidate: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_plagiarism_ngram")
private func panther_metrics_plagiarism_ngram(_ corpusJson: UnsafePointer<CChar>, _ candidate: UnsafePointer<CChar>, _ ngram: Int32) -> Double
