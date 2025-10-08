PantherSDK Samples

This folder contains IDE-openable samples for each platform that demonstrate chat-like interactions with metrics, storage, and logs using the Rust FFI.

Swift (iOS Simulator)
- Path: samples/swift
- Project generator: XcodeGen (`project.yml`).
- UI: Buttons for Record Metric, Refresh Items (lists metric names), and Get Logs.
- Steps:
  1) `rustup target add aarch64-apple-ios-sim`
  2) `cargo build -p panther-ffi --features "metrics-inmemory storage-inmemory" --release --target aarch64-apple-ios-sim`
  3) `cbindgen --crate panther-ffi --output bindings/include/panther.h`
  4) `cd samples/swift && ./GENERATE_PROJECT.sh` and open the generated `.xcodeproj`.

Android (Kotlin JNI, Emulator)
- Path: samples/kotlin/android
- Build files: Kotlin DSL (`build.gradle.kts`, `settings.gradle.kts`).
- UI: Buttons for Record Metric, List Items (metric names), and Get Logs.
- Steps:
  1) `cargo install cargo-ndk`
  2) `rustup target add x86_64-linux-android`
  3) `cargo ndk -t x86_64 -o samples/kotlin/android/app/src/main/jniLibs build -p panther-ffi --features "metrics-inmemory storage-inmemory" --release`
  4) `cbindgen --crate panther-ffi --output bindings/include/panther.h`
  5) Open in Android Studio and Run on an x86_64 emulator.

Flutter
- Path: samples/flutter
- Included: FFI bindings + example app sources and a generator script.
- Note: This sample uses Dart FFI directly. If you require platform channels with intermediate Swift/Kotlin layers, I can scaffold a plugin-style project on request.
- Steps:
  1) Run `samples/flutter/GENERATE_PROJECT.sh`
  2) In generated app, `flutter pub add ffi`
  3) Build and link native libs for your target as described in README.

React Native
- Path: samples/react_native
- Includes iOS Objective‑C module and Android Java + JNI wrappers with a JS module (`Panther.ts`).
- Steps:
  1) `samples/react_native/GENERATE_PROJECT.sh` to create a CLI RN project and copy scaffolding.
  2) Build platform libs and headers (as above), link iOS static lib + header, and place Android `.so` into `jniLibs`.
  3) Wire the native module (iOS add file to target; Android register in a package if needed). Use the exported JS functions to call into Rust.

