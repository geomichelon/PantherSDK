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

    static func getLogs() -> [String] {
        guard let ptr = panther_logs_get() else { return [] }
        let json = String(cString: ptr)
        panther_free_string(ptr)
        let data = Data(json.utf8)
        if let arr = try? JSONSerialization.jsonObject(with: data) as? [String] { return arr }
        return []
    }
}

