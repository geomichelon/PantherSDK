# Last Updates

Date: 2025-10-18

- Created a new iOS sample app project:
  - Path: samples/swift/PantherSDKSampleIOS/PantherSDKSampleIOS.xcodeproj
  - Files: samples/swift/PantherSDKSampleIOS/PantherSDKSampleIOS/App.swift, samples/swift/PantherSDKSampleIOS/PantherSDKSampleIOS/ContentView.swift
  - SwiftUI UI with modes Single/Multi/With Proof; providers OpenAI/Ollama/Default; model presets; custom guidelines JSON; cost estimation per provider; results cards and proof JSON view.
- Centralized the XCFramework location:
  - New folder: samples/swift/frameworkIOS
  - Project now references ../frameworkIOS and embeds/signs the framework
  - Renamed artifact to PantherSDKIOS.xcframework
- Fixed Xcode project references:
  - Updated group to frameworkIOS and FRAMEWORK_SEARCH_PATHS to $(PROJECT_DIR)/../frameworkIOS
  - Framework file reference renamed to PantherSDKIOS.xcframework and added to Link + Embed phases
- Built the XCFramework with validation features:
  - Command: FEATURES="metrics-inmemory storage-inmemory validation validation-openai validation-ollama" ./scripts/release/build_ios_xcframework.sh
  - Copied as: rsync -a dist/$VERSION/ios/PantherSDK.xcframework samples/swift/frameworkIOS/PantherSDKIOS.xcframework
  - Fixed Permission denied by: chmod +x scripts/release/build_ios_xcframework.sh
- Fixed bundle layout so Info.plist is at the XCFramework root:
  - Flattened PantherSDKIOS.xcframework to contain ios-arm64, ios-arm64-simulator and Info.plist at root
- Rust FFI fixes (crates/panther-ffi/src/lib.rs):
  - Removed duplicate panther_validation_run_custom_with_proof implementation
  - Resolved move/borrow errors by cloning prompt/guidelines used across async closures
- Housekeeping (previous samples):
  - Cleaned samples/swift (removed older sample folders and projects)
- Documentation:
  - samples/swift/frameworkIOS/README.md (how to generate/copy PantherSDKIOS.xcframework)
  - samples/swift/PantherSDKSampleIOS/xcframework/README.md (points to the central folder)

Automation
- New helper script to rebuild and refresh the framework in the sample:
  - scripts/ios_refresh_framework.sh
  - Usage:
    - Optional: export FEATURES="metrics-inmemory storage-inmemory validation validation-openai validation-ollama"
    - Run: ./scripts/ios_refresh_framework.sh
    - Copies dist/<version>/ios/PantherSDK.xcframework into samples/swift/frameworkIOS/PantherSDKIOS.xcframework (flattened)

Sample refactor (UI + Bridge)
- New bridge module: samples/swift/PantherSDKSampleIOS/PantherSDKSampleIOS/PantherBridge.swift
  - Centralizes FFI (@_silgen_name) and network glue (proof anchor/status).
  - API: validateDefault/OpenAI/Ollama/Multi/Custom (+WithProof), tokenCount, calculateCost, biasDetect, loadGuidelinesFromURL, anchorProof, checkProofStatus.
- UI separation:
  - Components.swift (SectionHeader, ModelPicker, SummaryView, ResultCard)
  - Models.swift (ValidationRow, CostRules)
  - ContentView.swift now delegates to PantherBridge/Components.
- New features in sample screen:
  - Compliance Report: biasDetect over responses + Trust Index (avg adherence penalized por bias_score).
  - Proof actions: Anchor/Status (Python API /proof/anchor, /proof/status), auto-extracts combined_hash.
  - Load Guidelines by URL: fetch JSON and set as Custom Guidelines.
- Xcode project updated to include new files and add them to Sources.

Build/run checklist
- Generate XCFramework: see command above
- Copy to central folder: samples/swift/frameworkIOS/PantherSDKIOS.xcframework
- Open project: samples/swift/PantherSDKSampleIOS/PantherSDKSampleIOS.xcodeproj
- Ensure Link + Embed show PantherSDKIOS.xcframework
- If testing Ollama over HTTP, add ATS exceptions to the app Info.plist

Notes
- Cost shown in the sample is an estimate (token counting via simple heuristic + editable pricing JSON).
- OpenAI requires API key; set via UI fields.
