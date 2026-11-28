package com.vnbrowser.app

import android.net.Uri
import java.util.Locale

object AdBlockEngine {
    private val blockedHosts = listOf(
        "doubleclick.net", "googlesyndication.com", "googleadservices.com",
        "adservice.google.com", "pagead2.googlesyndication.com",
        "taboola.com", "outbrain.com", "mgid.com", "adsterra.com",
        "popads.net", "popcash.net", "propellerads.com", "exoclick.com",
        "juicyads.com", "trafficjunky.com", "onclickads.net", "adnxs.com",
        "criteo.com", "pubmatic.com", "rubiconproject.com", "openx.net",
        "qq88", "xx8", "8xbet", "kubet", "jun88", "hi88", "f8bet",
        "789bet", "shbet", "mb66", "new88", "okvip", "bong88", "fb88",
        "fun88", "w88", "m88", "188bet", "bk8"
    )

    private val oauthHosts = listOf(
        "accounts.google.com", "appleid.apple.com", "github.com",
        "login.microsoftonline.com", "login.live.com", "discord.com"
    )

    fun extractHost(url: String?): String {
        if (url.isNullOrBlank()) return ""
        return try {
            (Uri.parse(url).host ?: "").lowercase(Locale.ROOT)
        } catch (_: Exception) {
            ""
        }
    }

    private fun isYouTubeMediaHost(host: String): Boolean {
        return host.endsWith(".googlevideo.com") || host.endsWith(".ytimg.com")
    }

    fun shouldBlockRequest(requestUrl: String, pageUrl: String?, adblockEnabled: Boolean): Boolean {
        if (!adblockEnabled) return false
        val lower = requestUrl.lowercase(Locale.ROOT)
        if (lower.contains("youtube.com/api/stats/ads") ||
            lower.contains("youtube.com/pagead/") ||
            lower.contains("get_midroll_info")
        ) {
            return true
        }
        val reqHost = extractHost(requestUrl)
        if (reqHost.isEmpty() || isYouTubeMediaHost(reqHost)) return false

        if (blockedHosts.any { reqHost == it || reqHost.endsWith(".$it") || reqHost.contains(it) }) {
            return true
        }
        return lower.contains("/ads/") || lower.contains("banner_ad") ||
            lower.contains("popunder") || lower.contains("affiliatecode=")
    }

    fun isAllowedPopup(sourceUrl: String?, targetUrl: String?): Boolean {
        val targetHost = extractHost(targetUrl)
        if (targetHost.isEmpty()) return false
        if (blockedHosts.any { targetHost.contains(it) }) return false
        if (oauthHosts.any { targetHost == it || targetHost.endsWith(".$it") }) return true

        val srcHost = extractHost(sourceUrl)
        if (srcHost.isEmpty()) return true
        return targetHost == srcHost ||
            targetHost.endsWith(".$srcHost") ||
            srcHost.endsWith(".$targetHost")
    }
}
