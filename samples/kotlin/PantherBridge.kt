package com.example.panther

object PantherBridge {
    init {
        System.loadLibrary("panther_ffi")
    }

    external fun pantherInit(): Int
    external fun pantherGenerate(prompt: String): String
}

