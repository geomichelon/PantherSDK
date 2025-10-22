import SwiftUI
import UIKit

struct SectionHeader: View {
    let title: String
    init(_ t: String) { title = t }
    var body: some View { Text(title).font(.headline) }
}

struct ModelPicker: View {
    let title: String
    let presets: [String]
    @Binding var selection: String
    var body: some View {
        VStack(alignment: .leading) {
            Text(title).font(.subheadline)
            Picker("", selection: $selection) { ForEach(presets, id: \.self) { Text($0).tag($0) } }
                .pickerStyle(.menu)
                .labelsHidden()
            TextField("Custom", text: $selection).textFieldStyle(.roundedBorder)
        }
    }
}

struct SummaryView: View {
    let rows: [ValidationRow]
    var best: ValidationRow? { rows.max(by: { $0.score < $1.score }) }
    var totalCost: Double { rows.reduce(0) { $0 + $1.costUSD } }
    var avgLatency: Int { guard !rows.isEmpty else { return 0 }; return rows.map { $0.latencyMs }.reduce(0, +) / rows.count }
    var body: some View {
        HStack {
            VStack(alignment: .leading) { Text("Best provider").font(.caption).foregroundColor(.secondary); Text(best?.provider ?? "–").font(.headline) }
            Spacer()
            VStack(alignment: .trailing) { Text("Avg latency").font(.caption).foregroundColor(.secondary); Text("\(avgLatency) ms").font(.headline) }
            Spacer()
            VStack(alignment: .trailing) { Text("Total cost").font(.caption).foregroundColor(.secondary); Text(String(format: "$ %.4f", totalCost)).font(.headline) }
        }
        .padding(8)
        .background(Color(UIColor.secondarySystemBackground))
        .cornerRadius(8)
    }
}

struct ResultCard: View {
    let row: ValidationRow
    @State private var expanded = false
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(row.provider).font(.headline)
                    if let st = row.strategy, !st.isEmpty {
                        Text(st).font(.caption2).foregroundColor(.secondary)
                    }
                }
                Spacer()
                Text(String(format: "%.1f%%", row.score))
                    .font(.subheadline)
                    .foregroundColor(row.score >= 80 ? .green : (row.score >= 60 ? .orange : .red))
            }
            HStack(spacing: 12) { Label("\(row.latencyMs) ms", systemImage: "clock"); Label("in \(row.tokensIn)", systemImage: "text.alignleft"); Label("out \(row.tokensOut)", systemImage: "text.justify"); Text(String(format: "$ %.4f", row.costUSD)).bold() }.font(.caption)
            if let coh = row.coherence, let flu = row.fluency {
                HStack(spacing: 12) {
                    Label(String(format: "coh %.2f", coh), systemImage: "text.redaction")
                    Label(String(format: "flu %.2f", flu), systemImage: "textformat.abc")
                }.font(.caption2).foregroundColor(.secondary)
            }
            HStack {
                Button(expanded ? "Hide response" : "View response") { expanded.toggle() }
                    .buttonStyle(.borderedProminent)
                    .buttonBorderShape(.capsule)
                Spacer()
                Button("Copy") { UIPasteboard.general.string = row.response }
                    .buttonStyle(.borderedProminent)
                    .buttonBorderShape(.capsule)
                    .accessibilityLabel("Copy response")
            }
            if expanded {
                Text(row.response)
                    .textSelection(.enabled)
                    .font(.system(.footnote, design: .monospaced))
                    .padding(8)
                    .background(Color(UIColor.tertiarySystemBackground))
                    .cornerRadius(6)
                    .contextMenu { Button("Copy") { UIPasteboard.general.string = row.response } }
            }
        }
        .padding(12)
        .background(Color(UIColor.secondarySystemBackground))
        .cornerRadius(10)
    }
}

// Global helper to hide the keyboard by resigning first responder
#if canImport(UIKit)
extension View {
    func hideKeyboardOnTap() -> some View {
        self.simultaneousGesture(TapGesture().onEnded {
            UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
        })
    }
}
#endif

// Inline cost rules editor used from Validate via NavigationLink
struct CostRulesInlineEditor: View {
    @Binding var costRulesJson: String
    @Environment(\.dismiss) private var dismiss
    var body: some View {
        VStack(alignment: .leading) {
            TextEditor(text: $costRulesJson)
                .font(.system(.footnote, design: .monospaced))
                .padding(8)
                .background(Color(UIColor.secondarySystemBackground))
                .cornerRadius(8)
            HStack {
                Button("Restore Default") { costRulesJson = CostRules.defaultJSON }
                    .buttonStyle(.borderedProminent)
                    .buttonBorderShape(.capsule)
                Spacer()
                Button("Close") { dismiss() }
                    .buttonStyle(.borderedProminent)
                    .buttonBorderShape(.capsule)
            }
        }
        .padding()
        .navigationTitle("Cost Table")
    }
}

// MultiProviderRow removido: UI de múltiplos LLMs foi retirada
