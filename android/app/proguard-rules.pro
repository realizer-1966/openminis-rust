# OpenMinis ProGuard rules

# Rust 네이티브 라이브러리 보호
-keep class com.openminis.app.** { *; }

# JNI 메서드 보호
-keepclasseswithmembernames class * {
    native <methods>;
}

# WebView JavascriptInterface 보호
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}