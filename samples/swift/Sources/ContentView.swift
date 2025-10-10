import SwiftUI

struct ContentView: View {
    @State private var metrics: [String] = []
    @State private var logs: [String] = []
    @State private var prompt: String = "Explain insulin function"
    @State private var validation: [String] = []

    var body: some View {
        NavigationView {
            VStack(spacing: 12) {
                HStack {
                    Button("Record Metric") { PantherSDKBridge.recordMetric("button_press", value: 1.0) }
                    Button("Refresh Items") { metrics = PantherSDKBridge.listMetrics() }
                    Button("Get Logs") { logs = PantherSDKBridge.getRecentLogs() }
                }
                HStack {
                    TextField("Prompt", text: $prompt).textFieldStyle(.roundedBorder)
                    Button("Validate") { validation = PantherSDKBridge.validate(prompt: prompt) }
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
        .onAppear { PantherSDKBridge.initSDK() }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View { ContentView() }
}
