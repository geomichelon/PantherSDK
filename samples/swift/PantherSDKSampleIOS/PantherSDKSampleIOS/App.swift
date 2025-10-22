import SwiftUI

@main
struct PantherSDKSampleIOSApp: App {
<<<<<<< HEAD
    @StateObject private var session = AppSession()
    init() { _ = panther_init() }
    var body: some Scene {
        WindowGroup { RootTabs().environmentObject(session) }
=======
    init() { _ = panther_init() }
    var body: some Scene {
        WindowGroup { ContentView() }
>>>>>>> origin/main
    }
}

@_silgen_name("panther_init")
func panther_init() -> Int32
<<<<<<< HEAD
=======

>>>>>>> origin/main
