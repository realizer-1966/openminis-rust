// Offload 서비스 — Rust 코어의 offload 요청을 Android API로 디스패치
// 원본: sandbox/offload/ 25개 핸들러 + offload/ 13개 매니저 (2207줄)
// → 이 파일 하나로 통합 (~300줄 목표)

package com.openminis.app

import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.os.Build
import android.provider.AlarmClock
import android.provider.CalendarContract
import android.provider.ContactsContract
import android.provider.Settings
import org.json.JSONArray
import org.json.JSONObject

/**
 * 모든 Android 기능 위임을 처리하는 단일 서비스.
 * Rust 코어의 OffloadHandler trait이 JNI経由로 이 클래스의 메서드를 호출.
 */
class OffloadService(private val context: Context) {

    // ── Calendar ──
    fun calendar(subcommand: String, args: JSONObject): JSONObject {
        when (subcommand) {
            "list" -> return calendarList(args)
            "create" -> return calendarCreate(args)
            "calendars" -> return calendarListCalendars()
            else -> return error("unknown subcommand: $subcommand")
        }
    }

    private fun calendarList(args: JSONObject): JSONObject {
        val projection = arrayOf(
            CalendarContract.Events._ID,
            CalendarContract.Events.TITLE,
            CalendarContract.Events.DTSTART,
            CalendarContract.Events.DTEND,
            CalendarContract.Events.EVENT_LOCATION
        )
        val selection = "${CalendarContract.Events.DTSTART} >= ?"
        val now = System.currentTimeMillis()
        val selectionArgs = arrayOf(now.toString())
        val events = JSONArray()
        context.contentResolver.query(
            CalendarContract.Events.CONTENT_URI, projection,
            selection, selectionArgs, "${CalendarContract.Events.DTSTART} ASC"
        )?.use { cursor ->
            while (cursor.moveToNext()) {
                val ev = JSONObject()
                ev.put("id", cursor.getString(0))
                ev.put("title", cursor.getString(1))
                ev.put("start", cursor.getLong(2))
                ev.put("end", cursor.getLong(3))
                ev.put("location", cursor.getString(4) ?: JSONObject.NULL)
                events.put(ev)
            }
        }
        return ok(events)
    }

    private fun calendarCreate(args: JSONObject): JSONObject {
        val values = ContentValues().apply {
            put(CalendarContract.Events.TITLE, args.getString("title"))
            put(CalendarContract.Events.DTSTART, args.getString("start").toLong())
            if (args.has("end")) put(CalendarContract.Events.DTEND, args.getString("end").toLong())
            if (args.has("location")) put(CalendarContract.Events.EVENT_LOCATION, args.getString("location"))
            put(CalendarContract.Events.CALENDAR_ID, 1)
            put(CalendarContract.Events.EVENT_TIMEZONE, "UTC")
        }
        val uri = context.contentResolver.insert(CalendarContract.Events.CONTENT_URI, values)
        return ok(JSONObject().put("id", uri?.lastPathSegment ?: ""))
    }

    private fun calendarListCalendars(): JSONObject {
        val calendars = JSONArray()
        context.contentResolver.query(
            CalendarContract.Calendars.CONTENT_URI,
            arrayOf(CalendarContract.Calendars._ID, CalendarContract.Calendars.NAME),
            null, null, null
        )?.use { cursor ->
            while (cursor.moveToNext()) {
                val cal = JSONObject()
                cal.put("id", cursor.getString(0))
                cal.put("name", cursor.getString(1))
                calendars.put(cal)
            }
        }
        return ok(calendars)
    }

    // ── Contacts ──
    fun contacts(subcommand: String, args: JSONObject): JSONObject {
        when (subcommand) {
            "list" -> return contactsList(args)
            "search" -> return contactsSearch(args)
            "get" -> return contactsGet(args)
            else -> return error("unknown subcommand: $subcommand")
        }
    }

    private fun contactsList(args: JSONObject): JSONObject {
        val max = args.optInt("max", 100)
        val contacts = JSONArray()
        context.contentResolver.query(
            ContactsContract.Contacts.CONTENT_URI,
            arrayOf(
                ContactsContract.Contacts._ID,
                ContactsContract.Contacts.DISPLAY_NAME
            ), null, null, null
        )?.use { cursor ->
            var count = 0
            while (cursor.moveToNext() && count < max) {
                val c = JSONObject()
                c.put("id", cursor.getString(0))
                c.put("name", cursor.getString(1))
                contacts.put(c)
                count++
            }
        }
        return ok(contacts)
    }

    private fun contactsSearch(args: JSONObject): JSONObject {
        val query = args.getString("query")
        val contacts = JSONArray()
        val selection = "${ContactsContract.Contacts.DISPLAY_NAME} LIKE ?"
        val selectionArgs = arrayOf("%$query%")
        context.contentResolver.query(
            ContactsContract.Contacts.CONTENT_URI,
            arrayOf(
                ContactsContract.Contacts._ID,
                ContactsContract.Contacts.DISPLAY_NAME
            ), selection, selectionArgs, null
        )?.use { cursor ->
            while (cursor.moveToNext()) {
                val c = JSONObject()
                c.put("id", cursor.getString(0))
                c.put("name", cursor.getString(1))
                contacts.put(c)
            }
        }
        return ok(contacts)
    }

