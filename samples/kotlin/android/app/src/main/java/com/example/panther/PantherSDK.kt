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
    }
}
