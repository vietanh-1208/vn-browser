-keepattributes *Annotation*
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}
-dontwarn javax.annotation.**
-dontwarn com.google.errorprone.annotations.**
