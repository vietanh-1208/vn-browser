package com.vnbrowser.app

import android.annotation.SuppressLint
import android.app.AlertDialog
import android.app.DownloadManager
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.graphics.Bitmap
import android.net.Uri
import android.os.Bundle
import android.os.Environment
import android.os.Message
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.webkit.*
import android.widget.*
import androidx.appcompat.app.AppCompatActivity
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewFeature
import org.json.JSONObject
import java.io.ByteArrayInputStream
import java.net.URLEncoder

data class BrowserTab(
    val id: Long,
    var title: String,
    var url: String,
    val incognito: Boolean,
    var blockedCount: Int = 0,
    val webView: WebView
)

class MainActivity : AppCompatActivity() {

    private lateinit var urlInput: EditText
    private lateinit var adBadge: TextView
    private lateinit var btnTabs: Button
    private lateinit var progressBar: ProgressBar
    private lateinit var webContainer: FrameLayout
    private lateinit var vault: PasswordVault

    private val tabs = mutableListOf<BrowserTab>()
    private var activeTabIndex = 0
    private var nextTabId = 1L
    private var adblockEnabled = true
    private var darkMode = true
    private var totalBlocked = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        vault = PasswordVault(this)
        urlInput = findViewById(R.id.urlInput)
        adBadge = findViewById(R.id.adBadge)
        btnTabs = findViewById(R.id.btnTabs)
        progressBar = findViewById(R.id.progressBar)
        webContainer = findViewById(R.id.webContainer)

        findViewById<ImageButton>(R.id.btnBack).setOnClickListener {
            val wv = currentTab()?.webView
            if (wv != null && wv.canGoBack()) wv.goBack()
        }
        btnTabs.setOnClickListener { showTabsDialog() }
        findViewById<ImageButton>(R.id.btnMenu).setOnClickListener { showMainMenu() }

        urlInput.setOnEditorActionListener { _, actionId, event ->
            if (actionId == EditorInfo.IME_ACTION_GO ||
                (event != null && event.keyCode == KeyEvent.KEYCODE_ENTER && event.action == KeyEvent.ACTION_DOWN)
            ) {
                navigateCurrent(urlInput.text.toString())
                hideKeyboard()
                true
            } else false
        }

