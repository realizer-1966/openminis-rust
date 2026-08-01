package com.openminis.app

import android.os.Bundle
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.JavascriptInterface
import android.webkit.WebSettings
import androidx.appcompat.app.AppCompatActivity
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.net.ServerSocket
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter

class MainActivity : AppCompatActivity() {

    companion object {
        init {
            System.loadLibrary("minis_core")
        }
    }

    private lateinit var webView: WebView
    private var serverPort: Int = 8765

    private external fun nativeInit(): Boolean
    private external fun nativeShell(command: String): String
    private external fun nativeBashism(command: String): String
    private external fun nativeResolveUrl(url: String): String
    private external fun nativeListTools(): String

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        try { nativeInit() } catch(e: UnsatisfiedLinkError) {}
        startLocalServer()

        webView = WebView(this)
        val settings = webView.settings
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.allowFileAccess = true
        settings.cacheMode = WebSettings.LOAD_NO_CACHE
        webView.webViewClient = WebViewClient()
        webView.addJavascriptInterface(MinisBridge(), "MinisNative")
        webView.loadUrl("file:///android_asset/index.html")
        setContentView(webView)
    }

    inner class MinisBridge {
        @JavascriptInterface
        fun shell(command: String): String {
            return try { nativeShell(command) } catch(e: Exception) { """{"error":"${e.message}"}""" }
        }
        @JavascriptInterface
        fun bashism(command: String): String {
            return try { nativeBashism(command) } catch(e: Exception) { """{"error":"${e.message}"}""" }
        }
        @JavascriptInterface
        fun resolveUrl(url: String): String {
            return try { nativeResolveUrl(url) } catch(e: Exception) { """{"error":"${e.message}"}""" }
        }
        @JavascriptInterface
        fun listTools(): String {
            return try { nativeListTools() } catch(e: Exception) { """{"error":"${e.message}"}""" }
        }
        @JavascriptInterface
        fun serverPort(): Int = this@MainActivity.serverPort
    }

    private fun startLocalServer() {
        CoroutineScope(Dispatchers.IO).launch {
            val server = ServerSocket(0)
            serverPort = server.localPort
            while (!isFinishing) {
                try { handleRequest(server.accept()) } catch(e: Exception) { break }
            }
        }
    }

    private fun handleRequest(socket: java.net.Socket) {
        try {
            val reader = BufferedReader(InputStreamReader(socket.getInputStream()))
            val requestLine = reader.readLine() ?: return
            val headers = StringBuilder()
            while (true) { val l = reader.readLine() ?: break; if (l.isEmpty()) break; headers.appendLine(l) }
            val parts = requestLine.split(" ")
            val method = parts.getOrNull(0) ?: "GET"
            val path = parts.getOrNull(1) ?: "/"
            val cl = headers.lines().find { it.lowercase().startsWith("content-length:") }?.split(":")?.get(1)?.trim()?.toIntOrNull() ?: 0
            var body = ""
            if (cl > 0) { val buf = CharArray(cl); reader.read(buf, 0, cl); body = String(buf) }

            val (status, ct, rb) = when {
                method == "OPTIONS" -> Triple("204 No Content", "text/plain", "")
                path == "/" -> Triple("200 OK", "text/html", assets.open("index.html").bufferedReader().use { it.readText() })
                path == "/api/status" -> Triple("200 OK", "application/json", """{"status":"running","port":$serverPort}""")
                path == "/api/tools" -> Triple("200 OK", "application/json", """{"tools":${try{nativeListTools()}catch(e:Exception){"[]"}}}""")
                path == "/api/shell" && method == "POST" -> Triple("200 OK", "application/json", try { nativeShell(JSONObject(body).optString("command","")) } catch(e:Exception) { """{"error":"${e.message}"}""" })
                path == "/api/bashism" && method == "POST" -> Triple("200 OK", "application/json", try { nativeBashism(JSONObject(body).optString("command","")) } catch(e:Exception) { """{"error":"${e.message}"}""" })
                path == "/api/url" && method == "POST" -> Triple("200 OK", "application/json", try { nativeResolveUrl(JSONObject(body).optString("url","")) } catch(e:Exception) { """{"error":"${e.message}"}""" })
                else -> Triple("404 Not Found", "application/json", """{"error":"not found"}""")
            }
            val resp = "HTTP/1.1 $status\r\nContent-Type: $ct\r\nContent-Length: ${rb.toByteArray().size}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n$rb"
            OutputStreamWriter(socket.getOutputStream()).use { it.write(resp); it.flush() }
            socket.close()
        } catch(e: Exception) { try { socket.close() } catch(_: Exception) {} }
    }

    override fun onBackPressed() { if (webView.canGoBack()) webView.goBack() else super.onBackPressed() }
}