Central iOS XCFramework output folder

Coloque aqui a PantherSDKIOS.xcframework (renomeada) gerada pelo script de release:

1) Build a XCFramework:

   # OpenAI + Ollama (baseline)
   FEATURES="metrics-inmemory storage-inmemory validation validation-openai validation-ollama" \
   ./scripts/release/build_ios_xcframework.sh

   # Include Anthropic (sync)
   FEATURES="metrics-inmemory storage-inmemory validation validation-openai validation-ollama validation-anthropic" \
   ./scripts/release/build_ios_xcframework.sh

   # Include Anthropic (async) — enables async validation path
   FEATURES="metrics-inmemory storage-inmemory validation validation-async validation-openai validation-ollama validation-anthropic validation-anthropic-async" \
   ./scripts/release/build_ios_xcframework.sh

2) Copie o bundle gerado para esta pasta:

   VERSION=$(awk '
     $0 ~ /^\[workspace\.package\]/{inpkg=1; next}
     inpkg && $1 ~ /^version/ { gsub(/\"/, ""); gsub(/version *= */, ""); print $0; exit }
   ' Cargo.toml | tr -d '[:space:]')
   rsync -a dist/$VERSION/ios/PantherSDK.xcframework samples/swift/frameworkIOS/PantherSDKIOS.xcframework

O projeto Xcode em samples/swift/PantherSDKSampleIOS já referencia esta pasta (frameworkIOS) e faz embed/sign do framework.
