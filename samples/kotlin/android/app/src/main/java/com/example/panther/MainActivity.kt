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
        ProviderPreset("Ollama",   "ollama", "http://127.0.0.1:11434",                 "llama3",                   false),
        ProviderPreset("Anthropic","anthropic","https://api.anthropic.com",          "claude-3-5-sonnet-latest", true)
    )
    private val openAIModels = listOf("gpt-4o-mini", "gpt-4.1-mini", "gpt-4.1", "gpt-4o", "chatgpt-5")
    private val ollamaModels = listOf("llama3", "phi3", "mistral")

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

        // Model preset rows (toggle visibility by provider type)
        val modelPresetRowOpenAI = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; visibility = View.GONE }
        openAIModels.forEach { m ->
            val b = Button(this).apply { text = m; setOnClickListener { modelInput.setText(m) } }
            modelPresetRowOpenAI.addView(b)
        }
        val modelPresetRowOllama = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; visibility = View.GONE }
        ollamaModels.forEach { m ->
            val b = Button(this).apply { text = m; setOnClickListener { modelInput.setText(m) } }
            modelPresetRowOllama.addView(b)
        }

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
        val btnGuidelinesScores = Button(this).apply { text = "Guidelines Scores (Hybrid)" }
        // Index persistence controls
        val idxNameInput = EditText(this).apply { hint = "Index name"; setText("default") }
        val btnIndexSave = Button(this).apply { text = "Save Index" }
        val btnIndexLoad = Button(this).apply { text = "Load Index" }

        // Cost rules editor
        val costRulesTitle = TextView(this).apply { text = "Cost Rules (JSON)"; textSize = 16f }
        val costRulesInput = EditText(this).apply {
            hint = "Cost rules as JSON array"
            setLines(6); isSingleLine = false
            setText(prefs.getString("cost.rules", PantherSDK.defaultCostRulesJson) ?: PantherSDK.defaultCostRulesJson)
        }
        val btnRestoreCost = Button(this).apply { text = "Restore Default" }
        val btnSaveCost = Button(this).apply { text = "Save Cost Rules" }

        val btnValidate = Button(this).apply { text = "Validate" }
        // Mode selector: Single / Multi / With Proof
        val modeSpinner = Spinner(this)
        val modes = listOf("Single", "Multi", "With Proof")
        modeSpinner.adapter = ArrayAdapter(this, android.R.layout.simple_spinner_dropdown_item, modes)
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
        // Load provider session
        run {
            val t = prefs.getString("prov.type", null)
            val b = prefs.getString("prov.base", null)
            val m = prefs.getString("prov.model", null)
            val k = prefs.getString("prov.key", null)
            if (t != null) {
                val idx = presets.indexOfFirst { it.label.equals(t, ignoreCase = true) || it.type.equals(t, ignoreCase = true) }
                if (idx >= 0) currentPreset = presets[idx]
            }
            if (b != null) baseInput.setText(b)
            if (m != null) modelInput.setText(m)
            if (k != null) apiKeyInput.setText(k)
        }
        applyPreset(currentPreset, baseInput, modelInput, apiKeyInput)
        spinner.onItemSelectedListener = object : AdapterView.OnItemSelectedListener {
            override fun onItemSelected(parent: AdapterView<*>?, view: View?, position: Int, id: Long) {
                currentPreset = presets[position]
                applyPreset(currentPreset, baseInput, modelInput, apiKeyInput)
                // Toggle model preset rows
                when (currentPreset.type) {
                    "openai" -> { modelPresetRowOpenAI.visibility = View.VISIBLE; modelPresetRowOllama.visibility = View.GONE }
                    "ollama" -> { modelPresetRowOpenAI.visibility = View.GONE; modelPresetRowOllama.visibility = View.VISIBLE }
                    else -> { modelPresetRowOpenAI.visibility = View.GONE; modelPresetRowOllama.visibility = View.GONE }
                }
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

        btnIndexSave.setOnClickListener {
            val name = idxNameInput.text.toString().trim().ifEmpty { "default" }
            val json = guidelineInput.text.toString().trim()
            if (json.isEmpty()) { guidelineStatus.text = "Provide guidelines JSON"; guidelineStatus.visibility = View.VISIBLE; return@setOnClickListener }
            val rc = PantherBridge.guidelinesSave(name, json)
            guidelineStatus.text = if (rc == 0) "Index saved: $name" else "Save failed"
            guidelineStatus.visibility = View.VISIBLE
        }
        btnIndexLoad.setOnClickListener {
            val name = idxNameInput.text.toString().trim().ifEmpty { "default" }
            val n = PantherBridge.guidelinesLoad(name)
            guidelineStatus.text = if (n > 0) "Index loaded: $name ($n items)" else "Load failed or empty"
            guidelineStatus.visibility = View.VISIBLE
        }

        btnGuidelinesScores.setOnClickListener {
            val json = guidelineInput.text.toString().trim()
            val q = prompt.text.toString().trim()
            if (json.isEmpty() || q.isEmpty()) { outputView.text = "Enter guidelines JSON and prompt"; return@setOnClickListener }
            try {
                val n = PantherBridge.guidelinesIngest(json)
                if (n <= 0) { outputView.text = "No guidelines ingested"; return@setOnClickListener }
                val out = PantherBridge.guidelinesScores(q, 5, "hybrid")
                // Parse JSON and format with bow/jaccard details
                val arr = org.json.JSONArray(out)
                val lines = mutableListOf<String>()
                for (i in 0 until arr.length()) {
                    val o = arr.getJSONObject(i)
                    val topic = o.optString("topic", "?")
                    val score = o.optDouble("score", 0.0)
                    val bow = o.optDouble("bow", Double.NaN)
                    val jac = o.optDouble("jaccard", Double.NaN)
                    val bowStr = if (bow.isNaN()) "" else String.format("bow %.3f", bow)
                    val jacStr = if (jac.isNaN()) "" else String.format(", jac %.3f", jac)
                    lines.add("$topic – ${"%.3f".format(score)} ($bowStr$jacStr)")
                }
                outputView.text = lines.joinToString("\n")
            } catch (e: Exception) {
                outputView.text = e.message
            }
        }

        btnRestoreCost.setOnClickListener {
            costRulesInput.setText(PantherSDK.defaultCostRulesJson)
        }
        btnSaveCost.setOnClickListener {
            prefs.edit().putString("cost.rules", costRulesInput.text.toString()).apply()
            Toast.makeText(this, "Cost rules saved", Toast.LENGTH_SHORT).show()
        }

        content.apply {
            addView(btnRecord)
            addView(btnList)
            addView(btnLogs)
            addView(prompt)
            addView(spinner)
            addView(baseInput)
            addView(modelInput)
            addView(modelPresetRowOpenAI)
            addView(modelPresetRowOllama)
            addView(apiKeyInput)
            // Save provider session
            addView(Button(this@MainActivity).apply {
                text = "Save Provider Session"
                setOnClickListener {
                    prefs.edit()
                        .putString("prov.type", currentPreset.label)
                        .putString("prov.base", baseInput.text.toString().trim())
                        .putString("prov.model", modelInput.text.toString().trim())
                        .putString("prov.key", apiKeyInput.text.toString().trim())
                        .apply()
                    Toast.makeText(this@MainActivity, "Provider session saved", Toast.LENGTH_SHORT).show()
                }
            })
            addView(TextView(this@MainActivity).apply { text = "Mode" })
            addView(modeSpinner)
            addView(includeOllamaCheck)
            addView(ollamaBaseInput)
            addView(ollamaModelInput)
            addView(guidelineToggle)
            addView(guidelineInput)
            addView(btnSaveGuidelines)
            addView(btnGuidelinesScores)
            // Index persistence
            addView(idxNameInput)
            val idxRow = LinearLayout(this@MainActivity).apply { orientation = LinearLayout.HORIZONTAL }
            idxRow.addView(btnIndexSave)
            val spaceIdx = Space(this@MainActivity); spaceIdx.minimumWidth = 24; idxRow.addView(spaceIdx)
            idxRow.addView(btnIndexLoad)
            addView(idxRow)
            addView(guidelineStatus)
            // Cost rules (JSON) editor
            addView(costRulesTitle)
            addView(costRulesInput)
            val costRow = LinearLayout(this@MainActivity).apply { orientation = LinearLayout.HORIZONTAL }
            costRow.addView(btnRestoreCost)
            val spacer = Space(this@MainActivity)
            spacer.minimumWidth = 24
            costRow.addView(spacer)
            costRow.addView(btnSaveCost)
            addView(costRow)
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
            val providers = mutableListOf<PantherSDK.LLM>()
            providers.add(
                PantherSDK.LLM(
                    type = currentPreset.type,
                    base_url = base,
                    model = mdl,
                    api_key = if (currentPreset.requiresKey) apiKeyInput.text.toString().trim() else null
                )
            )
            if (includeOllamaCheck.isChecked) {
                providers.add(
                    PantherSDK.LLM(
                        type = "ollama",
                        base_url = ollamaBaseInput.text.toString().trim(),
                        model = ollamaModelInput.text.toString().trim(),
                        api_key = null
                    )
                )
            }

            val mode = modes[modeSpinner.selectedItemPosition]
            val useCustom = guidelineToggle.isChecked
            val guidelines = if (useCustom) guidelineInput.text.toString() else null
            PantherSDK.setCostRulesJson(costRulesInput.text.toString())

            val providersJsonStr = run {
                val arr = org.json.JSONArray()
                providers.forEach { l ->
                    val o = org.json.JSONObject()
                    o.put("type", l.type)
                    o.put("base_url", l.base_url)
                    o.put("model", l.model)
                    if (l.type == "openai" && !l.api_key.isNullOrBlank()) o.put("api_key", l.api_key)
                    if (l.type == "anthropic" && !l.api_key.isNullOrBlank()) o.put("api_key", l.api_key)
                    arr.put(o)
                }
                arr.toString()
            }

            val resultText: String = when (mode) {
                "Single" -> {
                    if (currentPreset.type == "openai") {
                        PantherBridge.validateOpenAI(prompt.text.toString(), apiKeyInput.text.toString().trim(), mdl, base)
                    } else if (currentPreset.type == "ollama") {
                        PantherBridge.validateOllama(prompt.text.toString(), base, mdl)
                    } else if (currentPreset.type == "anthropic") {
                        // Use multi with single entry
                        PantherBridge.validateMulti(prompt.text.toString(), providersJsonStr)
                    } else {
                        PantherBridge.validate(prompt.text.toString())
                    }
                }
                "Multi" -> {
                    if (useCustom && !guidelines.isNullOrBlank())
                        PantherBridge.validateCustom(prompt.text.toString(), providersJsonStr, guidelines)
                    else
                        PantherBridge.validateMulti(prompt.text.toString(), providersJsonStr)
                }
                else -> { // With Proof
                    if (useCustom && !guidelines.isNullOrBlank())
                        PantherBridge.validateCustomWithProof(prompt.text.toString(), providersJsonStr, guidelines)
                    else
                        PantherBridge.validateMultiWithProof(prompt.text.toString(), providersJsonStr)
                }
            }

            val sdk = PantherSDK.make(providers)
            val (lines, proof) = try {
                // Reuse existing parser/formatter for lines + cost
                sdk.validateAndGetProof(prompt.text.toString(), guidelines)
            } catch (_: Throwable) { listOf(resultText) to null }
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
