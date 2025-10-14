React Native Samples (iOS Simulator / Android Emulator)

Goal: Call PantherSDK from React Native using native modules bridging the C ABI.

API exposed by native module `PantherModule`:
- `init()`
- `generate(prompt)` (reference)
- `metricsBleu(reference, candidate)` (reference)
- `recordMetric(metricName)`
- `listStorageItems()` -> JSON array
- `getLogs()` -> JSON array
- `validateMulti(prompt, providersJson)` -> JSON array of results
- `validateMultiWithProof(prompt, providersJson)` -> `{ results, proof }` (Stage 1)
- `version()` -> SDK version string

What this sample does
- Record Metric: calls `panther_metrics_record` through native module
- List Items: returns metric names via `panther_storage_list_metrics`
- Get Logs: returns in‑process logs via `panther_logs_get`
- Validate (white‑label): call `validateMultiWithProof(prompt, providersJson)` (Stage 1) with a JSON array of providers, e.g.:
  `[{"type":"openai","api_key":"sk-...","base_url":"https://api.openai.com","model":"gpt-4o-mini"}]`. The response includes `proof.combined_hash`.
- Anchor Proof (Stage 2): use helper `anchorProof(hash, apiBase?, apiKey?)` from `Panther.ts` to call your backend API (`/proof/anchor`).

iOS (Objective‑C module)
1) Build iOS static lib + header
- `rustup target add aarch64-apple-ios-sim`
- `cargo build -p panther-ffi --release --target aarch64-apple-ios-sim`
- `cbindgen --crate panther-ffi --output ./panther.h`
2) Add to RN iOS project
- Add `libpanther_ffi.a` to the Xcode project (Link Binary With Libraries).
- Add `panther.h` to a public header path.
- Add `samples/react_native/ios/PantherModule.m` to your iOS target.
3) Use from JS
- See `samples/react_native/Panther.ts` and the example screen `samples/react_native/AppSample.tsx`.

Android (Java + JNI module)
1) Build shared lib for emulator
- `cargo ndk -t x86_64 -o android/app/src/main/jniLibs build -p panther-ffi --release`
2) JNI wrapper
- Add `samples/react_native/android/panther_jni.c` to your RN Android module via `externalNativeBuild`.
3) Java module
- Add `samples/react_native/android/PantherModule.java` and register in a package.
4) Use from JS
- See `samples/react_native/Panther.ts` and the example screen `samples/react_native/AppSample.tsx`.

Creation
- Run `samples/react_native/GENERATE_PROJECT.sh` to create `PantherSample` RN project and copy bridge files.

Example UI (AppSample.tsx)
- A minimal screen is provided at `samples/react_native/AppSample.tsx` with:
  - Prompt field
  - Providers JSON field (defaults to OpenAI‑compatible entry)
  - Backend API Base + API Key inputs
  - Buttons: Validate (calls `validateMultiWithProof`) and Anchor Proof (calls `/proof/anchor` on your API)
- To try it quickly:
  1) Make sure `PantherModule` is wired per the steps above for your RN project.
  2) Copy `Panther.ts` and `AppSample.tsx` into your RN app (e.g., `src/`).
  3) Set `App.tsx` to render `<AppSample />`.
  4) Start your backend API (see root README → Python API Quickstart). For Android emulator, use `http://10.0.2.2:8000`.
  5) Validate and then Anchor Proof to see `tx_hash` (Stage 2).
