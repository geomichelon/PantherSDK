import Foundation

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

<<<<<<< HEAD
    // MARK: - Content Metrics (BLEU/Coherence/Fluency)
    static func coherence(_ text: String) -> Double { text.withCString { t in panther_metrics_coherence(t) } }
    static func fluency(_ text: String) -> Double { text.withCString { t in panther_metrics_fluency(t) } }

    static func loadGuidelinesFromURL(_ url: String, completion: @escaping (String?) -> Void) {
        // Support http(s), and naive s3:// / gs:// mapping to public endpoints
        func mapSpecial(_ s: String) -> URL? {
            if s.hasPrefix("s3://") {
                // s3://bucket/key -> https://bucket.s3.amazonaws.com/key
                let rest = s.dropFirst(5)
                if let slash = rest.firstIndex(of: "/") {
                    let bucket = rest[..<slash]
                    let key = rest[slash...].dropFirst()
                    return URL(string: "https://\(bucket).s3.amazonaws.com/\(key)")
                }
            }
            if s.hasPrefix("gs://") {
                // gs://bucket/key -> https://storage.googleapis.com/bucket/key
                let rest = s.dropFirst(5)
                if let slash = rest.firstIndex(of: "/") {
                    let bucket = rest[..<slash]
                    let key = rest[slash...].dropFirst()
                    return URL(string: "https://storage.googleapis.com/\(bucket)/\(key)")
                }
            }
            return URL(string: s)
        }
        guard let u = mapSpecial(url) else { completion(nil); return }
=======
    static func loadGuidelinesFromURL(_ url: String, completion: @escaping (String?) -> Void) {
        guard let u = URL(string: url) else { completion(nil); return }
>>>>>>> origin/main
        URLSession.shared.dataTask(with: u) { data, _, _ in
            guard let data = data, let s = String(data: data, encoding: .utf8) else { completion(nil); return }
            completion(s)
        }.resume()
    }

<<<<<<< HEAD
    // MARK: - Guidelines Similarity
    static func guidelinesIngest(json: String) -> Int {
        json.withCString { j in Int(panther_guidelines_ingest_json(j)) }
    }
    static func guidelinesScores(query: String, topK: Int = 5, method: String = "bow") -> String {
        query.withCString { q in method.withCString { m in strPtr(panther_guidelines_similarity(q, Int32(topK), m)) } }
    }
    static func guidelinesSave(name: String, json: String) -> Int {
        name.withCString { n in json.withCString { j in Int(panther_guidelines_save_json(n, j)) } }
    }
    static func guidelinesLoad(name: String) -> Int {
        name.withCString { n in Int(panther_guidelines_load(n)) }
    }
    static func guidelinesBuildEmbeddings(method: String) -> Int {
        method.withCString { m in Int(panther_guidelines_embeddings_build(m)) }
    }

=======
>>>>>>> origin/main
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

<<<<<<< HEAD
@_silgen_name("panther_metrics_coherence")
private func panther_metrics_coherence(_ text: UnsafePointer<CChar>) -> Double
@_silgen_name("panther_metrics_fluency")
private func panther_metrics_fluency(_ text: UnsafePointer<CChar>) -> Double

@_silgen_name("panther_guidelines_ingest_json")
private func panther_guidelines_ingest_json(_ guidelinesJson: UnsafePointer<CChar>) -> Int32
@_silgen_name("panther_guidelines_similarity")
private func panther_guidelines_similarity(_ query: UnsafePointer<CChar>, _ topK: Int32, _ method: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("panther_guidelines_save_json")
private func panther_guidelines_save_json(_ name: UnsafePointer<CChar>, _ json: UnsafePointer<CChar>) -> Int32
@_silgen_name("panther_guidelines_load")
private func panther_guidelines_load(_ name: UnsafePointer<CChar>) -> Int32
@_silgen_name("panther_guidelines_embeddings_build")
private func panther_guidelines_embeddings_build(_ method: UnsafePointer<CChar>) -> Int32
=======
>>>>>>> origin/main