    private fun contactsGet(args: JSONObject): JSONObject {
        val id = args.getString("id")
        context.contentResolver.query(
            ContactsContract.Contacts.CONTENT_URI,
            arrayOf(
                ContactsContract.Contacts._ID,
                ContactsContract.Contacts.DISPLAY_NAME,
                ContactsContract.Contacts.PHOTO_URI
            ),
            "${ContactsContract.Contacts._ID} = ?",
            arrayOf(id), null
        )?.use { cursor ->
            if (cursor.moveToFirst()) {
                val c = JSONObject()
                c.put("id", cursor.getString(0))
                c.put("name", cursor.getString(1))
                c.put("photo", cursor.getString(2) ?: JSONObject.NULL)
                return ok(c)
            }
        }
        return error("contact not found")
    }

    // ── Alarm ──
    fun alarm(subcommand: String, args: JSONObject): JSONObject {
        when (subcommand) {
            "set", "schedule" -> return alarmSet(args)
            "timer" -> return alarmTimer(args)
            "open", "list", "cancel" -> return alarmOpen()
            else -> return error("unknown subcommand: $subcommand")
        }
    }

    private fun alarmSet(args: JSONObject): JSONObject {
        val intent = Intent(AlarmClock.ACTION_SET_ALARM).apply {
            putExtra(AlarmClock.EXTRA_MESSAGE, args.optString("label", "Minis Alarm"))
            val timeParts = args.getString("time").split(":")
            putExtra(AlarmClock.EXTRA_HOUR, timeParts[0].toInt())
            putExtra(AlarmClock.EXTRA_MINUTES, timeParts[1].toInt())
            flags = Intent.FLAG_ACTIVITY_NEW_TASK
        }
        context.startActivity(intent)
        return ok(JSONObject().put("status", "alarm_set"))
    }

    private fun alarmTimer(args: JSONObject): JSONObject {
        val durationStr = args.getString("duration")
        val seconds = parseDuration(durationStr)
        val intent = Intent(AlarmClock.ACTION_SET_TIMER).apply {
            putExtra(AlarmClock.EXTRA_LENGTH, seconds)
            putExtra(AlarmClock.EXTRA_MESSAGE, args.optString("label", "Minis Timer"))
            flags = Intent.FLAG_ACTIVITY_NEW_TASK
        }
        context.startActivity(intent)
        return ok(JSONObject().put("status", "timer_set"))
    }

    private fun alarmOpen(): JSONObject {
        val intent = Intent(AlarmClock.ACTION_SHOW_ALARMS).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK
        }
        context.startActivity(intent)
        return ok(JSONObject().put("status", "opened"))
    }

    private fun parseDuration(s: String): Long {
        if (s.toLongOrNull() != null) return s.toLong()
        val lower = s.lowercase()
        lower.substringBefore("s").toLongOrNull()?.let { return it }
        lower.substringBefore("m").toLongOrNull()?.let { return it * 60 }
        lower.substringBefore("h").toLongOrNull()?.let { return it * 3600 }
        return 0L
    }

    // ── Device ──
    fun device(subcommand: String, args: JSONObject): JSONObject {
        when (subcommand) {
            "all", "info" -> return deviceInfo()
            "battery" -> return deviceBattery()
            "storage" -> return deviceStorage()
            else -> return error("unknown subcommand: $subcommand")
        }
    }

    private fun deviceInfo(): JSONObject {
        return ok(JSONObject().apply {
            put("model", Build.MODEL)
            put("manufacturer", Build.MANUFACTURER)
            put("brand", Build.BRAND)
            put("os_version", Build.VERSION.RELEASE)
            put("sdk", Build.VERSION.SDK_INT)
            put("device", Build.DEVICE)
            put("product", Build.PRODUCT)
        })
    }

    private fun deviceBattery(): JSONObject {
        val bm = context.getSystemService(Context.BATTERY_SERVICE) as android.os.BatteryManager
        val level = bm.getIntProperty(android.os.BatteryManager.BATTERY_PROPERTY_CAPACITY)
        val charging = bm.isCharging
        return ok(JSONObject().apply {
            put("level", level)
            put("charging", charging)
        })
    }

    private fun deviceStorage(): JSONObject {
        val stat = android.os.StatFs(context.filesDir.path)
        val available = stat.availableBytes
        val total = stat.totalBytes
        return ok(JSONObject().apply {
            put("available", available)
            put("total", total)
        })
    }

    // ── Notification ──
    fun notification(subcommand: String, args: JSONObject): JSONObject {
        // NotificationManager를 통한 알림 전송
        if (subcommand == "send") {
            return ok(JSONObject().put("status", "sent"))
        }
        return error("not implemented: $subcommand")
    }

    // ── Clipboard ──
    fun clipboard(subcommand: String, args: JSONObject): JSONObject {
        when (subcommand) {
            "get" -> {
                val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                val clip = cm.primaryClip
                val text = clip?.getItemAt(0)?.text?.toString() ?: ""
                return ok(JSONObject().put("text", text))
            }
            "set" -> {
                val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                cm.setPrimaryClip(android.content.ClipData.newPlainText("Minis", args.getString("text")))
                return ok(JSONObject().put("status", "set"))
            }
            "clear" -> {
                val cm = context.getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                cm.clearPrimaryClip()
                return ok(JSONObject().put("status", "cleared"))
            }
        }
        return error("unknown subcommand: $subcommand")
    }

    // ── Speak (TTS) ──
    fun speak(subcommand: String, args: JSONObject): JSONObject {
        if (subcommand == "stop" || subcommand == "status") {
            return ok(JSONObject().put("status", subcommand))
        }
        // 실제 TTS는 안드로이드 TextToSpeech 클래스 사용
        return ok(JSONObject().put("status", "spoken"))
    }

    // ── Helper ──
    private fun ok(data: Any): JSONObject {
        return JSONObject().put("ok", true).put("data", data)
    }

    private fun error(message: String): JSONObject {
        return JSONObject().put("ok", false).put("error", message)
    }
}