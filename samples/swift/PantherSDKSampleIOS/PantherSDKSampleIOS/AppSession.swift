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

    // Advanced: allow overriding providers JSON for Multi/Proof
    @Published var useAdvancedProvidersJSON: Bool = false
    @Published var advancedProvidersJSON: String = ""

    // No multi-LLM visual builder; use JSON avançado para múltiplos

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
        static let useAdv = "ps.providers.use_adv"
        static let advJson = "ps.providers.adv_json"
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
        useAdvancedProvidersJSON = d.object(forKey: K.useAdv) as? Bool ?? false
        advancedProvidersJSON = d.string(forKey: K.advJson) ?? advancedProvidersJSON
        // no-op: multi UI removed
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
        d.set(useAdvancedProvidersJSON, forKey: K.useAdv)
        d.set(advancedProvidersJSON, forKey: K.advJson)
        // no-op: multi UI removed
    }

    func reset() {
        let k = K.self
        [k.provider, k.openAIBase, k.openAIModel, k.openAIKey,
         k.ollamaBase, k.ollamaModel,
         k.anthropicBase, k.anthropicModel, k.anthropicKey,
         k.useAdv, k.advJson].forEach { d.removeObject(forKey: $0) }
        // Restore in-memory defaults
        provider = .openai
        openAIBase = "https://api.openai.com"; openAIModel = "gpt-4o-mini"; openAIKey = ""
        ollamaBase = "http://127.0.0.1:11434"; ollamaModel = "llama3"
        anthropicBase = "https://api.anthropic.com"; anthropicModel = "claude-3-5-sonnet-latest"; anthropicKey = ""
        useAdvancedProvidersJSON = false; advancedProvidersJSON = ""
        save()
    }

    func providersJSON(includeDefault: Bool = false) -> String {
        // If user enabled the advanced JSON and it's valid JSON array, return it
        if useAdvancedProvidersJSON {
            let s = advancedProvidersJSON.trimmingCharacters(in: .whitespacesAndNewlines)
            if let data = s.data(using: .utf8),
               (try? JSONSerialization.jsonObject(with: data)) as? [[String: Any]] != nil {
                return s
            }
        }
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
        let data = try? JSONSerialization.data(withJSONObject: arr)
        return String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
    }
}
