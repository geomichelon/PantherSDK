import Foundation

final class PantherSDKBridge {
    static func initSDK() { _ = panther_init() }

    static func recordMetric(_ name: String, value: Double = 1.0) {
        _ = panther_metrics_record(name, value)
    }

    static func listMetrics() -> [String] {
        guard let ptr = panther_storage_list_metrics() else { return [] }
        let json = String(cString: ptr)
        panther_free_string(ptr)
        let data = Data(json.utf8)
        if let arr = try? JSONSerialization.jsonObject(with: data) as? [String] { return arr }
        return []
    }

    static func getRecentLogs() -> [String] {
        guard let ptr = panther_logs_get_recent() else { return [] }
        let json = String(cString: ptr)
        panther_free_string(ptr)
        let data = Data(json.utf8)
        if let arr = try? JSONSerialization.jsonObject(with: data) as? [String] { return arr }
        return []
    }

    static func validate(prompt: String) -> [String] {
        #if canImport(Foundation)
        guard let fn = dlsym(UnsafeMutableRawPointer(bitPattern: -2), "panther_validation_run_default") else {
            return ["validation not enabled in this build"]
        }
        typealias F = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
        let f = unsafeBitCast(fn, to: F.self)
        if let ptr = f(prompt) {
            let json = String(cString: ptr)
            panther_free_string(ptr)
            if let data = json.data(using: .utf8),
               let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {
                return arr.map { r in
                    let name = r["provider_name"] as? String ?? "?"
                    let score = (r["adherence_score"] as? Double) ?? 0
                    let lat = (r["latency_ms"] as? Int) ?? 0
                    return "\(name) – \(String(format: "%.1f", score))% – \(lat) ms"
                }
            }
        }
        return ["validation function unavailable"]
        #else
        return ["not supported"]
        #endif
    }
}
