Android (Kotlin) Emulator Sample — PantherSampleKotlin (Kotlin DSL)

Goal: Call PantherSDK via JNI in an Android app (Emulator x86_64).

What this sample does
- Record Metric: `recordMetric("button_press")` via `panther_metrics_record`
- List Items: JSON array of metric names via `panther_storage_list_metrics`
- Get Logs: JSON array of log lines via `panther_logs_get`
- Validate (white‑label): fields for provider (type/base/model/key) and a button to run validation for any LLM. Under the hood, the app calls a single FFI `panther_validation_run_multi` via `PantherBridge.validateMulti`.
- Multi‑provider with proof: calls `validateMultiWithProof` and shows `combined_hash`.
- Token + Cost estimate: each result line appends `tokens_in/tokens_out` and `$cost` using `panther_token_count` and `panther_calculate_cost` with editable rules.

0) Open Android Studio and import `samples/kotlin/android`.

1) Build shared lib for Android emulator
- Install cargo-ndk: `cargo install cargo-ndk`
- Add target: `rustup target add x86_64-linux-android`
- Build FFI (enable validation + metrics/bias):
  `cargo ndk -t x86_64 -o app/src/main/jniLibs build -p panther-ffi --release --features "validation validation-openai validation-ollama metrics-inmemory storage-inmemory"`
  This creates `app/src/main/jniLibs/x86_64/libpanther_ffi.so`.

2) JNI wrapper is configured
- `externalNativeBuild` and CMake are preconfigured under `app/src/main/cpp`.
- Ensure `bindings/include/panther.h` exists (run cbindgen).

3) Kotlin bridge
- `app/src/main/java/com/example/panther/PantherBridge.kt` exposes:
  - `recordMetric(name: String)`
  - `listStorageItems(): String` (JSON array)
  - `getLogs(): String` (JSON array)
  - `validateMulti(prompt, providersJson)` (white‑label validation)
- `MainActivity.kt` builds `providersJson` and prints a formatted list of results.

PantherSDK (Kotlin) usage
```kotlin
// Build white‑label list
val llm = PantherSDK.LLM(type = "openai", base_url = "https://api.openai.com", model = "gpt-4o-mini", api_key = "sk-...")
val sdk = PantherSDK.make(listOf(llm))
val lines = sdk.validate("Explain insulin function")
```

4) Run on Android Emulator (x86_64)
- Ensure the emulator ABI is x86_64 and run.

2) JNI wrapper is configured
- `app/src/main/cpp/panther_jni.c` exposes validate/withProof + metrics + bias + token/cost.
- Ensure `bindings/include/panther.h` exists (run cbindgen if missing).

3) Run on Android Emulator (x86_64)
- Ensure the emulator ABI is x86_64 and run.

Notes
- Use `10.0.2.2` for backend calls from the emulator.
- The default pricing table is embedded in `PantherSDK.kt` (`defaultCostRulesJson`).
