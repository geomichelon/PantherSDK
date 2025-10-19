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
            val tin = PantherBridge.tokenCount(prompt)
            val lines = (0 until arr.length()).map { i ->
                val o = arr.getJSONObject(i)
                val name = o.optString("provider_name", "?")
                val score = o.optDouble("adherence_score", 0.0)
                val lat = o.optInt("latency_ms", 0)
                val text = o.optString("raw_text", "")
                val tout = PantherBridge.tokenCount(text)
                val rules = (customCostRulesJson ?: defaultCostRulesJson)
                val cost = PantherBridge.calculateCost(tin, tout, name, rules)
                String.format("%s – %.1f%% – %d ms – %d/%d tok – $%.4f", name, score, lat, tin, tout, cost)
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

        // Default pricing (editable in-app if desired)
        val defaultCostRulesJson: String = """
[
  {"match": "openai:gpt-4o-mini",  "usd_per_1k_in": 0.15, "usd_per_1k_out": 0.60},
  {"match": "openai:gpt-4.1-mini", "usd_per_1k_in": 0.30, "usd_per_1k_out": 1.20},
  {"match": "openai:gpt-4.1",      "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},
  {"match": "openai:gpt-4o",       "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},
  {"match": "openai:chatgpt-5",    "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},
  {"match": "ollama:llama3",       "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00},
  {"match": "ollama:phi3",         "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00},
  {"match": "ollama:mistral",      "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00}
]
"""

        // Optional runtime override (set from activities/fragments)
        @JvmStatic var customCostRulesJson: String? = null
        @JvmStatic fun setCostRulesJson(json: String?) { customCostRulesJson = json }
    }
}
