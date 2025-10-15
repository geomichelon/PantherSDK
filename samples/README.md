PantherSDK Samples

This folder contains IDE-openable samples for each platform that demonstrate chat-like interactions with metrics, storage, and logs using the Rust FFI.

Swift (iOS Simulator)
- Path: samples/swift
- Project generator: XcodeGen (`project.yml`).
- UI: Buttons for Record Metric, Refresh Items (lists metric names), Get Logs, and Validation.
- Validation: White‑label — pick provider type and enter Base URL / Model / API Key (for OpenAI‑like). App calls a single FFI `panther_validation_run_multi` via a Swift façade (`PantherSDK`).
- Steps:
  1) `rustup target add aarch64-apple-ios-sim`
  2) `cargo build -p panther-ffi --features "metrics-inmemory storage-inmemory" --release --target aarch64-apple-ios-sim`
  3) `cbindgen --crate panther-ffi --output bindings/include/panther.h`
  4) `cd samples/swift && ./GENERATE_PROJECT.sh` and open the generated `.xcodeproj`.

Android (Kotlin JNI, Emulator)
- Path: samples/kotlin/android
- Build files: Kotlin DSL (`build.gradle.kts`, `settings.gradle.kts`).
- UI: Buttons for Record Metric, List Items (metric names), Get Logs, plus fields for provider (type/base/model/key) and Validate.
- Validation: White‑label — Kotlin `PantherSDK.make(listOf(LLM(...)))` then `validate(prompt)`.
- Steps:
  1) `cargo install cargo-ndk`
  2) `rustup target add x86_64-linux-android`
  3) `cargo ndk -t x86_64 -o samples/kotlin/android/app/src/main/jniLibs build -p panther-ffi --features "metrics-inmemory storage-inmemory" --release`
  4) `cbindgen --crate panther-ffi --output bindings/include/panther.h`
  5) Open in Android Studio and Run on an x86_64 emulator.

Flutter
- Path: samples/flutter
- Included: FFI bindings + example app with fields for provider (type/base/model/key). Calls `validateMulti(prompt, providersJson)`.
- Note: This sample uses Dart FFI directly. If you require platform channels with intermediate Swift/Kotlin layers, I can scaffold a plugin-style project on request.
- Steps:
  1) Run `samples/flutter/GENERATE_PROJECT.sh`
  2) In generated app, `flutter pub add ffi`
  3) Build and link native libs for your target as described in README.

React Native
- Path: samples/react_native
- Includes iOS Objective‑C module and Android Java + JNI wrappers with a JS module (`Panther.ts`).
- Validation: Use `validateMulti(prompt, providersJson)` from `Panther.ts` (pass JSON array of providers as described above).
- Steps:
  1) `samples/react_native/GENERATE_PROJECT.sh` to create a CLI RN project and copy scaffolding.
  2) Build platform libs and headers (as above), link iOS static lib + header, and place Android `.so` into `jniLibs`.
  3) Wire the native module (iOS add file to target; Android register in a package if needed). Use the exported JS functions to call into Rust.

Agents (Stage 6) — Quickstart via API
- Enable FFI with agents (e.g., sync providers):
  - `cargo build -p panther-ffi --features "agents agents-openai agents-ollama"`
- Start the Python API (loads FFI from `target/*`):
  - `uvicorn panthersdk.api:create_app --host 0.0.0.0 --port 8000`
- Trigger an agent run (curl):
```
curl -sX POST http://127.0.0.1:8000/agent/run \
  -H 'Content-Type: application/json' \
  -d '{
    "plan": { "type": "ValidateSealAnchor" },
    "input": {
      "prompt": "Explain insulin function",
      "providers": [
        {"type":"openai","api_key":"'$OPENAI_API_KEY'","base_url":"https://api.openai.com","model":"gpt-4o-mini"}
      ]
    },
    "async_run": true
  }'
```
- Follow up:
  - `curl -s "http://127.0.0.1:8000/agent/status?run_id=<id>" | jq`
  - `curl -Ns "http://127.0.0.1:8000/agent/events/stream?run_id=<id>"`

In‑app integration
- Swift/Kotlin/Flutter/RN can call these HTTP endpoints for orchestration.
- Or integrate FFI `panther_agent_run` directly (platform bridge). Examples will be added.
