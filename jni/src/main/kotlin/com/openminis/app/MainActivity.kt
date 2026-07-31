// Tauri Android 호스트 액티비티 — Rust 코어 로드 + WebView 호스팅
// 원본: MainActivity.kt (300+줄)에서 Tauri 전용으로 최소화
//
// Tauri Mobile이 생성하는 기본 액티비티를 확장하여:
// 1. Rust 네이티브 라이브러리 로드
// 2. offload 콜백 등록 (Rust → Kotlin → Android API)
// 3. 권한 요청 처리

package com.openminis.app

import android.os.Bundle
import androidx.activity.ComponentActivity

class MainActivity : ComponentActivity() {

    companion object {
        init {
            // Rust 네이티브 라이브러리 로드
            System.loadLibrary("openminis")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Rust 코어 초기화 — offload 콜백 등록
        nativeInitOffloadCallbacks(
            CalendarOffload(),
            ContactsOffload(),
            AlarmOffload(),
            LocationOffload(),
            DeviceOffload(),
            NotificationOffload(),
            PhotosOffload(),
            ClipboardOffload(),
            SpeakOffload(),
            SpeechOffload(),
            ShizukuOffload(),
        )

        // Tauri WebView는 플러그인 시스템으로 자동 설정됨
    }

    // ── JNI 진입점 (Rust src-tauri에서 호출) ──
    private external fun nativeInitOffloadCallbacks(
        vararg handlers: OffloadCallback
    )

    // ── Offload 콜백 인터페이스 ──
    interface OffloadCallback {
        val name: String
        fun execute(subcommand: String, argsJson: String): String
    }
}