        val initialUrl = intent?.dataString ?: "https://www.google.com"
        openNewTab(initialUrl, incognito = false)
    }

    private fun currentTab(): BrowserTab? = tabs.getOrNull(activeTabIndex)

    private fun hideKeyboard() {
        val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
        imm.hideSoftInputFromWindow(urlInput.windowToken, 0)
        urlInput.clearFocus()
    }

    private fun resolveAddress(raw: String): String {
        val t = raw.trim()
        if (t.isEmpty()) return "https://www.google.com"
        if (t.startsWith("http://") || t.startsWith("https://")) return t
        if (!t.contains(" ") && t.contains(".")) return "https://$t"
        return "https://www.google.com/search?q=" + URLEncoder.encode(t, "UTF-8")
    }

    private fun navigateCurrent(raw: String) {
        val target = resolveAddress(raw)
        currentTab()?.let {
            it.url = target
            it.webView.loadUrl(target)
        }
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun openNewTab(url: String, incognito: Boolean) {
        val tabId = nextTabId++
        val wv = WebView(this)
        wv.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = !incognito
            databaseEnabled = !incognito
            setSupportMultipleWindows(true)
            javaScriptCanOpenWindowsAutomatically = false
            useWideViewPort = true
            loadWithOverviewMode = true
            builtInZoomControls = true
            displayZoomControls = false
            mediaPlaybackRequiresUserGesture = false
            if (incognito) {
                cacheMode = WebSettings.LOAD_NO_CACHE
            }
        }
        if (darkMode && WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
            WebSettingsCompat.setAlgorithmicDarkeningAllowed(wv.settings, true)
        }

        val tab = BrowserTab(tabId, if (incognito) "Ẩn danh" else "Tab mới", url, incognito, 0, wv)

        wv.addJavascriptInterface(object {
            @JavascriptInterface
            fun onFormSubmit(site: String, username: String, password: String) {
                if (incognito || site.isBlank() || password.isBlank()) return
                runOnUiThread {
                    AlertDialog.Builder(this@MainActivity)
                        .setTitle("🔑 Lưu mật khẩu?")
                        .setMessage("Lưu tài khoản $username cho $site vào két sắt mã hoá?")
                        .setPositiveButton("Lưu") { _, _ ->
                            vault.upsert(site, username, password)
                            Toast.makeText(this@MainActivity, "Đã lưu mật khẩu cho $site", Toast.LENGTH_SHORT).show()
                        }
                        .setNegativeButton("Không", null)
                        .show()
                }
            }
        }, "VNBridge")

        wv.webViewClient = object : WebViewClient() {
            override fun shouldInterceptRequest(view: WebView?, request: WebResourceRequest?): WebResourceResponse? {
                val reqUrl = request?.url?.toString() ?: return null
                if (AdBlockEngine.shouldBlockRequest(reqUrl, tab.url, adblockEnabled)) {
                    tab.blockedCount++
                    totalBlocked++
                    runOnUiThread { updateChromeUi() }
                    return WebResourceResponse("text/plain", "utf-8", ByteArrayInputStream(ByteArray(0)))
                }
                return super.shouldInterceptRequest(view, request)
            }

            override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
                val target = request?.url?.toString() ?: return false
                if (AdBlockEngine.shouldBlockRequest(target, tab.url, adblockEnabled)) {
                    tab.blockedCount++
                    totalBlocked++
                    runOnUiThread {
                        updateChromeUi()
                        Toast.makeText(this@MainActivity, "🛡️ Đã chặn chuyển hướng quảng cáo", Toast.LENGTH_SHORT).show()
                    }
                    return true
                }
                return false
            }

            override fun onPageStarted(view: WebView?, pageUrl: String?, favicon: Bitmap?) {
                if (!pageUrl.isNullOrBlank()) tab.url = pageUrl
                progressBar.visibility = View.VISIBLE
                updateChromeUi()
            }

            override fun onPageFinished(view: WebView?, pageUrl: String?) {
                progressBar.visibility = View.GONE
                if (!pageUrl.isNullOrBlank()) tab.url = pageUrl
                view?.evaluateJavascript(
                    """
                    (() => {
                      if (window.__vnHooked) return;
                      window.__vnHooked = true;
                      document.addEventListener('submit', (e) => {
                        const f = e.target;
                        if (!f || !f.querySelector) return;
                        const pw = f.querySelector('input[type="password"]');
                        if (!pw || !pw.value) return;
                        const u = f.querySelector('input[type="email"],input[type="text"],input[type="tel"]');
                        if (window.VNBridge) {
                          window.VNBridge.onFormSubmit(location.hostname || '', u ? (u.value || '').trim() : '', pw.value);
                        }
                      }, true);
                    })();
                    """.trimIndent(),
                    null
                )
                updateChromeUi()
            }
        }

        wv.webChromeClient = object : WebChromeClient() {
            override fun onProgressChanged(view: WebView?, newProgress: Int) {
                progressBar.progress = newProgress
                if (newProgress >= 100) progressBar.visibility = View.GONE
            }

            override fun onReceivedTitle(view: WebView?, title: String?) {
                if (!title.isNullOrBlank()) {
                    tab.title = title
                }
            }

            override fun onCreateWindow(view: WebView?, isDialog: Boolean, isUserGesture: Boolean, resultMsg: Message?): Boolean {
                val href = view?.hitTestResult?.extra
                if (!isUserGesture || !AdBlockEngine.isAllowedPopup(tab.url, href)) {
                    tab.blockedCount++
                    totalBlocked++
                    updateChromeUi()
                    Toast.makeText(this@MainActivity, "🛡️ Đã chặn tab bật lên tự động", Toast.LENGTH_SHORT).show()
                    return false
                }
                if (!href.isNullOrBlank()) {
                    openNewTab(href, incognito)
                }
                return false
            }
        }

        wv.setDownloadListener { dlUrl, _, contentDisposition, mimeType, _ ->
            val req = DownloadManager.Request(Uri.parse(dlUrl))
            val fileName = URLUtil.guessFileName(dlUrl, contentDisposition, mimeType)
            req.setTitle(fileName)
            req.setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
            req.setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, fileName)
            val dm = getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager
            dm.enqueue(req)
            Toast.makeText(this, "⬇ Đang tải $fileName", Toast.LENGTH_SHORT).show()
        }

        tabs.add(tab)
        switchToTab(tabs.lastIndex)
        wv.loadUrl(resolveAddress(url))
    }

    private fun switchToTab(index: Int) {
        if (index !in tabs.indices) return
        activeTabIndex = index
        webContainer.removeAllViews()
        webContainer.addView(
            tabs[index].webView,
            FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
        )
        updateChromeUi()
    }

    private fun closeTab(index: Int) {
        if (tabs.size <= 1) {
            navigateCurrent("https://www.google.com")
            return
        }
        val removed = tabs.removeAt(index)
        removed.webView.destroy()
        val next = activeTabIndex.coerceAtMost(tabs.lastIndex)
        switchToTab(next)
    }

    private fun updateChromeUi() {
        val tab = currentTab() ?: return
        if (!urlInput.hasFocus()) {
            urlInput.setText(tab.url)
        }
        adBadge.text = if (adblockEnabled) "🛡️ ${tab.blockedCount}" else "🛡️ Tắt"
        btnTabs.text = tabs.size.toString()
    }

    private fun showTabsDialog() {
        val labels = tabs.mapIndexed { i, t ->
            val prefix = if (t.incognito) "🕵️ " else ""
            val cur = if (i == activeTabIndex) "✓ " else ""
            "$cur$prefix${t.title.take(36)}"
        }.toTypedArray()

        AlertDialog.Builder(this)
            .setTitle("Danh sách Tab (${tabs.size})")
            .setItems(labels) { _, which -> switchToTab(which) }
            .setPositiveButton("+ Tab mới") { _, _ -> openNewTab("https://www.google.com", false) }
            .setNeutralButton("+ Ẩn danh") { _, _ -> openNewTab("https://www.google.com", true) }
            .setNegativeButton("Đóng tab hiện tại") { _, _ -> closeTab(activeTabIndex) }
            .show()
    }

    private fun showMainMenu() {
        val items = arrayOf(
            "🔄 Tải lại trang",
            "🔑 Tự điền tài khoản trang này",
            "🛡️ Quản lý mật khẩu (Mở khoá bằng máy)",
            "📖 Chế độ đọc",
            if (adblockEnabled) "🚫 Tắt chặn quảng cáo" else "🛡️ Bật chặn quảng cáo",
            if (darkMode) "☀️ Tắt nền tối trang web" else "🌙 Bật nền tối trang web"
        )
        AlertDialog.Builder(this)
            .setTitle("VN Browser v1.1.0")
            .setItems(items) { _, which ->
                when (which) {
                    0 -> currentTab()?.webView?.reload()
                    1 -> authenticateDevice("Xác minh bằng khoá máy để tự điền mật khẩu") { autofillCurrentPage() }
                    2 -> authenticateDevice("Xác minh bằng khoá máy để xem Mật khẩu đã lưu") { showPasswordManager() }
                    3 -> toggleReaderMode()
                    4 -> {
                        adblockEnabled = !adblockEnabled
                        updateChromeUi()
                        currentTab()?.webView?.reload()
                    }
                    5 -> {
                        darkMode = !darkMode
                        if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
                            tabs.forEach { WebSettingsCompat.setAlgorithmicDarkeningAllowed(it.webView.settings, darkMode) }
                        }
                    }
                }
            }
            .show()
    }

    private fun authenticateDevice(subtitle: String, onSuccess: () -> Unit) {
        val bm = BiometricManager.from(this)
        val authenticators = BiometricManager.Authenticators.BIOMETRIC_STRONG or
            BiometricManager.Authenticators.DEVICE_CREDENTIAL
        if (bm.canAuthenticate(authenticators) != BiometricManager.BIOMETRIC_SUCCESS) {
            onSuccess()
            return
        }
        val executor = ContextCompat.getMainExecutor(this)
        val prompt = BiometricPrompt(this, executor, object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                onSuccess()
            }

            override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                Toast.makeText(this@MainActivity, "Đã huỷ xác minh", Toast.LENGTH_SHORT).show()
            }
        })
        val info = BiometricPrompt.PromptInfo.Builder()
            .setTitle("VN Browser - Bảo mật")
            .setSubtitle(subtitle)
            .setAllowedAuthenticators(authenticators)
            .build()
        prompt.authenticate(info)
    }

    private fun autofillCurrentPage() {
        val tab = currentTab() ?: return
        val host = AdBlockEngine.extractHost(tab.url)
        val cred = vault.findForHost(host)
        if (cred == null) {
            Toast.makeText(this, "Chưa có tài khoản lưu cho $host", Toast.LENGTH_SHORT).show()
            return
        }
        val uJson = JSONObject.quote(cred.username)
        val pJson = JSONObject.quote(cred.password)
        val js = """
            (() => {
              const setVal = (el, val) => {
                if (!el) return;
                el.value = val;
                el.dispatchEvent(new Event('input', { bubbles: true }));
                el.dispatchEvent(new Event('change', { bubbles: true }));
              };
              const pw = document.querySelector('input[type="password"]');
              const u = (pw && pw.form ? pw.form : document).querySelector('input[type="email"],input[type="text"],input[type="tel"]');
              setVal(u, $uJson);
              setVal(pw, $pJson);
            })();
        """.trimIndent()
        tab.webView.evaluateJavascript(js, null)
        Toast.makeText(this, "🔑 Đã điền tài khoản cho ${cred.site}", Toast.LENGTH_SHORT).show()
    }

    private fun showPasswordManager() {
        val list = vault.list()
        if (list.isEmpty()) {
            Toast.makeText(this, "Chưa có mật khẩu nào được lưu", Toast.LENGTH_SHORT).show()
            return
        }
        val labels = list.map { "${it.site}\n${it.username}" }.toTypedArray()
        AlertDialog.Builder(this)
            .setTitle("🔑 Mật khẩu đã lưu (${list.size})")
            .setItems(labels) { _, which ->
                val c = list[which]
                AlertDialog.Builder(this)
                    .setTitle(c.site)
                    .setMessage("Tài khoản: ${c.username}\nMật khẩu: ${c.password}")
                    .setPositiveButton("Sao chép MK") { _, _ ->
                        val cm = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        cm.setPrimaryClip(ClipData.newPlainText("password", c.password))
                        Toast.makeText(this, "✓ Đã sao chép mật khẩu", Toast.LENGTH_SHORT).show()
                    }
                    .setNegativeButton("Xoá") { _, _ ->
                        vault.delete(c.id)
                        Toast.makeText(this, "Đã xoá tài khoản", Toast.LENGTH_SHORT).show()
                    }
                    .setNeutralButton("Đóng", null)
                    .show()
            }
            .show()
    }

    private fun toggleReaderMode() {
        currentTab()?.webView?.evaluateJavascript(
            """
            (() => {
              const old = document.getElementById('__vnReader');
              if (old) { old.remove(); return; }
              const art = document.querySelector('article, main, [role=main]') || document.body;
              const ov = document.createElement('div');
              ov.id = '__vnReader';
              ov.style = 'position:fixed;inset:0;z-index:2147483647;background:#0f172a;color:#e2e8f0;overflow:auto;padding:24px 16px;font-size:18px;line-height:1.7;';
              ov.innerHTML = '<button onclick="this.parentElement.remove()" style="padding:8px 14px;border-radius:8px;border:none;background:#10b981;color:#fff;font-weight:700;margin-bottom:16px">✕ Thoát</button><h2>' + document.title + '</h2><div>' + art.innerHTML + '</div>';
              document.body.appendChild(ov);
            })();
            """.trimIndent(),
            null
        )
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        val wv = currentTab()?.webView
        if (wv != null && wv.canGoBack()) {
            wv.goBack()
        } else {
            super.onBackPressed()
        }
    }
}
