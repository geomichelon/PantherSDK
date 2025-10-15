package com.example.panther

import org.json.JSONArray

object PantherBridge {
    init {
        System.loadLibrary("panther_jni")
        System.loadLibrary("panther_ffi")
    }

    external fun pantherInit(): Int
    external fun pantherGenerate(prompt: String): String
    external fun metricsBleu(reference: String, candidate: String): Double
    external fun metricsPlagiarismNgram(corpusJson: String, candidate: String, ngram: Int): Double
    external fun metricsPlagiarism(corpusJson: String, candidate: String): Double
    external fun recordMetric(name: String): Int
    external fun listStorageItems(): String
    external fun getLogs(): String
    external fun validate(prompt: String): String
    external fun validateMulti(prompt: String, providersJson: String): String
    external fun validateCustom(prompt: String, providersJson: String, guidelinesJson: String): String
    external fun version(): String
    external fun validateMultiWithProof(prompt: String, providersJson: String): String
    external fun validateCustomWithProof(prompt: String, providersJson: String, guidelinesJson: String): String

    // Helpers for corpus JSON and plagiarism score
    fun corpusJson(list: List<String>): String = JSONArray(list).toString()
    fun plagiarismScore(corpus: List<String>, candidate: String, ngram: Int = 3): Double {
        val n = if (ngram > 0) ngram else 3
        return metricsPlagiarismNgram(corpusJson(corpus), candidate, n)
    }
}
