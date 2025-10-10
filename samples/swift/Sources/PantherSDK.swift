import Foundation

public enum PantherSDK {

    public enum ProviderType: String, Codable {
        case openai
        case ollama
    }

    public struct LLM: Codable, Equatable {
        public var type: ProviderType
        public var baseURL: String
        public var model: String
        public var apiKey: String?

        public init(type: ProviderType, baseURL: String, model: String, apiKey: String? = nil) {
            self.type = type
            self.baseURL = baseURL
            self.model = model
            self.apiKey = apiKey
        }
    }

    public final class Client {
        private let providersJSON: String

        fileprivate init(providers: [LLM]) {
            _ = panther_init()
            let enc = JSONEncoder()
            enc.outputFormatting = [.withoutEscapingSlashes]
            let data = try? enc.encode(providers)
            self.providersJSON = String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
        }

        public func validate(prompt: String, guidelinesJSON: String? = nil) -> [String] {
            let raw = validateRawJSON(prompt: prompt, guidelinesJSON: guidelinesJSON)
            guard let data = raw.data(using: .utf8),
                  let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else {
                return [raw]
            }
            return arr.map { entry in
                let name = entry["provider_name"] as? String ?? "?"
                let score = (entry["adherence_score"] as? Double) ?? 0
                let latency = (entry["latency_ms"] as? Int) ?? 0
                return "\(name) – \(String(format: "%.1f", score))% – \(latency) ms"
            }
        }

        public func validateRawJSON(prompt: String, guidelinesJSON: String? = nil) -> String {
            if let guidelinesJSON = guidelinesJSON {
                guard let ptr = panther_validation_run_custom(prompt, providersJSON, guidelinesJSON) else {
                    return "{\"error\":\"ffi call failed\"}"
                }
                let json = String(cString: ptr)
                panther_free_string(ptr)
                return json
            } else {
                guard let ptr = panther_validation_run_multi(prompt, providersJSON) else {
                    return "{\"error\":\"ffi call failed\"}"
                }
                let json = String(cString: ptr)
                panther_free_string(ptr)
                return json
            }
        }
    }

    public static func make(llms: [LLM]) -> Client { Client(providers: llms) }

    // MARK: - Metrics & logs helpers
    public static func recordMetric(_ name: String, value: Double = 1.0) {
        _ = panther_metrics_record(name, value)
    }

    public static func listMetricNames() -> [String] {
        guard let ptr = panther_storage_list_metrics() else { return [] }
        let json = String(cString: ptr)
        panther_free_string(ptr)
        guard let data = json.data(using: .utf8),
              let arr = try? JSONSerialization.jsonObject(with: data) as? [String] else { return [] }
        return arr
    }

    public static func recentLogs() -> [String] {
        guard let ptr = panther_logs_get_recent() else { return [] }
        let json = String(cString: ptr)
        panther_free_string(ptr)
        guard let data = json.data(using: .utf8),
              let arr = try? JSONSerialization.jsonObject(with: data) as? [String] else { return [] }
        return arr
    }

    // MARK: - Guidelines cache & validation
    public enum Guidelines {
        private static let storageKey = "PantherSDK.GuidelinesJSON"
        public static let defaultJSON: String = """
\
[
  {
    \"topic\": \"Uso de medicamentos na gravidez\",
    \"expected_terms\": [
      \"consulta médica\",
      \"contraindicado\",
      \"categoria de risco\",
      \"dosagem\",
      \"efeitos colaterais\",
      \"advertência\",
      \"ANVISA\",
      \"orientação profissional\"
    ]
  }
]
"""
        public static func load() -> String {
            UserDefaults.standard.string(forKey: storageKey) ?? defaultJSON
        }
        public static func save(_ json: String) {
            UserDefaults.standard.set(json, forKey: storageKey)
        }
    }

    public static func validateGuidelineJSON(_ json: String) -> Bool {
        guard let data = json.data(using: .utf8),
              let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else { return false }
        return arr.allSatisfy { entry in
            guard let topic = entry["topic"] as? String,
                  let expected = entry["expected_terms"] as? [Any] else { return false }
            return !topic.isEmpty && expected.allSatisfy { $0 is String }
        }
    }
}
