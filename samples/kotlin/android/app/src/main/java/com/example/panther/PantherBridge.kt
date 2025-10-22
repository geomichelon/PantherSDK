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
    external fun validateOpenAI(prompt: String, apiKey: String, model: String, base: String): String
    external fun validateOllama(prompt: String, base: String, model: String): String
    external fun version(): String
    external fun validateMultiWithProof(prompt: String, providersJson: String): String
    external fun validateCustomWithProof(prompt: String, providersJson: String, guidelinesJson: String): String
    external fun biasDetect(samplesJson: String): String
    external fun tokenCount(text: String): Int
    external fun calculateCost(tokensIn: Int, tokensOut: Int, providerName: String, costRulesJson: String): Double

<<<<<<< HEAD
    // Guidelines similarity (FFI)
    external fun guidelinesIngest(json: String): Int
    external fun guidelinesScores(query: String, topK: Int, method: String): String
    external fun guidelinesSave(name: String, json: String): Int
    external fun guidelinesLoad(name: String): Int
    external fun guidelinesBuildEmbeddings(method: String): Int

=======
>>>>>>> origin/main
    // Helpers for corpus JSON and plagiarism score
    fun corpusJson(list: List<String>): String = JSONArray(list).toString()
    fun plagiarismScore(corpus: List<String>, candidate: String, ngram: Int = 3): Double {
        val n = if (ngram > 0) ngram else 3
        return metricsPlagiarismNgram(corpusJson(corpus), candidate, n)
    }
}
