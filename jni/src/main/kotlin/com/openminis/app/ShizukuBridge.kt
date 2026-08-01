package com.openminis.app

/**
 * Shizuku 브릿지 — 권한이 필요한 시스템 API 호출
 * Shizuku SDK가 없는 환경에서는 placeholder로 작동
 */
class ShizukuBridge {

    enum class State { NOT_INSTALLED, NOT_RUNNING, NEED_PERMISSION, READY }

    fun state(): State {
        // Shizuku SDK 의존성이 없으면 NOT_INSTALLED 반환
        return State.NOT_INSTALLED
    }

    fun exec(command: String): String {
        // Shizuku 없이는 실행 불가
        return "Shizuku not available"
    }
}