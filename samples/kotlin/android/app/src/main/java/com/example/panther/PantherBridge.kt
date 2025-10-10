package com.example.panther

object PantherBridge {
    init { System.loadLibrary("panther_jni"); System.loadLibrary("panther_ffi") }

    external fun pantherInit(): Int
    external fun pantherGenerate(prompt: String): String
    external fun metricsBleu(reference: String, candidate: String): Double
    external fun recordMetric(name: String): Int
    external fun listStorageItems(): String
    external fun getLogs(): String
    external fun validate(prompt: String): String
}
