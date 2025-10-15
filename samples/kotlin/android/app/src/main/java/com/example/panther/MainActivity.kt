package com.example.panther

import android.os.Bundle
import android.text.InputType
import android.view.View
import android.widget.*
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    data class ProviderPreset(
        val label: String,
        val type: String,
        val baseUrl: String,
        val defaultModel: String,
        val requiresKey: Boolean
    )

    private val presets = listOf(
        ProviderPreset("OpenAI",   "openai", "https://api.openai.com",                   "gpt-4o-mini",              true),
        ProviderPreset("Groq",     "openai", "https://api.groq.com/openai/v1",          "llama3-70b-8192",          true),
        ProviderPreset("Together", "openai", "https://api.together.xyz/v1",             "meta-llama/Meta-Llama-3.1-70B-Instruct", true),
        ProviderPreset("Mistral",  "openai", "https://api.mistral.ai",                 "mistral-small-latest",     true),
        ProviderPreset("Ollama",   "ollama", "http://127.0.0.1:11434",                 "llama3",                   false)
    )

    private val prefs by lazy { getSharedPreferences("panther-sdk", MODE_PRIVATE) }
    private var lastProof: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val layout = ScrollView(this)
        val content = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(16,16,16,16) }
        layout.addView(content)

        val btnRecord = Button(this).apply { text = "Record Metric" }
        val btnList = Button(this).apply { text = "List Items" }
        val btnLogs = Button(this).apply { text = "Get Logs" }
        val prompt = EditText(this).apply { hint = "Prompt" }

        val spinner = Spinner(this)
        spinner.adapter = ArrayAdapter(this, android.R.layout.simple_spinner_dropdown_item, presets.map { it.label })

        val baseInput = EditText(this)
        val modelInput = EditText(this)
        val apiKeyInput = EditText(this).apply { hint = "API Key" }
        apiKeyInput.inputType = InputType.TYPE_TEXT_VARIATION_PASSWORD

        val includeOllamaCheck = CheckBox(this).apply { text = "Include local Ollama" }
        val ollamaBaseInput = EditText(this).apply { hint = "Ollama Base" }
        val ollamaModelInput = EditText(this).apply { hint = "Ollama Model" }

        val guidelineToggle = CheckBox(this).apply { text = "Use custom guidelines" }
        val guidelineInput = EditText(this).apply {
            hint = "Guidelines JSON"
            setLines(10)
            isSingleLine = false
            setText(loadGuidelines())
        }
        val guidelineStatus = TextView(this)
        val btnSaveGuidelines = Button(this).apply { text = "Save Guidelines" }

        val btnValidate = Button(this).apply { text = "Validate" }
        // Backend API config
        val backendBaseInput = EditText(this).apply { hint = "API Base (e.g. http://10.0.2.2:8000)" }
        val backendKeyInput = EditText(this).apply { hint = "API Key (X-API-Key, optional)" }
        val btnAnchor = Button(this).apply { text = "Anchor Proof" }
        val btnStatus = Button(this).apply { text = "Check Status" }
        val btnOpenExplorer = Button(this).apply { text = "View on Explorer" }
        val btnOpenContract = Button(this).apply { text = "View Contract" }
        var lastExplorerUrl: String? = null
        var lastContractUrl: String? = null
        val outputView = TextView(this)

        var currentPreset = presets.first()
        applyPreset(currentPreset, baseInput, modelInput, apiKeyInput)
        spinner.onItemSelectedListener = object : AdapterView.OnItemSelectedListener {
            override fun onItemSelected(parent: AdapterView<*>?, view: View?, position: Int, id: Long) {
                currentPreset = presets[position]
                applyPreset(currentPreset, baseInput, modelInput, apiKeyInput)
            }
            override fun onNothingSelected(parent: AdapterView<*>?) {}
        }

        includeOllamaCheck.setOnCheckedChangeListener { _, checked ->
            val visibility = if (checked) View.VISIBLE else View.GONE
            ollamaBaseInput.visibility = visibility
            ollamaModelInput.visibility = visibility
        }
        guidelineInput.visibility = View.GONE
        btnSaveGuidelines.visibility = View.GONE
        guidelineStatus.visibility = View.GONE
        guidelineToggle.setOnCheckedChangeListener { _, checked ->
            val visibility = if (checked) View.VISIBLE else View.GONE
            guidelineInput.visibility = visibility
            btnSaveGuidelines.visibility = visibility
            guidelineStatus.visibility = View.GONE
        }

        btnSaveGuidelines.setOnClickListener {
            val json = guidelineInput.text.toString()
            if (validateGuidelines(json)) {
                prefs.edit().putString("guidelines", json).apply()
                guidelineStatus.text = "Guidelines saved."
                guidelineStatus.visibility = View.VISIBLE
            } else {
                guidelineStatus.text = "Invalid JSON (expect array of {topic, expected_terms})."
                guidelineStatus.visibility = View.VISIBLE
            }
        }

        content.apply {
            addView(btnRecord)
            addView(btnList)
            addView(btnLogs)
            addView(prompt)
            addView(spinner)
            addView(baseInput)
            addView(modelInput)
            addView(apiKeyInput)
            addView(includeOllamaCheck)
            addView(ollamaBaseInput)
            addView(ollamaModelInput)
            addView(guidelineToggle)
            addView(guidelineInput)
            addView(btnSaveGuidelines)
            addView(guidelineStatus)
            addView(TextView(this@MainActivity).apply { text = "Backend API" })
            addView(backendBaseInput)
            addView(backendKeyInput)
            addView(btnValidate)
            addView(btnAnchor)
            addView(btnStatus)
            addView(btnOpenExplorer)
            addView(btnOpenContract)
            addView(outputView)
        }
        setContentView(layout)

        PantherBridge.pantherInit()
        // Load backend prefs
        backendBaseInput.setText(prefs.getString("api.base", if (android.os.Build.FINGERPRINT.contains("generic")) "http://10.0.2.2:8000" else "http://127.0.0.1:8000"))
        backendKeyInput.setText(prefs.getString("api.key", ""))

        btnRecord.setOnClickListener { PantherBridge.recordMetric("button_press") }
        btnList.setOnClickListener { outputView.text = PantherBridge.listStorageItems() }
        btnLogs.setOnClickListener { outputView.text = PantherBridge.getLogs() }
        btnValidate.setOnClickListener {
            val llms = mutableListOf<PantherSDK.LLM>()
            val base = baseInput.text.toString().trim()
            val mdl = modelInput.text.toString().trim()
            if (currentPreset.requiresKey && apiKeyInput.text.isNullOrBlank()) {
                outputView.text = "API key required for ${currentPreset.label}"
                return@setOnClickListener
            }
            llms.add(
                PantherSDK.LLM(
                    type = currentPreset.type,
                    base_url = base,
                    model = mdl,
                    api_key = if (currentPreset.requiresKey) apiKeyInput.text.toString().trim() else null
                )
            )
            if (includeOllamaCheck.isChecked) {
                llms.add(
                    PantherSDK.LLM(
                        type = "ollama",
                        base_url = ollamaBaseInput.text.toString().trim(),
                        model = ollamaModelInput.text.toString().trim(),
                        api_key = null
                    )
                )
            }
            val sdk = PantherSDK.make(llms)
            val guidelines = if (guidelineToggle.isChecked) guidelineInput.text.toString() else null
            val (lines, proof) = sdk.validateAndGetProof(prompt.text.toString(), guidelines)
            val buf = StringBuilder()
            buf.append(lines.joinToString("\n"))
            if (proof != null) {
                buf.append("\nProof: ").append(proof)
                lastProof = proof
            } else {
                lastProof = null
            }
            outputView.text = buf.toString()
        }

        btnAnchor.setOnClickListener {
            val proof = lastProof
            if (proof == null) {
                outputView.text = outputView.text.toString() + "\nNo proof to anchor."
                return@setOnClickListener
            }
            Thread {
                try {
                    val base = backendBaseInput.text.toString().trim().ifBlank { if (android.os.Build.FINGERPRINT.contains("generic")) "http://10.0.2.2:8000" else "http://127.0.0.1:8000" }
                    val url = java.net.URL("$base/proof/anchor")
                    val conn = (url.openConnection() as java.net.HttpURLConnection).apply {
                        requestMethod = "POST"
                        setRequestProperty("Content-Type", "application/json")
                        val key = backendKeyInput.text.toString().trim()
                        if (key.isNotEmpty()) setRequestProperty("X-API-Key", key)
                        doOutput = true
                    }
                    val payload = "{" + "\"hash\":\"0x$proof\"" + "}"
                    conn.outputStream.use { it.write(payload.toByteArray()) }
                    val res = conn.inputStream.bufferedReader().use { it.readText() }
                    val obj = org.json.JSONObject(res)
                    val tx = obj.optString("tx_hash", null)
                    val ex = obj.optString("explorer_url", null)
                    runOnUiThread {
                        outputView.text = outputView.text.toString() + (if (tx != null) "\nAnchored tx: $tx" else "\nAnchor failed")
                        lastExplorerUrl = ex
                    }
                } catch (e: Exception) {
                    runOnUiThread {
                        outputView.text = outputView.text.toString() + "\nAnchor error: ${e.message}"
                    }
                }
            }.start()
            // Save backend prefs
            prefs.edit().putString("api.base", backendBaseInput.text.toString().trim()).putString("api.key", backendKeyInput.text.toString().trim()).apply()
        }

        btnStatus.setOnClickListener {
            val proof = lastProof
            if (proof == null) {
                outputView.text = outputView.text.toString() + "\nNo proof to check."
                return@setOnClickListener
            }
            Thread {
                try {
                    val base = backendBaseInput.text.toString().trim().ifBlank { if (android.os.Build.FINGERPRINT.contains("generic")) "http://10.0.2.2:8000" else "http://127.0.0.1:8000" }
                    val url = java.net.URL("$base/proof/status?hash=0x$proof")
                    val conn = (url.openConnection() as java.net.HttpURLConnection)
                    val key = backendKeyInput.text.toString().trim()
                    if (key.isNotEmpty()) conn.setRequestProperty("X-API-Key", key)
                    val res = conn.inputStream.bufferedReader().use { it.readText() }
                    val obj = org.json.JSONObject(res)
                    val anchored = obj.optBoolean("anchored", false)
                    val cu = obj.optString("contract_url", null)
                    runOnUiThread {
                        outputView.text = outputView.text.toString() + "\nAnchored: $anchored"
                        lastContractUrl = cu
                    }
                } catch (e: Exception) {
                    runOnUiThread { outputView.text = outputView.text.toString() + "\nStatus error: ${e.message}" }
                }
            }.start()
        }

        btnOpenExplorer.setOnClickListener {
            val url = lastExplorerUrl
            if (url != null) {
                val intent = android.content.Intent(android.content.Intent.ACTION_VIEW, android.net.Uri.parse(url))
                startActivity(intent)
            } else {
                outputView.text = outputView.text.toString() + "\nNo explorer URL."
            }
        }

        btnOpenContract.setOnClickListener {
            val url = lastContractUrl
            if (url != null) {
                val intent = android.content.Intent(android.content.Intent.ACTION_VIEW, android.net.Uri.parse(url))
                startActivity(intent)
            } else {
                outputView.text = outputView.text.toString() + "\nNo contract URL."
            }
        }

        // --- Plagiarism (Jaccard n-gram) section ---
        val plagTitle = TextView(this).apply { text = "Plagiarism (Jaccard n-gram)"; textSize = 18f }
        val plagCorpusInput = EditText(this).apply {
            hint = "Corpus (one per line)"
            setLines(4); isSingleLine = false
            setText("Insulin regulates glucose in the blood.\nVitamin C supports the immune system.")
        }
        val plagCandidateInput = EditText(this).apply {
            hint = "Candidate text"
            setText("Insulin regulates glucose in the blood.")
        }
        val plagNInput = EditText(this).apply { hint = "n-gram (3)"; setText("3"); inputType = InputType.TYPE_CLASS_NUMBER }
        val btnPlag = Button(this).apply { text = "Check Plagiarism" }
        btnPlag.setOnClickListener {
            try {
                val lines = plagCorpusInput.text.toString().split("\n").map { it.trim() }.filter { it.isNotEmpty() }
                val cand = plagCandidateInput.text.toString()
                val n = plagNInput.text.toString().toIntOrNull() ?: 3
                val score = PantherBridge.plagiarismScore(lines, cand, n)
                outputView.text = outputView.text.toString() + "\nPlagiarism score: ${String.format("%.2f", score)}"
            } catch (e: Exception) {
                outputView.text = outputView.text.toString() + "\nPlagiarism error: ${e.message}"
            }
        }

        // Add to content
        content.addView(plagTitle)
        content.addView(plagCorpusInput)
        content.addView(plagCandidateInput)
        val row = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL }
        row.addView(btnPlag)
        val lbl = TextView(this).apply { text = "  n-gram: " }
        row.addView(lbl)
        row.addView(plagNInput)
        content.addView(row)
    }

    private fun applyPreset(preset: ProviderPreset, base: EditText, model: EditText, apiKey: EditText) {
        base.setText(preset.baseUrl)
        model.setText(preset.defaultModel)
        apiKey.visibility = if (preset.requiresKey) View.VISIBLE else View.GONE
        if (!preset.requiresKey) apiKey.setText("")
    }

    private fun loadGuidelines(): String {
        return prefs.getString("guidelines", PantherSDK.defaultGuidelines) ?: PantherSDK.defaultGuidelines
    }

    private fun validateGuidelines(json: String): Boolean {
        return try {
            val arr = JSONArray(json)
            (0 until arr.length()).all { idx ->
                val obj = arr.getJSONObject(idx)
                obj.has("topic") && obj.has("expected_terms")
            }
        } catch (_: Throwable) {
            false
        }
    }
}
