package com.vnbrowser.app

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import org.json.JSONArray
import org.json.JSONObject

data class SavedCred(
    val id: String,
    val site: String,
    val username: String,
    val password: String
)

class PasswordVault(context: Context) {
    private val prefs: SharedPreferences

    init {
        val masterKey = MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        prefs = EncryptedSharedPreferences.create(
            context,
            "vn_browser_vault",
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )
    }

    fun list(): MutableList<SavedCred> {
        val raw = prefs.getString("credentials", "[]") ?: "[]"
        val arr = JSONArray(raw)
        val out = mutableListOf<SavedCred>()
        for (i in 0 until arr.length()) {
            val o = arr.getJSONObject(i)
            out.add(
                SavedCred(
                    id = o.optString("id"),
                    site = o.optString("site"),
                    username = o.optString("username"),
                    password = o.optString("password")
                )
            )
        }
        return out
    }

    private fun saveAll(items: List<SavedCred>) {
        val arr = JSONArray()
        for (c in items) {
            arr.put(JSONObject().apply {
                put("id", c.id)
                put("site", c.site)
                put("username", c.username)
                put("password", c.password)
            })
        }
        prefs.edit().putString("credentials", arr.toString()).apply()
    }

    fun upsert(site: String, username: String, password: String) {
        val items = list()
        val idx = items.indexOfFirst {
            it.site.equals(site, ignoreCase = true) && it.username == username
        }
        if (idx >= 0) {
            items[idx] = items[idx].copy(password = password)
        } else {
            items.add(0, SavedCred("cred_${System.currentTimeMillis()}", site, username, password))
        }
        saveAll(items)
    }

    fun delete(id: String) {
        val items = list().filterNot { it.id == id }
        saveAll(items)
    }

    fun findForHost(host: String): SavedCred? {
        if (host.isBlank()) return null
        return list().firstOrNull {
            it.site.equals(host, ignoreCase = true) ||
                host.endsWith(".${it.site}", ignoreCase = true) ||
                it.site.endsWith(".${host}", ignoreCase = true)
        }
    }
}
