React Native Samples (iOS Simulator / Android Emulator)

Goal: Call PantherSDK from React Native using native modules bridging the C ABI.

API exposed by native module `PantherModule`:
- `init()`
- `generate(prompt)` (reference)
- `metricsBleu(reference, candidate)` (reference)
- `recordMetric(metricName)`
- `listStorageItems()` -> JSON array
- `getLogs()` -> JSON array

What this sample does
- Record Metric: calls `panther_metrics_record` through native module
- List Items: returns metric names via `panther_storage_list_metrics`
- Get Logs: returns in‑process logs via `panther_logs_get`

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
- See `samples/react_native/Panther.ts` and wire into your component buttons.

Android (Java + JNI module)
1) Build shared lib for emulator
- `cargo ndk -t x86_64 -o android/app/src/main/jniLibs build -p panther-ffi --release`
2) JNI wrapper
- Add `samples/react_native/android/panther_jni.c` to your RN Android module via `externalNativeBuild`.
3) Java module
- Add `samples/react_native/android/PantherModule.java` and register in a package.
4) Use from JS
- See `samples/react_native/Panther.ts` and wire into your component buttons.

Creation
- Run `samples/react_native/GENERATE_PROJECT.sh` to create `PantherSample` RN project and copy bridge files.
