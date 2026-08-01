package com.openminis.app

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.graphics.Rect
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo

class MinisAccessibilityService : AccessibilityService() {

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {}
    override fun onInterrupt() {}

    fun readScreen(): String {
        val root = rootInActiveWindow ?: return "{}"
        val json = StringBuilder()
        nodeToJson(root, json)
        return json.toString()
    }

    fun tap(x: Float, y: Float): Boolean {
        val path = Path().apply { moveTo(x, y) }
        val gesture = GestureDescription.Builder()
            .addStroke(GestureDescription.StrokeDescription(path, 0, 100))
            .build()
        return dispatchGesture(gesture, null, null)
    }

    fun typeText(node: AccessibilityNodeInfo?, text: String) {
        val n = node ?: return
        val args = android.os.Bundle()
        args.putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, text)
        n.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
    }

    private fun nodeToJson(node: AccessibilityNodeInfo, sb: StringBuilder) {
        val bounds = Rect()
        node.getBoundsInScreen(bounds)
        sb.append("{\"text\":\"${node.text ?: ""}\",")
        sb.append("\"class\":\"${node.className ?: ""}\",")
        sb.append("\"id\":\"${node.viewIdResourceName ?: ""}\",")
        sb.append("\"rect\":[${bounds.left},${bounds.top},${bounds.right},${bounds.bottom}]")
        if (node.childCount > 0) {
            sb.append(",\"children\":[")
            for (i in 0 until node.childCount) {
                if (i > 0) sb.append(",")
                node.getChild(i)?.let { nodeToJson(it, sb) }
            }
            sb.append("]")
        }
        sb.append("}")
    }
}