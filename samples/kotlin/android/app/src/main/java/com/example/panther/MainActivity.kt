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
            addView(btnValidate)
            addView(outputView)
        }
        setContentView(layout)

        PantherBridge.pantherInit()

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
            }
            outputView.text = buf.toString()
        }
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
