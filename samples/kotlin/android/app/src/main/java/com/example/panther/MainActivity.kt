package com.example.panther

import android.os.Bundle
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.EditText
import android.widget.Button
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val layout = LinearLayout(this)
        layout.orientation = LinearLayout.VERTICAL
        val btnRecord = Button(this).apply { text = "Record Metric" }
        val btnList = Button(this).apply { text = "List Items" }
        val btnLogs = Button(this).apply { text = "Get Logs" }
        val tv = TextView(this)
        layout.addView(btnRecord)
        layout.addView(btnList)
        layout.addView(btnLogs)
        layout.addView(tv)
        setContentView(layout)

        PantherBridge.pantherInit()

        btnRecord.setOnClickListener { PantherBridge.recordMetric("button_press") }
        btnList.setOnClickListener { tv.text = PantherBridge.listStorageItems() }
        btnLogs.setOnClickListener { tv.text = PantherBridge.getLogs() }
    }
}
