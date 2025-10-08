build-all:
	bash samples/swift/scripts/build_rust.sh
	bash samples/kotlin/scripts/build_rust.sh
	bash samples/flutter/scripts/build_rust.sh || true
	bash samples/react_native/scripts/build_rust.sh || true

