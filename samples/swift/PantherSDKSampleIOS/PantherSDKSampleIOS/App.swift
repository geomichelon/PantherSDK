import SwiftUI

@main
struct PantherSDKSampleIOSApp: App {
    init() { _ = panther_init() }
    var body: some Scene {
        WindowGroup { ContentView() }
    }
}

@_silgen_name("panther_init")
func panther_init() -> Int32

