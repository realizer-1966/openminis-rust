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

    /// Rust JNI 진입점 — Rust 코어 초기화
    private external fun nativeInit(): Boolean

    /// Rust JNI — 셸 명령 실행
    private external fun nativeShell(command: String): String

    /// Rust JNI — bashism 감지
    private external fun nativeBashism(command: String): String

    /// Rust JNI — minis:// URL 해석
    private external fun nativeResolveUrl(url: String): String

    /// Rust JNI — 등록된 툴 목록
    private external fun nativeListTools(): String

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Rust 코어 초기화
        try {
            nativeInit()
        } catch (e: UnsatisfiedLinkError) {
            // .so 없이도 UI는 작동 (placeholder 응답)
        }

        // 로컬 HTTP 서버 시작 (별도 스레드)
        startLocalServer()

        // WebView 설정
        webView = WebView(this)
        val settings = webView.settings
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.allowFileAccess = true
        settings.cacheMode = WebSettings.LOAD_NO_CACHE

        webView.webViewClient = WebViewClient()
        webView.addJavascriptInterface(MinisBridge(), "MinisNative")

        // assets/index.html 로드
        webView.loadUrl("file:///android_asset/index.html")

        setContentView(webView)
    }

    inner class MinisBridge {
        @JavascriptInterface
        fun shell(command: String): String {
            return try { nativeShell(command) } catch(e: Exception) {
                """{"error":"${e.message}"}"""
            }
        }

        @JavascriptInterface
        fun bashism(command: String): String {
            return try { nativeBashism(command) } catch(e: Exception) {
                """{"error":"${e.message}"}"""
            }
        }

        @JavascriptInterface
        fun resolveUrl(url: String): String {
            return try { nativeResolveUrl(url) } catch(e: Exception) {
                """{"error":"${e.message}"}"""
            }
        }

        @JavascriptInterface
        fun listTools(): String {
            return try { nativeListTools() } catch(e: Exception) {
                """{"error":"${e.message}"}"""
            }
        }

        @JavascriptInterface
        fun serverPort(): Int = this@MainActivity.serverPort
    }

    /// 로컬 HTTP 서버 — assets의 index.html과 API 제공
    private fun startLocalServer() {
        CoroutineScope(Dispatchers.IO).launch {
            val server = ServerSocket(0) // 사용 가능한 포트 자동 할당
            serverPort = server.localPort
            while (!isFinishing) {
                try {
                    val socket = server.accept()
                    handleRequest(socket)
                } catch (e: Exception) { break }
            }
        }
    }

    private fun handleRequest(socket: java.net.Socket) {
        try {
            val reader = BufferedReader(InputStreamReader(socket.getInputStream()))
            val requestLine = reader.readLine() ?: return
            val headers = StringBuilder()
            while (true) {
                val line = reader.readLine() ?: break
                if (line.isEmpty()) break
                headers.appendLine(line)
            }

            val parts = requestLine.split(" ")
            val method = parts.getOrNull(0) ?: "GET"
            val path = parts.getOrNull(1) ?: "/"

            // Content-Length 읽기
            val contentLength = headers.lines()
                .find { it.lowercase().startsWith("content-length:") }
                ?.split(":")?.get(1)?.trim()?.toIntOrNull() ?: 0
            var body = ""
            if (contentLength > 0) {
                val buf = CharArray(contentLength)
                reader.read(buf, 0, contentLength)
                body = String(buf)
            }

            val (status, contentType, responseBody) = when {
                method == "OPTIONS" -> Triple("204 No Content", "text/plain", "")
                path == "/" -> {
                    val html = assets.open("index.html").bufferedReader().use { it.readText() }
                    Triple("200 OK", "text/html; charset=utf-8", html)
                }
                path == "/api/status" -> {
                    Triple("200 OK", "application/json",
                        """{"status":"running","tools_count":8,"port":$serverPort}""")
                }
                path == "/api/tools" -> {
                    val tools = try { nativeListTools() } catch(e: Exception) { "[]" }
                    Triple("200 OK", "application/json", """{"tools":$tools}""")
                }
                path == "/api/shell" && method == "POST" -> {
                    val cmd = JSONObject(body).optString("command", "")
                    val result = try { nativeShell(cmd) } catch(e: Exception) {
                        """{"error":"${e.message}"}"""
                    }
                    Triple("200 OK", "application/json", result)
                }
                path == "/api/bashism" && method == "POST" -> {
                    val cmd = JSONObject(body).optString("command", "")
                    val result = try { nativeBashism(cmd) } catch(e: Exception) {
                        """{"error":"${e.message}"}"""
                    }
                    Triple("200 OK", "application/json", result)
                }
                path == "/api/url" && method == "POST" -> {
                    val url = JSONObject(body).optString("url", "")
                    val result = try { nativeResolveUrl(url) } catch(e: Exception) {
                        """{"error":"${e.message}"}"""
                    }
                    Triple("200 OK", "application/json", result)
                }
                else -> Triple("404 Not Found", "application/json", """{"error":"not found"}""")
            }

            val response = "HTTP/1.1 $status\r\n" +
                "Content-Type: $contentType\r\n" +
                "Content-Length: ${responseBody.toByteArray().size}\r\n" +
                "Access-Control-Allow-Origin: *\r\n" +
                "Connection: close\r\n\r\n" +
                responseBody

            val writer = OutputStreamWriter(socket.getOutputStream())
            writer.write(response)
            writer.flush()
            socket.close()
        } catch (e: Exception) {
            try { socket.close() } catch(_: Exception) {}
        }
    }

    override fun onBackPressed() {
        if (webView.canGoBack()) webView.goBack() else super.onBackPressed()
    }
}