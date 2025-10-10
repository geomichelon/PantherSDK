import SwiftUI

struct ContentView: View {
    @State private var metrics: [String] = []
    @State private var logs: [String] = []
    @State private var prompt: String = "Explain insulin function"
    @State private var validation: [String] = []
    @State private var providerIndex: Int = 0 // 0: OpenAI-like, 1: Ollama
    // OpenAI-like (white-label compatible endpoints)
    @State private var apiKey: String = ""
    @State private var model: String = "gpt-4o-mini"
    @State private var baseURL: String = "https://api.openai.com"
    // Ollama
    @State private var ollamaBase: String = "http://127.0.0.1:11434"
    @State private var ollamaModel: String = "llama3"

    var body: some View {
        NavigationView {
            VStack(spacing: 12) {
                HStack {
                    // Record a demo metric
                    Button("Record Metric") { PantherSDK.recordMetric("button_press", value: 1.0) }
                    // Read metric names from storage
                    Button("Refresh Items") { metrics = PantherSDK.listMetricNames() }
                    // Show recent in-process logs
                    Button("Get Logs") { logs = PantherSDK.recentLogs() }
                }
                HStack { TextField("Prompt", text: $prompt).textFieldStyle(.roundedBorder) }
                Picker("Provider", selection: $providerIndex) {
                    Text("OpenAI-like").tag(0)
                    Text("Ollama").tag(1)
                }.pickerStyle(.segmented)
                if providerIndex == 0 {
                    VStack(alignment: .leading) {
                        TextField("API Key", text: $apiKey).textFieldStyle(.roundedBorder)
                        TextField("Model", text: $model).textFieldStyle(.roundedBorder)
                        TextField("Base URL", text: $baseURL).textFieldStyle(.roundedBorder)
                    }
                } else {
                    VStack(alignment: .leading) {
                        TextField("Base URL", text: $ollamaBase).textFieldStyle(.roundedBorder)
                        TextField("Model", text: $ollamaModel).textFieldStyle(.roundedBorder)
                    }
                }
                Button("Validate") {
                    // Build white‑label provider list and run validation through PantherSDK
                    var llms: [PantherSDK.LLM] = []
                    if providerIndex == 0 {
                        if !apiKey.isEmpty {
                            llms.append(.init(type: .openai, baseURL: baseURL, model: model, apiKey: apiKey))
                        }
                    } else {
                        llms.append(.init(type: .ollama, baseURL: ollamaBase, model: ollamaModel))
                    }
                    let sdk = PantherSDK.make(llms: llms)
                    validation = sdk.validate(prompt: prompt)
                }
                List {
                    Section(header: Text("Metrics")) {
                        ForEach(metrics, id: \.self) { Text($0) }
                    }
                    Section(header: Text("Recent Logs")) {
                        ForEach(logs, id: \.self) { Text($0) }
                    }
                    Section(header: Text("Validation")) {
                        ForEach(validation, id: \.self) { Text($0) }
                    }
                }
            }
            .padding()
            .navigationTitle("PantherSampleSwift")
        }
        .onAppear { _ = panther_init() }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View { ContentView() }
}
