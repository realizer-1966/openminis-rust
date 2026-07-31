// Shizuku 브릿지 — 권한이 필요한 시스템 API 호출
// 원본: ShizukuManager.kt (135줄) + ShizukuBackend.kt (259줄)
// → 통합 (~80줄)

package com.openminis.app

import rikka.shizuku.Shizuku

class ShizukuBridge {

    enum class State { NOT_INSTALLED, NOT_RUNNING, NEED_PERMISSION, READY }

    fun state(): State {
        if (!Shizuku.pingBinder()) return State.NOT_RUNNING
        if (!Shizuku.checkSelfPermission()) return State.NEED_PERMISSION
        return State.READY
    }

    /// Shizuku 권한으로 셸 명령 실행
    fun exec(command: String): String {
        val process = Shizuku.newProcess(arrayOf("sh", "-c", command), null, "/")
        val output = process.inputStream.bufferedReader().readText()
        process.waitFor()
        return output
    }
}