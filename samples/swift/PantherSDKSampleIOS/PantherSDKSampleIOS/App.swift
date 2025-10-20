import SwiftUI

@main
struct PantherSDKSampleIOSApp: App {
    @StateObject private var session = AppSession()
    init() { _ = panther_init() }
    var body: some Scene {
        WindowGroup { RootTabs().environmentObject(session) }
    }
}

@_silgen_name("panther_init")
func panther_init() -> Int32
