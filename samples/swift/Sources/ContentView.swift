import SwiftUI

struct ContentView: View {
    @State private var metrics: [String] = []
    @State private var logs: [String] = []
    @State private var prompt: String = "Explain insulin function"
    @State private var validation: [String] = []
    @State private var proofHash: String? = nil

    private struct ProviderPreset {
        let label: String
        let type: PantherSDK.ProviderType
        let baseURL: String
        let defaultModel: String
        let requiresKey: Bool
    }

    private let presets: [ProviderPreset] = [
        ProviderPreset(label: "OpenAI",   type: .openai, baseURL: "https://api.openai.com",                   defaultModel: "gpt-4o-mini",              requiresKey: true),
        ProviderPreset(label: "Groq",     type: .openai, baseURL: "https://api.groq.com/openai/v1",          defaultModel: "llama3-70b-8192",          requiresKey: true),
        ProviderPreset(label: "Together", type: .openai, baseURL: "https://api.together.xyz/v1",             defaultModel: "meta-llama/Meta-Llama-3.1-70B-Instruct", requiresKey: true),
        ProviderPreset(label: "Mistral",  type: .openai, baseURL: "https://api.mistral.ai",                 defaultModel: "mistral-small-latest",     requiresKey: true),
        ProviderPreset(label: "Ollama",   type: .ollama, baseURL: "http://127.0.0.1:11434",                 defaultModel: "llama3",                   requiresKey: false)
    ]

    @State private var selectedPresetIndex: Int = 0
    @State private var baseURL: String = "https://api.openai.com"
    @State private var model: String = "gpt-4o-mini"
    @State private var apiKey: String = ""

    @State private var includeLocalOllama: Bool = false
    @State private var localOllamaBase: String = "http://127.0.0.1:11434"
    @State private var localOllamaModel: String = "llama3"

    @State private var guidelinesJSON: String = PantherSDK.Guidelines.defaultJSON
    @State private var useCustomGuidelines: Bool = false
    @State private var guidelineMessage: String?

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text("Metrics & Logs")
                        .font(.headline)
                    HStack {
                        Button("Record Metric") { PantherSDK.recordMetric("button_press", value: 1.0) }
                        Button("Refresh Items") { metrics = PantherSDK.listMetricNames() }
                        Button("Get Logs") { logs = PantherSDK.recentLogs() }
                    }
                    if !metrics.isEmpty {
                        Text("Metrics:")
                        ForEach(metrics, id: \.self) { Text($0) }
                    }
                    if !logs.isEmpty {
                        Text("Logs:")
                        ForEach(logs, id: \.self) { Text($0) }
                    }

                    Divider()

                    Text("Prompt")
                        .font(.headline)
                    TextField("Prompt", text: $prompt)
                        .textFieldStyle(.roundedBorder)

                    Text("LLM Providers")
                        .font(.headline)
                    Picker("Provider", selection: $selectedPresetIndex) {
                        ForEach(0..<presets.count, id: \.self) { Text(presets[$0].label) }
                    }
                    .pickerStyle(.segmented)
                    .onChange(of: selectedPresetIndex) { applyPreset(index: $0) }

                    TextField("Base URL", text: $baseURL)
                        .textFieldStyle(.roundedBorder)
                    TextField("Model", text: $model)
                        .textFieldStyle(.roundedBorder)
                    if presets[selectedPresetIndex].requiresKey {
                        SecureField("API Key", text: $apiKey)
                            .textFieldStyle(.roundedBorder)
                    }

                    Toggle("Include local Ollama", isOn: $includeLocalOllama)
                    if includeLocalOllama {
                        TextField("Ollama Base URL", text: $localOllamaBase)
                            .textFieldStyle(.roundedBorder)
                        TextField("Ollama Model", text: $localOllamaModel)
                            .textFieldStyle(.roundedBorder)
                    }

                    Divider()

                    Toggle("Use custom guidelines", isOn: $useCustomGuidelines)
                    TextEditor(text: $guidelinesJSON)
                        .frame(minHeight: 160)
                        .border(Color.gray.opacity(0.4))
                    Button("Save Guidelines") {
                        if PantherSDK.validateGuidelineJSON(guidelinesJSON) {
                            PantherSDK.Guidelines.save(guidelinesJSON)
                            guidelineMessage = "Guidelines saved."
                        } else {
                            guidelineMessage = "Invalid JSON: expected array of {topic, expected_terms}."
                        }
                    }
                    if let message = guidelineMessage {
                        Text(message).foregroundColor(.secondary)
                    }

                    Divider()

                    Button("Validate") {
                        validatePrompt()
                    }
                    .buttonStyle(.borderedProminent)

                    if !validation.isEmpty {
                        Text("Validation Results")
                            .font(.headline)
                        ForEach(validation, id: \.self) { Text($0) }
                        if let proof = proofHash {
                            Text("Proof: \(proof)")
                                .font(.footnote)
                                .foregroundColor(.secondary)
                        }
                    }
                }
                .padding()
            }
            .navigationTitle("Panther Sample Swift")
        }
        .onAppear {
            _ = panther_init()
            guidelinesJSON = PantherSDK.Guidelines.load()
            if let index = presets.firstIndex(where: { $0.label == "OpenAI" }) {
                applyPreset(index: index)
                selectedPresetIndex = index
            }
        }
    }

    private func applyPreset(index: Int) {
        let preset = presets[index]
        baseURL = preset.baseURL
        model = preset.defaultModel
        if !preset.requiresKey { apiKey = "" }
    }

    private func validatePrompt() {
        let preset = presets[selectedPresetIndex]
        if preset.requiresKey && apiKey.isEmpty {
            validation = ["API key required for \(preset.label)"]
            return
        }

        var llms: [PantherSDK.LLM] = [
            PantherSDK.LLM(type: preset.type, baseURL: baseURL, model: model, apiKey: preset.requiresKey ? apiKey : nil)
        ]
        if includeLocalOllama {
            llms.append(.init(type: .ollama, baseURL: localOllamaBase, model: localOllamaModel))
        }

        let guidelines = useCustomGuidelines ? guidelinesJSON : nil
        let sdk = PantherSDK.make(llms: llms)
        // Prefer validation with proof when using default guidelines
        let raw = sdk.validateWithProofRawJSON(prompt: prompt, guidelinesJSON: useCustomGuidelines ? guidelines : nil)
        if let data = raw.data(using: .utf8),
           let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
           let arr = obj["results"] as? [[String: Any]] {
            validation = arr.map { entry in
                let name = entry["provider_name"] as? String ?? "?"
                let score = (entry["adherence_score"] as? Double) ?? 0
                let latency = (entry["latency_ms"] as? Int) ?? 0
                return "\(name) – \(String(format: "%.1f", score))% – \(latency) ms"
            }
            if let proof = obj["proof"] as? [String: Any], let ch = proof["combined_hash"] as? String {
                proofHash = ch
            } else {
                proofHash = nil
            }
        } else {
            // fallback to previous behavior
            validation = sdk.validate(prompt: prompt, guidelinesJSON: guidelines)
            proofHash = nil
        }
    }
}
