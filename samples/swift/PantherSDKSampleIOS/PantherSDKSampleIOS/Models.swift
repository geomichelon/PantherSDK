import Foundation

struct ValidationRow: Identifiable {
    let id = UUID()
    let provider: String
    let score: Double
    let latencyMs: Int
    let tokensIn: Int
    let tokensOut: Int
    let costUSD: Double
    let response: String
}

enum CostRules {
    static let openAIModels = ["gpt-4o-mini", "gpt-4.1-mini", "gpt-4.1", "gpt-4o"]
    static let ollamaModels = ["llama3", "phi3", "mistral"]
    static let anthropicModels = ["claude-3-5-sonnet-latest", "claude-3-opus-latest", "claude-3-haiku-latest"]
    static let defaultJSON = """
    [
      {"match": "openai:gpt-4o-mini",   "usd_per_1k_in": 0.15, "usd_per_1k_out": 0.60},
      {"match": "openai:gpt-4.1-mini",  "usd_per_1k_in": 0.30, "usd_per_1k_out": 1.20},
      {"match": "openai:gpt-4.1",       "usd_per_1k_in": 5.00, "usd_per_1k_out": 15.00},
      {"match": "openai:gpt-4o",        "usd_per_1k_in": 5.00, "usd_per_1k_out": 15.00},
      {"match": "ollama:llama3",        "usd_per_1k_in": 0.00, "usd_per_1k_out": 0.00},
      {"match": "ollama:phi3",          "usd_per_1k_in": 0.00, "usd_per_1k_out": 0.00},
      {"match": "ollama:mistral",       "usd_per_1k_in": 0.00, "usd_per_1k_out": 0.00}
    ]
    """
}
