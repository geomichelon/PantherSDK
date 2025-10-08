package com.example.panther

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val tv = TextView(this)
        setContentView(tv)
        PantherBridge.pantherInit()
        val res = PantherBridge.pantherGenerate("hello from android")
        tv.text = res
    }
}

