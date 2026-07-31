// 접근성 서비스 — UI 자동화 (화면 읽기, 탭, 스와이프)
// 원본: MinisAccessibilityService.kt (264줄) + NodeRegistry.kt (57줄)
// AndroidManifest에 서비스로 등록해야 함

package com.openminis.app

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo

class MinisAccessibilityService : AccessibilityService() {

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {}
    override fun onInterrupt() {}

    /// 화면의 현재 노드 트리를 JSON으로 반환
    fun readScreen(): String {
        val root = rootInActiveWindow ?: return "{}"
        val json = StringBuilder()
        nodeToJson(root, json, 0)
        return json.toString()
    }

    /// 좌표 탭
    fun tap(x: Float, y: Float): Boolean {
        val path = Path().apply { moveTo(x, y) }
        val gesture = GestureDescription.Builder()
            .addStroke(GestureDescription.StrokeDescription(path, 0, 100))
            .build()
        return dispatchGesture(gesture, null, null)
    }

    /// 텍스트 입력
    fun typeText(node: AccessibilityNodeInfo?, text: String) {
        val n = node ?: return
        val args = android.os.Bundle()
        args.putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, text)
        n.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
    }

    private fun nodeToJson(node: AccessibilityNodeInfo, sb: StringBuilder, depth: Int) {
        val indent = "  ".repeat(depth)
        sb.append("$indent{\"text\":\"${node.text ?: ""}\",")
        sb.append("\"class\":\"${node.className ?: ""}\",")
        sb.append("\"id\":\"${node.viewIdResourceName ?: ""}\",")
        sb.append("\"rect\":[${node.boundsInScreen.left},${node.boundsInScreen.top},${node.boundsInScreen.right},${node.boundsInScreen.bottom}]")
        if (node.childCount > 0) {
            sb.append(",\"children\":[")
            for (i in 0 until node.childCount) {
                if (i > 0) sb.append(",")
                node.getChild(i)?.let { nodeToJson(it, sb, depth + 1) }
            }
            sb.append("]")
        }
        sb.append("}")
    }
}