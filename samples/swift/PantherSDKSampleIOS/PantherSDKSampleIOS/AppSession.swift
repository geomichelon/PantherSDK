import Foundation
import SwiftUI

final class AppSession: ObservableObject {
    enum Provider: String, CaseIterable { case openai, ollama, anthropic, `default` }

    @Published var provider: Provider = .openai
    @Published var openAIBase: String = "https://api.openai.com"
    @Published var openAIModel: String = "gpt-4o-mini"
    @Published var openAIKey: String = ""
    @Published var ollamaBase: String = "http://127.0.0.1:11434"
    @Published var ollamaModel: String = "llama3"
    @Published var anthropicBase: String = "https://api.anthropic.com"
    @Published var anthropicModel: String = "claude-3-5-sonnet-latest"
    @Published var anthropicKey: String = ""

    private let d = UserDefaults.standard
    private enum K {
        static let provider = "ps.provider"
        static let openAIBase = "ps.openai.base"
        static let openAIModel = "ps.openai.model"
        static let openAIKey = "ps.openai.key"
        static let ollamaBase = "ps.ollama.base"
        static let ollamaModel = "ps.ollama.model"
        static let anthropicBase = "ps.anth.base"
        static let anthropicModel = "ps.anth.model"
        static let anthropicKey = "ps.anth.key"
    }

    func load() {
        if let raw = d.string(forKey: K.provider), let p = Provider(rawValue: raw) { provider = p }
        openAIBase = d.string(forKey: K.openAIBase) ?? openAIBase
        openAIModel = d.string(forKey: K.openAIModel) ?? openAIModel
        openAIKey = d.string(forKey: K.openAIKey) ?? openAIKey
        ollamaBase = d.string(forKey: K.ollamaBase) ?? ollamaBase
        ollamaModel = d.string(forKey: K.ollamaModel) ?? ollamaModel
        anthropicBase = d.string(forKey: K.anthropicBase) ?? anthropicBase
        anthropicModel = d.string(forKey: K.anthropicModel) ?? anthropicModel
        anthropicKey = d.string(forKey: K.anthropicKey) ?? anthropicKey
    }

    func save() {
        d.set(provider.rawValue, forKey: K.provider)
        d.set(openAIBase, forKey: K.openAIBase)
        d.set(openAIModel, forKey: K.openAIModel)
        d.set(openAIKey, forKey: K.openAIKey)
        d.set(ollamaBase, forKey: K.ollamaBase)
        d.set(ollamaModel, forKey: K.ollamaModel)
        d.set(anthropicBase, forKey: K.anthropicBase)
        d.set(anthropicModel, forKey: K.anthropicModel)
        d.set(anthropicKey, forKey: K.anthropicKey)
    }

    func providersJSON(includeDefault: Bool = true) -> String {
        var arr: [[String: String]] = []
        switch provider {
        case .openai:
            arr.append(["type": "openai", "api_key": openAIKey, "model": openAIModel, "base_url": openAIBase])
        case .ollama:
            arr.append(["type": "ollama", "model": ollamaModel, "base_url": ollamaBase])
        case .anthropic:
            arr.append(["type": "anthropic", "api_key": anthropicKey, "model": anthropicModel, "base_url": anthropicBase])
        case .default:
            break
        }
        if includeDefault { arr.append(["type": "default", "model": "anvisa", "base_url": ""]) }
        let data = try? JSONSerialization.data(withJSONObject: arr)
        return String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
    }
}

