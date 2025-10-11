Android (Kotlin) Emulator Sample — PantherSampleKotlin (Kotlin DSL)

Goal: Call PantherSDK via JNI in an Android app (Emulator x86_64).

What this sample does
- Record Metric: `recordMetric("button_press")` via `panther_metrics_record`
- List Items: JSON array of metric names via `panther_storage_list_metrics`
- Get Logs: JSON array of log lines via `panther_logs_get`
- Validate (white‑label): fields for provider (type/base/model/key) and a button to run validation for any LLM. Under the hood, the app calls a single FFI `panther_validation_run_multi` via `PantherBridge.validateMulti`.

0) Open Android Studio and import `samples/kotlin/android`.

1) Build shared lib for Android emulator
- Install cargo-ndk: `cargo install cargo-ndk`
- Add target: `rustup target add x86_64-linux-android`
- Build: `cargo ndk -t x86_64 -o app/src/main/jniLibs build -p panther-ffi --release`
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

Notes
- Build FFI with features: `--features "metrics storage validation validation-openai validation-ollama"`
