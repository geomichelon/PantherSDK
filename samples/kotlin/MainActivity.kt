package com.example.panther

import android.os.Bundle
import android.widget.*
import org.json.JSONArray
import org.json.JSONObject
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        PantherBridge.pantherInit()

        val root = ScrollView(this)
        val ll = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(24,24,24,24) }
        root.addView(ll)

        val prompt = EditText(this).apply { hint = "Prompt"; setText("Explique recomendações seguras de medicamentos na gravidez.") }
        val providers = EditText(this).apply {
            hint = "Providers JSON"
            setText("""[
  {\"type\":\"openai\",\"base_url\":\"https://api.openai.com\",\"model\":\"gpt-4o-mini\",\"api_key\":\"\"}
]""")
            minLines = 4
        }
        val validateBtn = Button(this).apply { text = "Validate (with proof)" }
        val resultBox = TextView(this)

        val apiBase = EditText(this).apply { hint = "API Base (http://10.0.2.2:8000)" }
        val apiKey = EditText(this).apply { hint = "API Key (optional)" }
        val anchorBtn = Button(this).apply { text = "Anchor Proof" }
        val statusBtn = Button(this).apply { text = "Check Status" }

        val biasInput = EditText(this).apply {
            hint = "Compliance samples (one per line)"
            minLines = 3
        }
        val biasBtn = Button(this).apply { text = "Compute Bias" }

        ll.addView(prompt)
        ll.addView(providers)
        ll.addView(validateBtn)
        ll.addView(resultBox)
        ll.addView(TextView(this).apply { text = "Backend API" })
        ll.addView(apiBase)
        ll.addView(apiKey)
        ll.addView(anchorBtn)
        ll.addView(statusBtn)
        ll.addView(TextView(this).apply { text = "Compliance" })
        ll.addView(biasInput)
        ll.addView(biasBtn)

        setContentView(root)

        var lastProof: String? = null

        validateBtn.setOnClickListener {
            try {
                val raw = PantherBridge.validateMultiWithProof(prompt.text.toString(), providers.text.toString())
                val obj = try { JSONObject(raw) } catch (_:Throwable) { null }
                if (obj != null && obj.has("results")) {
                    val arr = obj.getJSONArray("results")
                    val lines = mutableListOf<String>()
                    for (i in 0 until arr.length()) {
                        val r = arr.getJSONObject(i)
                        val name = r.optString("provider_name","?")
                        val score = r.optDouble("adherence_score",0.0)
                        val lat = r.optInt("latency_ms",0)
                        lines.add("$name – ${"%.1f".format(score)}% – ${lat}ms")
                    }
                    val proof = obj.optJSONObject("proof")
                    lastProof = proof?.optString("combined_hash")
                    if (lastProof != null) lines.add("Proof: $lastProof")
                    resultBox.text = lines.joinToString("\n")
                } else if (raw.trim().startsWith("[")) {
                    val arr = JSONArray(raw)
                    val lines = mutableListOf<String>()
                    for (i in 0 until arr.length()) {
                        val r = arr.getJSONObject(i)
                        val name = r.optString("provider_name","?")
                        val score = r.optDouble("adherence_score",0.0)
                        val lat = r.optInt("latency_ms",0)
                        lines.add("$name – ${"%.1f".format(score)}% – ${lat}ms")
                    }
                    resultBox.text = lines.joinToString("\n")
                } else {
                    resultBox.text = raw
                }
            } catch (e:Throwable) {
                resultBox.text = e.message
            }
        }

        anchorBtn.setOnClickListener {
            val ph = lastProof ?: return@setOnClickListener
            val base = apiBase.text.toString().ifBlank { "http://10.0.2.2:8000" }
            Thread {
                try {
                    val url = java.net.URL("$base/proof/anchor")
                    val conn = (url.openConnection() as java.net.HttpURLConnection).apply {
                        requestMethod = "POST"
                        doOutput = true
                        setRequestProperty("Content-Type","application/json")
                        val k = apiKey.text.toString().trim()
                        if (k.isNotEmpty()) setRequestProperty("X-API-Key", k)
                    }
                    conn.outputStream.use { it.write("{"hash":"0x$ph"}".toByteArray()) }
                    val resp = conn.inputStream.bufferedReader().readText()
                    runOnUiThread { resultBox.text = (resultBox.text.toString()+"\n"+resp).trim() }
                } catch (e:Throwable) {
                    runOnUiThread { resultBox.text = (resultBox.text.toString()+"\nAnchor error: "+e.message).trim() }
                }
            }.start()
        }

        statusBtn.setOnClickListener {
            val ph = lastProof ?: return@setOnClickListener
            val base = apiBase.text.toString().ifBlank { "http://10.0.2.2:8000" }
            Thread {
                try {
                    val url = java.net.URL("$base/proof/status?hash=0x$ph")
                    val conn = (url.openConnection() as java.net.HttpURLConnection)
                    val k = apiKey.text.toString().trim()
                    if (k.isNotEmpty()) conn.setRequestProperty("X-API-Key", k)
                    val resp = conn.inputStream.bufferedReader().readText()
                    runOnUiThread { resultBox.text = (resultBox.text.toString()+"\n"+resp).trim() }
                } catch (e:Throwable) {
                    runOnUiThread { resultBox.text = (resultBox.text.toString()+"\nStatus error: "+e.message).trim() }
                }
            }.start()
        }

        biasBtn.setOnClickListener {
            val lines = biasInput.text.toString().split('\n').map { it.trim() }.filter { it.isNotEmpty() }
            val json = JSONArray(lines).toString()
            val res = PantherBridge.biasDetect(json)
            resultBox.text = (resultBox.text.toString()+"\nBias: "+res).trim()
        }
    }
}
