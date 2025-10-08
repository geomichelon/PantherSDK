import SwiftUI

struct ContentView: View {
    @State private var logs: [String] = []
    @State private var metrics: [String] = []

    var body: some View {
        VStack(spacing: 12) {
            Text("PantherSDK Swift Sample").font(.headline)
            HStack {
                Button("Record Metric") { recordMetric() }
                Button("Refresh Items") { refreshMetrics() }
                Button("Get Logs") { refreshLogs() }
            }
            List(metrics, id: \.self) { item in Text(item) }
            Text("Logs:")
            ScrollView { VStack(alignment: .leading) { ForEach(logs, id: \.self) { Text($0) } } }
        }
        .onAppear { PantherSDKBridge.initSDK() }
        .padding()
    }

    func recordMetric() { PantherSDKBridge.recordMetric("button_press", value: 1.0) }
    func refreshMetrics() { metrics = PantherSDKBridge.listMetrics() }
    func refreshLogs() { logs = PantherSDKBridge.getLogs() }
}

@main
struct SampleApp: App {
    var body: some Scene {
        WindowGroup { ContentView() }
    }
}
