import Foundation

struct ValidationRow: Identifiable {
    let id = UUID()
    let provider: String
    let score: Double
    let latencyMs: Int
    let tokensIn: Int
    let tokensOut: Int
    let costUSD: Double
    var response: String
    // Extras for strategies/metrics (optional in UI)
    let strategy: String?
    let coherence: Double?
    let fluency: Double?
}

enum CostRules {
    static let openAIModels = ["gpt-4o-mini", "gpt-4.1-mini", "gpt-4.1", "gpt-4o"]
    static let ollamaModels = ["llama3", "phi3", "mistral"]
    static let anthropicModels = ["claude-3-5-sonnet-latest", "claude-3-opus-latest", "claude-3-haiku-latest"]
    static let defaultJSON = """
    [
      {"match": "openai:gpt-4o-mini",   "usd_per_1k_in": 0.00015, "usd_per_1k_out": 0.00060},
      {"match": "openai:gpt-4.1-mini",  "usd_per_1k_in": 0.00030, "usd_per_1k_out": 0.00120},
      {"match": "openai:gpt-4.1",       "usd_per_1k_in": 0.00500, "usd_per_1k_out": 0.01500},
      {"match": "openai:gpt-4o",        "usd_per_1k_in": 0.00500, "usd_per_1k_out": 0.01500},
      {"match": "ollama:llama3",        "usd_per_1k_in": 0.00, "usd_per_1k_out": 0.00},
      {"match": "ollama:phi3",          "usd_per_1k_in": 0.00, "usd_per_1k_out": 0.00},
      {"match": "ollama:mistral",       "usd_per_1k_in": 0.00, "usd_per_1k_out": 0.00}
    ]
    """
}

struct SimilarityRow: Identifiable {
    let id = UUID()
    let topic: String
    let score: Double
    let bow: Double?
    let jaccard: Double?
}
