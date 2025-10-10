import Foundation

// PantherSDK: Minimal Swift facade over the Rust FFI, designed to be white‑label.
// You pass any LLM endpoints (OpenAI‑compatible or Ollama) with URL/key/model,
// and call validate(prompt:) to get adherence results ranked by score.

public enum PantherSDK {

    // ProviderType lists supported provider shapes for the white‑label flow.
    public enum ProviderType: String, Codable {
        case openai   // OpenAI‑compatible REST (OpenAI, Groq, Together, Mistral…)
        case ollama   // Local Ollama server
    }

    // LLM holds the white‑label configuration for a single endpoint.
    // For openai‑compatible providers, set apiKey+baseURL+model.
    // For Ollama, set baseURL+model (apiKey nil).
    public struct LLM: Codable, Equatable {
        public var type: ProviderType
        public var baseURL: String
        public var model: String
        public var apiKey: String? // only for openai

        public init(type: ProviderType, baseURL: String, model: String, apiKey: String? = nil) {
            self.type = type
            self.baseURL = baseURL
            self.model = model
            self.apiKey = apiKey
        }
    }

    // Client serializes LLM list once and calls the FFI as needed.
    public final class Client {
        private let providersJSON: String

        fileprivate init(providers: [LLM]) {
            // Initialize Rust core once (idempotent on the FFI side)
            _ = panther_init()
            // Serialize providers list to pass into the FFI "multi" function
            let enc = JSONEncoder()
            enc.outputFormatting = [.withoutEscapingSlashes]
            let data = try? enc.encode(providers)
            self.providersJSON = String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
        }

        // validate runs a single prompt across all configured providers and
        // returns a human‑readable list of lines with provider, score and latency.
        // To access raw JSON, use validateRawJSON.
        public func validate(prompt: String) -> [String] {
            let json = validateRawJSON(prompt: prompt)
            guard let data = json.data(using: .utf8),
                  let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else {
                return [json]
            }
            return arr.map { r in
                let name = r["provider_name"] as? String ?? "?"
                let score = (r["adherence_score"] as? Double) ?? 0
                let lat = (r["latency_ms"] as? Int) ?? 0
                return "\(name) – \(String(format: "%.1f", score))% – \(lat) ms"
            }
        }

        // validateRawJSON returns the raw JSON array of ValidationResult as String.
        public func validateRawJSON(prompt: String) -> String {
            // The FFI function takes C strings; Swift bridges automatically.
            guard let ptr = panther_validation_run_multi(prompt, providersJSON) else {
                return "{\"error\":\"ffi call failed\"}"
            }
            let s = String(cString: ptr)
            panther_free_string(ptr)
            return s
        }
    }

    // make is the single constructor you need to build the SDK client.
    // Example:
    //   let sdk = PantherSDK.make(llms:[
    //       .init(type:.openai, baseURL:"https://api.openai.com", model:"gpt-4o-mini", apiKey:"sk-...")
    //   ])
    //   let lines = sdk.validate(prompt:"Explain insulin function")
    public static func make(llms: [LLM]) -> Client { Client(providers: llms) }

    // MARK: - Telemetry & Storage helpers (white‑label)
    // Record a metric counter (e.g., button presses) directly via FFI.
    public static func recordMetric(_ name: String, value: Double = 1.0) {
        _ = panther_metrics_record(name, value)
    }

    // List known metric names from the storage backend (JSON -> [String]).
    public static func listMetricNames() -> [String] {
        guard let ptr = panther_storage_list_metrics() else { return [] }
        let json = String(cString: ptr)
        panther_free_string(ptr)
        if let data = json.data(using: .utf8), let arr = try? JSONSerialization.jsonObject(with: data) as? [String] {
            return arr
        }
        return []
    }

    // Fetch recent log lines collected in-process for demo purposes.
    public static func recentLogs() -> [String] {
        guard let ptr = panther_logs_get_recent() else { return [] }
        let json = String(cString: ptr)
        panther_free_string(ptr)
        if let data = json.data(using: .utf8), let arr = try? JSONSerialization.jsonObject(with: data) as? [String] {
            return arr
        }
        return []
    }
}
