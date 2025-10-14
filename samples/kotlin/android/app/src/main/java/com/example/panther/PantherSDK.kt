package com.example.panther

import org.json.JSONArray
import org.json.JSONObject

class PantherSDK private constructor(private val providersJson: String) {

    data class LLM(
        val type: String,
        val base_url: String,
        val model: String,
        val api_key: String? = null,
    )

    fun validate(prompt: String, guidelinesJson: String? = null): List<String> {
        val raw = if (guidelinesJson != null) {
            PantherBridge.validateCustom(prompt, providersJson, guidelinesJson)
        } else {
            PantherBridge.validateMulti(prompt, providersJson)
        }
        return try {
            val arr = JSONArray(raw)
            (0 until arr.length()).map { i ->
                val o = arr.getJSONObject(i)
                val name = o.optString("provider_name", "?")
                val score = o.optDouble("adherence_score", 0.0)
                val lat = o.optInt("latency_ms", 0)
                String.format("%s – %.1f%% – %d ms", name, score, lat)
            }
        } catch (_: Throwable) {
            listOf(raw)
        }
    }

    fun validateAndGetProof(prompt: String, guidelinesJson: String? = null): Pair<List<String>, String?> {
        val raw = if (guidelinesJson != null) {
            PantherBridge.validateCustomWithProof(prompt, providersJson, guidelinesJson)
        } else {
            PantherBridge.validateMultiWithProof(prompt, providersJson)
        }
        return try {
            val obj = org.json.JSONObject(raw)
            val arr = obj.getJSONArray("results")
            val lines = (0 until arr.length()).map { i ->
                val o = arr.getJSONObject(i)
                val name = o.optString("provider_name", "?")
                val score = o.optDouble("adherence_score", 0.0)
                val lat = o.optInt("latency_ms", 0)
                String.format("%s – %.1f%% – %d ms", name, score, lat)
            }
            val proof = obj.optJSONObject("proof")?.optString("combined_hash", null)
            lines to proof
        } catch (_: Throwable) {
            listOf(raw) to null
        }
    }

    companion object {
        val defaultGuidelines: String = """\
[
  {
    \"topic\": \"Uso de medicamentos na gravidez\",
    \"expected_terms\": [
      \"consulta médica\",
      \"contraindicado\",
      \"categoria de risco\",
      \"dosagem\",
      \"efeitos colaterais\",
      \"advertência\",
      \"ANVISA\",
      \"orientação profissional\"
    ]
  }
]
"""

        fun make(llms: List<LLM>): PantherSDK {
            PantherBridge.pantherInit()
            val arr = JSONArray()
            llms.forEach { l ->
                val o = JSONObject()
                o.put("type", l.type)
                o.put("base_url", l.base_url)
                o.put("model", l.model)
                if (l.type == "openai" && l.api_key != null) o.put("api_key", l.api_key)
                arr.put(o)
            }
            return PantherSDK(arr.toString())
        }

        fun version(): String = try { PantherBridge.version() } catch (_: Throwable) { "" }
    }
}
