// 알림 리스너 서비스 — 활성 알림 읽기
// 원본: MinisNotificationListenerService.kt (130줄)
// AndroidManifest에 서비스로 등록 + 사용자가 권한 부여 필요

package com.openminis.app

import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification

class MinisNotificationListenerService : NotificationListenerService() {

    /// 활성 알림 목록을 JSON으로 반환
    fun listActiveNotifications(max: Int): String {
        val sb = StringBuilder("[")
        val notifications = getActiveNotifications()
        val count = minOf(notifications.size, max)
        for (i in 0 until count) {
            if (i > 0) sb.append(",")
            val n = notifications[i]
            val extras = n.notification.extras
            sb.append("{")
            sb.append("\"package\":\"${n.packageName}\",")
            sb.append("\"title\":\"${extras.getCharSequence("android.title") ?: ""}\",")
            sb.append("\"text\":\"${extras.getCharSequence("android.text") ?: ""}\",")
            sb.append("\"key\":\"${n.key}\"")
            sb.append("}")
        }
        sb.append("]")
        return sb.toString()
    }

    /// 모든 알림 삭제
    fun clearAll() {
        cancelAllNotifications()
    }
}