plugins { id("com.android.application") }

android {
    namespace = "com.example.panther"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.example.panther"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"
        ndk { abiFilters.add("x86_64") }
    }

    buildTypes {
        getByName("release") { isMinifyEnabled = false }
    }

    externalNativeBuild { cmake { path = file("src/main/cpp/CMakeLists.txt") } }
    sourceSets.getByName("main").jniLibs.srcDirs("src/main/jniLibs")
}

dependencies { implementation("androidx.appcompat:appcompat:1.7.0") }

// Pre-build: build Rust FFI and copy into jniLibs
tasks.register<Exec>("buildRustFfi") {
    workingDir = file("${rootDir}/../scripts")
    commandLine("bash", "build_rust.sh")
}

tasks.named("preBuild").configure { dependsOn("buildRustFfi") }
