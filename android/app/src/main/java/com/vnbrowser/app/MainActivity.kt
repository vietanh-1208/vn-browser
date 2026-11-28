package com.vnbrowser.app

import android.Manifest
import android.annotation.SuppressLint
import android.app.AlertDialog
import android.app.DownloadManager
import android.app.PictureInPictureParams
import android.content.BroadcastReceiver
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageManager
import android.content.res.Configuration
import android.graphics.Bitmap
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.Handler
import android.os.Looper
import android.os.Message
import android.text.Editable
import android.text.TextWatcher
import android.util.Rational
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.view.WindowManager
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.webkit.*
import android.widget.*
import androidx.appcompat.app.AppCompatActivity
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import com.google.android.material.bottomsheet.BottomSheetDialog
import org.json.JSONArray
import org.json.JSONObject
import java.io.ByteArrayInputStream
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

class BgPlayWebView(context: Context) : WebView(context) {
    var allowBgAudio: Boolean = true
    override fun onWindowVisibilityChanged(visibility: Int) {
        if (allowBgAudio && visibility != View.VISIBLE) {
            super.onWindowVisibilityChanged(View.VISIBLE)
        } else {
            super.onWindowVisibilityChanged(visibility)
        }
    }
}

data class BrowserTab(
    val id: Long,
    var title: String,
    var url: String,
    val incognito: Boolean,
    var blockedCount: Int = 0,
    var hasVideo: Boolean = false,
    val webView: BgPlayWebView
)

class MainActivity : AppCompatActivity() {

    companion object {
        private const val NEWTAB_URL = "https://vn.newtab/"
        private const val HISTORY_URL = "https://vn.history/"
        private const val DOWNLOADS_URL = "https://vn.downloads/"
        private const val PASSWORDS_URL = "https://vn.passwords/"
        private const val SETTINGS_URL = "https://vn.settings/"

        private const val ANDROID_BG_MEDIA_SCRIPT = """
            (function() {
              if (window.__vnBgMediaHooked) return;
              window.__vnBgMediaHooked = true;
              try {
                Object.defineProperty(Document.prototype, 'hidden', { get: () => false, configurable: true });
                Object.defineProperty(Document.prototype, 'webkitHidden', { get: () => false, configurable: true });
                Object.defineProperty(Document.prototype, 'visibilityState', { get: () => 'visible', configurable: true });
                Object.defineProperty(Document.prototype, 'webkitVisibilityState', { get: () => 'visible', configurable: true });
              } catch (e) {}

              const blockVis = (orig) => function(type, listener, opts) {
                if (type === 'visibilitychange' || type === 'webkitvisibilitychange') {
                  if (window.__vnAllowBgPlay !== false) return;
                }
                return orig.call(this, type, listener, opts);
              };
              try {
                Document.prototype.addEventListener = blockVis(Document.prototype.addEventListener);
                Window.prototype.addEventListener = blockVis(Window.prototype.addEventListener);
              } catch (e) {}

              setInterval(() => {
                try {
                  if (location.hostname.endsWith('youtube.com')) {
                    window._lact = Date.now();
                  }
                  const hasV = Boolean(document.querySelector('video'));
                  if (window.__vnLastHasV !== hasV && window.VNBridge && window.VNBridge.onVideoDetected) {
                    window.__vnLastHasV = hasV;
                    window.VNBridge.onVideoDetected(hasV);
                  }
                } catch (e) {}
              }, 1200);
            })();
        """
    }

    private lateinit var rootFrame: FrameLayout
    private lateinit var mainColumn: LinearLayout
    private lateinit var urlInput: EditText
    private lateinit var topBar: LinearLayout
    private lateinit var bottomDivider: View
    private lateinit var bottomBar: LinearLayout
    private lateinit var omniboxPill: LinearLayout
    private lateinit var btnHome: ImageButton
    private lateinit var btnReload: ImageButton
    private lateinit var btnNewTab: ImageButton
    private lateinit var btnMenu: ImageButton
    private lateinit var btnPip: TextView
    private lateinit var btnTranslate: TextView
    private lateinit var adBadge: TextView
    private lateinit var btnTabs: TextView
    private lateinit var btnBack: ImageButton
    private lateinit var btnForward: ImageButton
    private lateinit var progressBar: ProgressBar
    private lateinit var webContainer: FrameLayout
    private lateinit var suggestList: ListView
    private lateinit var tabSwitcherOverlay: LinearLayout
    private lateinit var tabSwitcherHeader: LinearLayout
    private lateinit var tabSwitcherTitle: TextView
    private lateinit var tabGrid: GridView
    private lateinit var tabGridAdapter: BaseAdapter
    private lateinit var vault: PasswordVault
    private val mainHandler = Handler(Looper.getMainLooper())
    private var suggestRunnable: Runnable? = null
    private val suggestions = mutableListOf<String>()
    private lateinit var suggestAdapter: ArrayAdapter<String>
    private var programmaticUrlSet = false

    private val tabs = mutableListOf<BrowserTab>()
    private var activeTabIndex = 0
    private var nextTabId = 1L
    private var adblockEnabled = true
    private var bgPlayEnabled = true
    private var darkMode = true
    private var desktopMode = false
    private var searchEngine = "Google"
    private var totalBlocked = 0

    private val historyItems = mutableListOf<Pair<String, String>>()
    private val downloadItems = mutableListOf<String>()
    private var ytAdSkipperScript: String = ""

    private val pauseMediaReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            if (intent?.action == MediaPlaybackService.ACTION_PAUSE_WEB) {
                tabs.forEach { t ->
                    t.webView.evaluateJavascript(
                        "document.querySelectorAll('video,audio').forEach(m=>{try{m.pause();}catch(_){}});",
                        null
                    )
                }
            }
        }
    }

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).toInt()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        WindowCompat.setDecorFitsSystemWindows(window, false)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            window.attributes = window.attributes.apply {
                layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
            }
        }
        setContentView(R.layout.activity_main)

        ytAdSkipperScript = try {
            assets.open("yt_ad_skipper.js").bufferedReader().use { it.readText() }
        } catch (_: Exception) {
            ""
        }
        vault = PasswordVault(this)
        loadLocalLists()
        rootFrame = findViewById(R.id.rootFrame)
        mainColumn = findViewById(R.id.mainColumn)
        urlInput = findViewById(R.id.urlInput)
        topBar = findViewById(R.id.topBar)
        bottomDivider = findViewById(R.id.bottomDivider)
        bottomBar = findViewById(R.id.bottomBar)
        omniboxPill = findViewById(R.id.omniboxPill)
        btnHome = findViewById(R.id.btnHome)
        btnReload = findViewById(R.id.btnReload)
        btnNewTab = findViewById(R.id.btnNewTab)
        btnMenu = findViewById(R.id.btnMenu)
        btnPip = findViewById(R.id.btnPip)
        btnTranslate = findViewById(R.id.btnTranslate)
        adBadge = findViewById(R.id.adBadge)
        btnPip.setOnClickListener { enterVideoPip() }
        btnTranslate.setOnClickListener { translateCurrentPage() }

        val filter = IntentFilter(MediaPlaybackService.ACTION_PAUSE_WEB)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            registerReceiver(pauseMediaReceiver, filter, Context.RECEIVER_NOT_EXPORTED)
            if (checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
                requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 101)
            }
        } else {
            registerReceiver(pauseMediaReceiver, filter)
        }
        btnTabs = findViewById(R.id.btnTabs)
        btnBack = findViewById(R.id.btnBack)
        btnForward = findViewById(R.id.btnForward)
        progressBar = findViewById(R.id.progressBar)
        webContainer = findViewById(R.id.webContainer)
        suggestList = findViewById(R.id.suggestList)
        tabSwitcherOverlay = findViewById(R.id.tabSwitcherOverlay)
        tabSwitcherHeader = findViewById(R.id.tabSwitcherHeader)
        tabSwitcherTitle = findViewById(R.id.tabSwitcherTitle)
        tabGrid = findViewById(R.id.tabGrid)

        ViewCompat.setOnApplyWindowInsetsListener(rootFrame) { _, windowInsets ->
            if (isInPictureInPictureMode) {
                topBar.visibility = View.GONE
                bottomDivider.visibility = View.GONE
                bottomBar.visibility = View.GONE
                mainColumn.setPadding(0, 0, 0, 0)
                return@setOnApplyWindowInsetsListener windowInsets
            }
            topBar.visibility = View.VISIBLE
            val bars = windowInsets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            val ime = windowInsets.getInsets(WindowInsetsCompat.Type.ime())
            val resStatusId = resources.getIdentifier("status_bar_height", "dimen", "android")
            val fallbackTop = if (resStatusId > 0) resources.getDimensionPixelSize(resStatusId) else dp(28)
            val topInset = maxOf(bars.top, fallbackTop)
            val leftInset = bars.left
            val rightInset = bars.right
            val navBottom = bars.bottom
            val imeOpen = ime.bottom > navBottom + dp(40)

            topBar.layoutParams = topBar.layoutParams.apply {
                height = dp(56) + topInset
            }
            topBar.setPadding(dp(10) + leftInset, topInset, dp(10) + rightInset, 0)

            if (imeOpen) {
                bottomDivider.visibility = View.GONE
                bottomBar.visibility = View.GONE
                mainColumn.setPadding(0, 0, 0, ime.bottom)
            } else {
                mainColumn.setPadding(0, 0, 0, 0)
                bottomDivider.visibility = View.VISIBLE
                bottomBar.visibility = View.VISIBLE
                bottomBar.layoutParams = bottomBar.layoutParams.apply {
                    height = dp(54) + navBottom
                }
                bottomBar.setPadding(leftInset, 0, rightInset, navBottom)
            }

            tabSwitcherHeader.layoutParams = tabSwitcherHeader.layoutParams.apply {
                height = dp(58) + topInset
            }
            tabSwitcherHeader.setPadding(dp(14) + leftInset, topInset, dp(10) + rightInset, 0)
            tabGrid.setPadding(dp(14) + leftInset, dp(14), dp(14) + rightInset, dp(14) + navBottom)

            windowInsets
        }
        ViewCompat.requestApplyInsets(rootFrame)
        applyNativeChromeTheme()

        tabGridAdapter = object : BaseAdapter() {
            override fun getCount(): Int = tabs.size
            override fun getItem(position: Int): Any = tabs[position]
            override fun getItemId(position: Int): Long = tabs[position].id
            override fun getView(position: Int, convertView: View?, parent: ViewGroup): View {
                val view = convertView ?: layoutInflater.inflate(R.layout.tab_card, parent, false)
                val tab = tabs[position]
                val isActive = position == activeTabIndex
                val root = view.findViewById<View>(R.id.tabCardRoot)
                val ico = view.findViewById<TextView>(R.id.tabCardIcon)
                val title = view.findViewById<TextView>(R.id.tabCardTitle)
                val badge = view.findViewById<TextView>(R.id.tabCardBadge)
                val domain = view.findViewById<TextView>(R.id.tabCardDomain)
                val close = view.findViewById<TextView>(R.id.tabCardClose)

                root.setBackgroundResource(
                    when {
                        isActive && tab.incognito -> R.drawable.bg_tab_card_incog
                        isActive -> R.drawable.bg_tab_card_active
                        else -> R.drawable.bg_tab_card_normal
                    }
                )
                val cleanTitle = tab.title.ifBlank { if (tab.incognito) "Tab ẩn danh" else "Tab mới" }
                ico.text = if (tab.incognito) "🕵" else cleanTitle.trim().firstOrNull()?.uppercaseChar()?.toString() ?: "V"
                title.text = cleanTitle
                badge.visibility = if (isActive) View.VISIBLE else View.GONE
                domain.text = if (tab.url == NEWTAB_URL) "Trang chủ VN Surf" else AdBlockEngine.extractHost(tab.url).ifBlank { tab.url }

                view.setOnClickListener {
                    switchToTab(position)
                    tabSwitcherOverlay.visibility = View.GONE
                }
                close.setOnClickListener {
                    closeTab(position)
                    refreshTabSwitcher()
                }
                return view
            }
        }
        tabGrid.adapter = tabGridAdapter

        findViewById<Button>(R.id.btnTabNewNormal).setOnClickListener {
            tabSwitcherOverlay.visibility = View.GONE
            openNewTab(NEWTAB_URL, false)
        }
        findViewById<Button>(R.id.btnTabNewIncog).setOnClickListener {
            tabSwitcherOverlay.visibility = View.GONE
            openNewTab(NEWTAB_URL, true)
        }
        findViewById<TextView>(R.id.btnTabSwitcherClose).setOnClickListener {
            tabSwitcherOverlay.visibility = View.GONE
        }

        suggestAdapter = object : ArrayAdapter<String>(this, android.R.layout.simple_list_item_1, suggestions) {
            override fun getView(position: Int, convertView: View?, parent: ViewGroup): View {
                val tv = super.getView(position, convertView, parent) as TextView
                tv.setTextColor(0xFFF8FAFC.toInt())
                tv.textSize = 14f
                tv.setPadding(36, 26, 36, 26)
                return tv
            }
        }
        suggestList.adapter = suggestAdapter
        suggestList.setOnItemClickListener { _, _, position, _ ->
            val q = suggestions.getOrNull(position) ?: return@setOnItemClickListener
            suggestList.visibility = View.GONE
            navigateCurrent(q)
            hideKeyboard()
        }

        urlInput.setOnFocusChangeListener { _, hasFocus ->
            if (hasFocus) {
                urlInput.post { urlInput.selectAll() }
            } else {
                suggestList.visibility = View.GONE
            }
        }

        urlInput.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}
            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}
            override fun afterTextChanged(s: Editable?) {
                if (programmaticUrlSet || !urlInput.hasFocus()) return
                val q = s?.toString()?.trim() ?: ""
                suggestRunnable?.let { mainHandler.removeCallbacks(it) }
                if (q.isEmpty() || q.startsWith("http://") || q.startsWith("https://")) {
                    suggestions.clear()
                    suggestAdapter.notifyDataSetChanged()
                    suggestList.visibility = View.GONE
                    return
                }
                val r = Runnable { fetchSuggestions(q) }
                suggestRunnable = r
                mainHandler.postDelayed(r, 180)
            }
        })

        findViewById<ImageButton>(R.id.btnHome).setOnClickListener {
            loadNewTabHome(currentTab())
        }
        findViewById<ImageButton>(R.id.btnReload).setOnClickListener {
            val tab = currentTab() ?: return@setOnClickListener
            if (tab.url == NEWTAB_URL) loadNewTabHome(tab) else tab.webView.reload()
        }
        btnBack.setOnClickListener {
            val wv = currentTab()?.webView
            if (wv != null && wv.canGoBack()) wv.goBack()
        }
        btnForward.setOnClickListener {
            val wv = currentTab()?.webView
            if (wv != null && wv.canGoForward()) wv.goForward()
        }
        findViewById<ImageButton>(R.id.btnNewTab).setOnClickListener {
            openNewTab(NEWTAB_URL, incognito = false)
        }
        findViewById<View>(R.id.btnTabsContainer).setOnClickListener { showTabsDialog() }
        findViewById<ImageButton>(R.id.btnMenu).setOnClickListener { showMainMenu() }
        adBadge.setOnClickListener {
            Toast.makeText(
                this,
                if (adblockEnabled) "🛡️ Đã chặn tổng cộng $totalBlocked quảng cáo & popup" else "🛡️ Chặn quảng cáo đang tắt",
                Toast.LENGTH_SHORT
            ).show()
        }

        urlInput.setOnEditorActionListener { _, actionId, event ->
            if (actionId == EditorInfo.IME_ACTION_GO ||
                (event != null && event.keyCode == KeyEvent.KEYCODE_ENTER && event.action == KeyEvent.ACTION_DOWN)
            ) {
                navigateCurrent(urlInput.text.toString())
                hideKeyboard()
                true
            } else false
        }

        val initialUrl = intent?.dataString ?: NEWTAB_URL
        openNewTab(initialUrl, incognito = false)
    }

    private fun currentTab(): BrowserTab? = tabs.getOrNull(activeTabIndex)

    private fun hideKeyboard() {
        val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
        imm.hideSoftInputFromWindow(urlInput.windowToken, 0)
        urlInput.clearFocus()
        suggestList.visibility = View.GONE
    }

    private fun fetchSuggestions(query: String) {
        Thread {
            try {
                val url = URL("https://suggestqueries.google.com/complete/search?client=firefox&q=" + URLEncoder.encode(query, "UTF-8"))
                val conn = url.openConnection() as HttpURLConnection
                conn.connectTimeout = 2500
                conn.readTimeout = 2500
                conn.setRequestProperty("User-Agent", "VNBrowser/1.1")
                val body = conn.inputStream.bufferedReader().use { it.readText() }
                conn.disconnect()
                val arr = JSONArray(body).optJSONArray(1) ?: JSONArray()
                val res = mutableListOf<String>()
                for (i in 0 until minOf(arr.length(), 6)) {
                    val item = arr.optString(i)
                    if (item.isNotBlank()) res.add("🔍  $item")
                }
                mainHandler.post {
                    if (urlInput.hasFocus() && urlInput.text.toString().trim() == query) {
                        suggestions.clear()
                        suggestions.addAll(res.map { it.removePrefix("🔍  ") })
                        suggestAdapter.notifyDataSetChanged()
                        suggestList.visibility = if (suggestions.isNotEmpty()) View.VISIBLE else View.GONE
                    }
                }
            } catch (_: Exception) {
            }
        }.start()
    }

    private fun resolveAddress(raw: String): String {
        val t = raw.trim()
        if (t.isEmpty() || t == NEWTAB_URL || t == "vn://newtab") return NEWTAB_URL
        if (t.startsWith("http://") || t.startsWith("https://")) return t
        if (!t.contains(" ") && t.contains(".")) return "https://$t"
        val enc = URLEncoder.encode(t, "UTF-8")
        return when (searchEngine) {
            "DuckDuckGo" -> "https://duckduckgo.com/?q=$enc"
            "Bing" -> "https://www.bing.com/search?q=$enc"
            else -> "https://www.google.com/search?q=$enc"
        }
    }

    private fun navigateCurrent(raw: String) {
        val target = resolveAddress(raw)
        val tab = currentTab() ?: return
        if (target == NEWTAB_URL) {
            loadNewTabHome(tab)
        } else {
            tab.url = target
            tab.webView.loadUrl(target)
        }
    }

    private fun loadNewTabHome(tab: BrowserTab?) {
        tab ?: return
        tab.url = NEWTAB_URL
        tab.title = if (tab.incognito) "Tab ẩn danh" else "Tab mới"
        tab.webView.loadDataWithBaseURL(
            NEWTAB_URL,
            buildNewTabHtml(tab.incognito),
            "text/html",
            "UTF-8",
            null
        )
        updateChromeUi()
    }

    private fun buildNewTabHtml(incognito: Boolean): String {
        val bg = when {
            incognito -> "#181524"
            darkMode -> "#202124"
            else -> "#f1f3f4"
        }
        val fg = if (darkMode || incognito) "#f8fafc" else "#202124"
        val subFg = if (darkMode || incognito) "#cbd5e1" else "#5f6368"
        val card = when {
            incognito -> "#241f35"
            darkMode -> "#292a2d"
            else -> "#ffffff"
        }
        val pill = when {
            incognito -> "#2f2847"
            darkMode -> "#35363a"
            else -> "#ffffff"
        }
        val scFg = if (darkMode || incognito) "#f1f5f9" else "#202124"
        val badge = if (incognito) {
            "<div style='display:inline-block;padding:5px 14px;border-radius:999px;background:linear-gradient(135deg,#7c3aed,#ec4899);color:#fff;font-size:12px;font-weight:700;margin-bottom:10px'>🕵️ Chế độ Ẩn danh</div>"
        } else ""
        return """
            <!doctype html>
            <html>
            <head>
            <meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no">
            <style>
              *{box-sizing:border-box;margin:0;padding:0;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif}
              body{background:$bg;color:$fg;min-height:100vh;padding:30px 18px 28px;background-image:radial-gradient(circle at 15% 5%,rgba(16,185,129,.18),transparent 42%),radial-gradient(circle at 85% 12%,rgba(56,189,248,.16),transparent 44%)}
              .hero{text-align:center;margin-bottom:22px}
              .logo{display:inline-block;padding:0 4px;font-size:36px;font-weight:900;letter-spacing:-.5px;background:linear-gradient(135deg,#10b981,#0ea5e9,#8b5cf6,#ec4899);-webkit-background-clip:text;-webkit-text-fill-color:transparent;margin-bottom:6px}
              .sub{font-size:12.5px;color:$subFg}
              .search{display:flex;align-items:center;gap:10px;background:$pill;border:1.5px solid rgba(56,189,248,.45);border-radius:999px;padding:10px 16px;box-shadow:0 8px 22px rgba(0,0,0,.12);margin-bottom:14px}
              .search input{flex:1;background:transparent;border:none;outline:none;color:$fg;font-size:15px}
              .search input::placeholder{color:$subFg}
              .chips{display:flex;gap:10px;justify-content:center;margin-bottom:20px}
              .chip{flex:1;max-width:170px;display:flex;align-items:center;justify-content:center;gap:6px;background:$card;border:1px solid rgba(56,189,248,.35);color:$fg;padding:10px 14px;border-radius:999px;font-size:13px;font-weight:700}
              .card{background:$card;border:1px solid rgba(148,163,184,.2);border-radius:22px;padding:16px 12px;margin-bottom:14px;box-shadow:0 8px 20px rgba(0,0,0,.10)}
              .grid{display:grid;grid-template-columns:repeat(4,1fr);gap:14px 8px}
              .sc{display:flex;flex-direction:column;align-items:center;gap:7px;text-decoration:none;color:$scFg;font-size:11.5px;font-weight:600;text-align:center}
              .ico{width:48px;height:48px;border-radius:16px;display:flex;align-items:center;justify-content:center;font-size:20px;font-weight:800;color:#fff;box-shadow:0 6px 14px rgba(0,0,0,.28)}
              .stats{display:flex;align-items:center;justify-content:space-between;background:linear-gradient(135deg,rgba(16,185,129,.16),rgba(14,165,233,.14));border:1px solid rgba(16,185,129,.35)}
              .stats b{color:#34d399;font-size:20px}
            </style>
            </head>
            <body>
              <div class="hero">
                $badge
                <div class="logo">VN Surf</div>
                <div class="sub">Nhanh · Màu sắc · Chặn quảng cáo & Bảo mật</div>
              </div>
              <form class="search" onsubmit="event.preventDefault();const v=document.getElementById('q').value;if(v.trim())VNBridge.navigate(v);">
                <span>🔍</span>
                <input id="q" type="text" placeholder="Tìm trên Google hoặc nhập URL" autocomplete="off" oninput="clearTimeout(window.__st);const val=this.value.trim();if(!val){document.getElementById('ntSug').style.display='none';return;}window.__st=setTimeout(()=>VNBridge.requestNewTabSuggest(val),160);" />
              </form>
              <div id="ntSug" style="display:none;background:$card;border:1px solid rgba(148,163,184,.25);border-radius:18px;padding:6px;margin:-6px 0 14px;box-shadow:0 10px 24px rgba(0,0,0,.2)"></div>
              <script>
                window.__vnNtSuggest = function(arr) {
                  const box = document.getElementById('ntSug');
                  if (!arr || !arr.length) { box.style.display = 'none'; return; }
                  box.innerHTML = arr.map(s => '<div style="padding:10px 12px;border-radius:12px;font-size:14px;color:$fg" onclick="VNBridge.navigate(' + JSON.stringify(s) + ')">🔍 &nbsp;' + s.replace(/</g,'&lt;') + '</div>').join('');
                  box.style.display = 'block';
                };
              </script>
              <div class="chips">
                <button class="chip" onclick="VNBridge.openIncognito()">🕵️ Ẩn danh</button>
                <button class="chip" onclick="VNBridge.openPasswords()">🔑 Mật khẩu</button>
              </div>
              <div class="card">
                <div class="grid">
                  <div class="sc" onclick="VNBridge.navigate('https://www.google.com')"><div class="ico" style="background:linear-gradient(135deg,#3b82f6,#06b6d4)">G</div><span>Google</span></div>
                  <div class="sc" onclick="VNBridge.navigate('https://m.youtube.com')"><div class="ico" style="background:linear-gradient(135deg,#ef4444,#f97316)">▶</div><span>YouTube</span></div>
                  <div class="sc" onclick="VNBridge.navigate('https://github.com')"><div class="ico" style="background:linear-gradient(135deg,#6366f1,#a855f7)">⌘</div><span>GitHub</span></div>
                  <div class="sc" onclick="VNBridge.navigate('https://m.facebook.com')"><div class="ico" style="background:linear-gradient(135deg,#2563eb,#38bdf8)">f</div><span>Facebook</span></div>
                  <div class="sc" onclick="VNBridge.navigate('https://www.tiktok.com')"><div class="ico" style="background:linear-gradient(135deg,#06b6d4,#ec4899)">♪</div><span>TikTok</span></div>
                  <div class="sc" onclick="VNBridge.navigate('https://chatgpt.com')"><div class="ico" style="background:linear-gradient(135deg,#10b981,#14b8a6)">✦</div><span>ChatGPT</span></div>
                  <div class="sc" onclick="VNBridge.navigate('https://vnexpress.net')"><div class="ico" style="background:linear-gradient(135deg,#f43f5e,#fb7185)">V</div><span>VnExpress</span></div>
                  <div class="sc" onclick="VNBridge.navigate('https://translate.google.com')"><div class="ico" style="background:linear-gradient(135deg,#8b5cf6,#3b82f6)">文</div><span>Dịch</span></div>
                </div>
              </div>
              <div class="card stats">
                <div>
                  <div style="font-size:13.5px;font-weight:700">🛡️ Khiên bảo vệ VN Surf</div>
                  <div style="font-size:12px;color:#cbd5e1;margin-top:2px">Chặn quảng cáo & tab nhảy tự động</div>
                </div>
                <b>$totalBlocked</b>
              </div>
            </body>
            </html>
        """.trimIndent()
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun openNewTab(url: String, incognito: Boolean) {
        val tabId = nextTabId++
        val wv = BgPlayWebView(this).apply {
            allowBgAudio = bgPlayEnabled
        }
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
        if (WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT)) {
            try {
                WebViewCompat.addDocumentStartJavaScript(wv, ANDROID_BG_MEDIA_SCRIPT, setOf("*"))
                if (ytAdSkipperScript.isNotEmpty()) {
                    WebViewCompat.addDocumentStartJavaScript(
                        wv,
                        ytAdSkipperScript,
                        setOf("https://*.youtube.com", "https://youtube.com", "http://*.youtube.com")
                    )
                }
            } catch (_: Exception) {}
        }

        val tab = BrowserTab(tabId, if (incognito) "Tab ẩn danh" else "Tab mới", url, incognito, 0, false, wv)

        wv.addJavascriptInterface(object {
            @JavascriptInterface
            fun navigate(raw: String) {
                runOnUiThread { navigateCurrent(raw) }
            }

            @JavascriptInterface
            fun requestNewTabSuggest(query: String) {
                val q = query.trim()
                if (q.isEmpty()) return
                Thread {
                    try {
                        val url = URL("https://suggestqueries.google.com/complete/search?client=firefox&q=" + URLEncoder.encode(q, "UTF-8"))
                        val conn = url.openConnection() as HttpURLConnection
                        conn.connectTimeout = 2500
                        conn.readTimeout = 2500
                        conn.setRequestProperty("User-Agent", "VNSurf/1.1")
                        val body = conn.inputStream.bufferedReader().use { it.readText() }
                        conn.disconnect()
                        val arr = JSONArray(body).optJSONArray(1) ?: JSONArray()
                        val out = JSONArray()
                        for (i in 0 until minOf(arr.length(), 6)) {
                            val item = arr.optString(i)
                            if (item.isNotBlank()) out.put(item)
                        }
                        mainHandler.post {
                            currentTab()?.webView?.evaluateJavascript("window.__vnNtSuggest && window.__vnNtSuggest($out);", null)
                        }
                    } catch (_: Exception) {}
                }.start()
            }

            @JavascriptInterface
            fun openIncognito() {
                runOnUiThread { openNewTab(NEWTAB_URL, incognito = true) }
            }

            @JavascriptInterface
            fun openPasswords() {
                runOnUiThread {
                    authenticateDevice("Xác minh bằng khoá máy để xem Mật khẩu đã lưu") {
                        openInternalTab(PASSWORDS_URL, "🔑 Mật khẩu")
                    }
                }
            }

            @JavascriptInterface
            fun deleteCred(id: String) {
                runOnUiThread {
                    vault.delete(id)
                    currentTab()?.let { if (it.url == PASSWORDS_URL) renderInternalTab(it) }
                }
            }

            @JavascriptInterface
            fun copyCred(pw: String) {
                runOnUiThread {
                    val cm = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                    cm.setPrimaryClip(ClipData.newPlainText("password", pw))
                    Toast.makeText(this@MainActivity, "✓ Đã sao chép mật khẩu", Toast.LENGTH_SHORT).show()
                }
            }

            @JavascriptInterface
            fun clearHistory() {
                runOnUiThread {
                    historyItems.clear()
                    saveLocalLists()
                    currentTab()?.let { if (it.url == HISTORY_URL) renderInternalTab(it) }
                }
            }

            @JavascriptInterface
            fun onAdSkipped() {
                runOnUiThread {
                    tab.blockedCount++
                    totalBlocked++
                    updateChromeUi()
                }
            }

            @JavascriptInterface
            fun onVideoDetected(hasVideo: Boolean) {
                runOnUiThread {
                    tab.hasVideo = hasVideo
                    updateChromeUi()
                }
            }

            @JavascriptInterface
            fun enterPip() {
                runOnUiThread { enterVideoPip() }
            }

            @JavascriptInterface
            fun toggleSetting(key: String) {
                runOnUiThread {
                    when (key) {
                        "adblock" -> {
                            adblockEnabled = !adblockEnabled
                            syncYouTubeAdblockAttr()
                            updateChromeUi()
                        }
                        "dark" -> {
                            darkMode = !darkMode
                            tabs.forEach { applyDarkModeToTab(it) }
                        }
                        "desktop" -> toggleDesktopMode()
                    }
                    currentTab()?.let { if (it.url == SETTINGS_URL) renderInternalTab(it) }
                }
            }

            @JavascriptInterface
            fun setSearchEngine(eng: String) {
                runOnUiThread {
                    searchEngine = eng
                    saveLocalLists()
                    currentTab()?.let { if (it.url == SETTINGS_URL) renderInternalTab(it) }
                    Toast.makeText(this@MainActivity, "🔍 Đã chọn tìm kiếm: $eng", Toast.LENGTH_SHORT).show()
                }
            }

            @JavascriptInterface
            fun clearCacheData() {
                runOnUiThread {
                    CookieManager.getInstance().removeAllCookies(null)
                    tabs.forEach { it.webView.clearCache(true) }
                    historyItems.clear()
                    saveLocalLists()
                    Toast.makeText(this@MainActivity, "✓ Đã xoá lịch sử, cookie & cache", Toast.LENGTH_SHORT).show()
                    currentTab()?.let { if (it.url == SETTINGS_URL) renderInternalTab(it) }
                }
            }

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
                if (!pageUrl.isNullOrBlank() && pageUrl != "about:blank") tab.url = pageUrl
                tab.hasVideo = tab.url.contains("youtube.com") || tab.url.contains("youtu.be")
                progressBar.visibility = View.VISIBLE
                updateChromeUi()
            }

            override fun onPageFinished(view: WebView?, pageUrl: String?) {
                progressBar.visibility = View.GONE
                if (!pageUrl.isNullOrBlank() && pageUrl != "about:blank") tab.url = pageUrl
                if (!tab.url.startsWith("https://vn.")) {
                    view?.evaluateJavascript(
                        "window.__vnAllowBgPlay=${bgPlayEnabled};",
                        null
                    )
                    view?.evaluateJavascript(ANDROID_BG_MEDIA_SCRIPT, null)
                    if (!tab.incognito && tab.url.startsWith("http")) {
                        val t = tab.title.ifBlank { tab.url }
                        historyItems.removeAll { it.second == tab.url }
                        historyItems.add(0, t to tab.url)
                        if (historyItems.size > 100) historyItems.removeAt(historyItems.lastIndex)
                        saveLocalLists()
                    }
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
                    applyDarkModeToTab(tab)
                    if (tab.url.contains("youtube.com")) {
                        val off = (!adblockEnabled).toString()
                        view?.evaluateJavascript(
                            "if(document.documentElement){document.documentElement.setAttribute('data-yt-adblock-disabled','$off');document.documentElement.setAttribute('data-yt-banners-disabled','$off');}",
                            null
                        )
                        if (ytAdSkipperScript.isNotEmpty()) {
                            view?.evaluateJavascript(ytAdSkipperScript, null)
                        }
                    }
                }
                updateChromeUi()
            }
        }

        wv.webChromeClient = object : WebChromeClient() {
            override fun onProgressChanged(view: WebView?, newProgress: Int) {
                progressBar.progress = newProgress
                if (newProgress >= 100) progressBar.visibility = View.GONE
            }

            override fun onReceivedTitle(view: WebView?, title: String?) {
                if (!title.isNullOrBlank() && title != "about:blank" && !title.startsWith("https://vn.newtab")) {
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
            downloadItems.add(0, "$fileName ($dlUrl)")
            saveLocalLists()
            Toast.makeText(this, "⬇ Đang tải $fileName", Toast.LENGTH_SHORT).show()
        }

        tabs.add(tab)
        switchToTab(tabs.lastIndex)
        if (resolveAddress(url) == NEWTAB_URL) {
            loadNewTabHome(tab)
        } else {
            wv.loadUrl(resolveAddress(url))
        }
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
            loadNewTabHome(currentTab())
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
            programmaticUrlSet = true
            urlInput.setText(if (tab.url == NEWTAB_URL) "" else tab.url)
            programmaticUrlSet = false
        }
        val canTranslate = tab.url.startsWith("http") && !tab.url.startsWith("https://vn.") && !tab.url.contains("translate.google.")
        btnTranslate.visibility = if (canTranslate) View.VISIBLE else View.GONE
        val showPip = tab.hasVideo || tab.url.contains("youtube.com/watch") || tab.url.contains("youtu.be/") || tab.url.contains("youtube.com")
        btnPip.visibility = if (showPip && !tab.url.startsWith("https://vn.")) View.VISIBLE else View.GONE
        adBadge.text = if (adblockEnabled) "🛡️ ${tab.blockedCount}" else "🛡️ Tắt"
        btnTabs.text = tabs.size.toString()
        btnBack.alpha = if (tab.webView.canGoBack()) 1.0f else 0.35f
        btnForward.alpha = if (tab.webView.canGoForward()) 1.0f else 0.35f
    }

    private fun translateCurrentPage() {
        val tab = currentTab() ?: return
        if (!tab.url.startsWith("http") || tab.url.startsWith("https://vn.")) {
            Toast.makeText(this, "Hãy mở một trang web để dịch", Toast.LENGTH_SHORT).show()
            return
        }
        val names = arrayOf("🇻🇳 Tiếng Việt", "🇺🇸 English", "🇯🇵 Tiếng Nhật", "🇰🇷 Tiếng Hàn", "🇨🇳 Tiếng Trung", "🇫🇷 Tiếng Pháp", "🇩🇪 Tiếng Đức", "🇪🇸 Tiếng Tây Ban Nha", "🇹🇭 Tiếng Thái", "🇷🇺 Tiếng Nga")
        val codes = arrayOf("vi", "en", "ja", "ko", "zh-CN", "fr", "de", "es", "th", "ru")
        AlertDialog.Builder(this)
            .setTitle("🌐 Chọn ngôn ngữ dịch")
            .setItems(names) { _, which ->
                val code = codes[which]
                val tr = "https://translate.google.com/translate?sl=auto&tl=$code&u=" + URLEncoder.encode(tab.url, "UTF-8")
                tab.url = tr
                tab.webView.loadUrl(tr)
                Toast.makeText(this, "🌐 Đang dịch sang ${names[which]}...", Toast.LENGTH_SHORT).show()
            }
            .setNegativeButton("Huỷ", null)
            .show()
    }

    private fun applyNativeChromeTheme() {
        val barBg = if (darkMode) 0xFF202124.toInt() else 0xFFFFFFFF.toInt()
        val botBg = if (darkMode) 0xFF292A2D.toInt() else 0xFFF1F3F4.toInt()
        val divCol = if (darkMode) 0xFF3C4043.toInt() else 0xFFDADCE0.toInt()
        val txtFg = if (darkMode) 0xFFF8FAFC.toInt() else 0xFF202124.toInt()
        val hintFg = if (darkMode) 0xFF9AA0A6.toInt() else 0xFF5F6368.toInt()
        val icoTint = if (darkMode) 0xFFE2E8F0.toInt() else 0xFF202124.toInt()
        rootFrame.setBackgroundColor(barBg)
        mainColumn.setBackgroundColor(barBg)
        topBar.setBackgroundColor(barBg)
        bottomDivider.setBackgroundColor(divCol)
        bottomBar.setBackgroundColor(botBg)
        omniboxPill.setBackgroundResource(if (darkMode) R.drawable.bg_omnibox else R.drawable.bg_omnibox_light)
        btnTabs.setBackgroundResource(if (darkMode) R.drawable.bg_tab_count else R.drawable.bg_tab_count_light)
        btnNewTab.setBackgroundResource(if (darkMode) R.drawable.bg_bottom_plus else R.drawable.bg_bottom_plus_light)
        urlInput.setTextColor(txtFg)
        urlInput.setHintTextColor(hintFg)
        btnTabs.setTextColor(txtFg)
        listOf(btnHome, btnReload, btnBack, btnForward, btnNewTab, btnMenu).forEach {
            it.setColorFilter(icoTint)
        }
        WindowInsetsControllerCompat(window, window.decorView).apply {
            isAppearanceLightStatusBars = !darkMode
            isAppearanceLightNavigationBars = !darkMode
        }
    }

    private fun applyDarkModeToTab(tab: BrowserTab) {
        applyNativeChromeTheme()
        if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
            WebSettingsCompat.setAlgorithmicDarkeningAllowed(tab.webView.settings, darkMode)
        }
        if (tab.url == NEWTAB_URL) {
            loadNewTabHome(tab)
            return
        }
        if (tab.url.startsWith("https://vn.")) {
            renderInternalTab(tab)
            return
        }
        val js = if (darkMode) {
            """
            (() => {
              let st = document.getElementById('__vnDarkCss');
              if (!st) {
                st = document.createElement('style');
                st.id = '__vnDarkCss';
                st.textContent = 'html { filter: invert(0.90) hue-rotate(180deg) !important; background-color: #e6e6e6 !important; } img, video, picture, canvas, svg, iframe { filter: invert(1) hue-rotate(180deg) !important; }';
                (document.head || document.documentElement).appendChild(st);
              }
            })();
            """.trimIndent()
        } else {
            "document.getElementById('__vnDarkCss')?.remove();"
        }
        tab.webView.evaluateJavascript(js, null)
    }

    private fun toggleDesktopMode() {
        desktopMode = !desktopMode
        tabs.forEach { t ->
            t.webView.settings.userAgentString = if (desktopMode) {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
            } else null
            if (!t.url.startsWith("https://vn.")) t.webView.reload()
        }
        Toast.makeText(this, if (desktopMode) "🖥️ Đã bật chế độ Máy tính" else "📱 Đã chuyển về chế độ Điện thoại", Toast.LENGTH_SHORT).show()
    }

    private fun openInternalTab(internalUrl: String, title: String) {
        val existingIdx = tabs.indexOfFirst { it.url == internalUrl }
        if (existingIdx >= 0) {
            switchToTab(existingIdx)
            renderInternalTab(tabs[existingIdx])
            return
        }
        openNewTab(NEWTAB_URL, false)
        val tab = currentTab() ?: return
        tab.url = internalUrl
        tab.title = title
        renderInternalTab(tab)
    }

    private fun renderInternalTab(tab: BrowserTab) {
        val html = buildInternalPageHtml(tab.url)
        tab.webView.loadDataWithBaseURL(tab.url, html, "text/html", "UTF-8", null)
        updateChromeUi()
    }

    private fun escHtml(s: String): String =
        s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\"", "&quot;")

    private fun buildInternalPageHtml(url: String): String {
        val pBg = if (darkMode) "#202124" else "#f1f3f4"
        val pCard = if (darkMode) "#292a2d" else "#ffffff"
        val pFg = if (darkMode) "#f8fafc" else "#202124"
        val pSub = if (darkMode) "#9aa0a6" else "#5f6368"
        val head = """
            <!doctype html><html><head>
            <meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1">
            <style>
              *{box-sizing:border-box;margin:0;padding:0;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif}
              body{background:$pBg;color:$pFg;min-height:100vh;padding:20px 16px}
              h2{font-size:21px;margin-bottom:14px}
              .card{background:$pCard;border:1px solid rgba(148,163,184,.2);border-radius:18px;padding:14px;margin-bottom:10px}
              .row{display:flex;align-items:center;justify-content:space-between;gap:10px}
              .btn{border:none;border-radius:12px;padding:8px 14px;background:linear-gradient(135deg,#10b981,#0ea5e9);color:#fff;font-weight:700;font-size:12.5px}
              .btn.danger{background:rgba(239,68,68,.18);color:#ef4444}
              .sub{font-size:12px;color:$pSub;word-break:break-all;margin-top:3px}
              .sec{font-size:12px;font-weight:800;text-transform:uppercase;color:#10b981;margin:14px 4px 8px;letter-spacing:.5px}
            </style></head><body>
        """.trimIndent()
        val body = when (url) {
            HISTORY_URL -> {
                val rows = if (historyItems.isEmpty()) "<div class='card'>Chưa có lịch sử nào.</div>"
                else historyItems.joinToString("") { (t, u) ->
                    "<div class='card' onclick='VNBridge.navigate(${JSONObject.quote(u)})'><b>${escHtml(t)}</b><div class='sub'>${escHtml(u)}</div></div>"
                }
                "<div class='row' style='margin-bottom:14px'><h2>🕒 Lịch sử</h2><button class='btn danger' onclick='VNBridge.clearHistory()'>Xoá hết</button></div>$rows"
            }
            DOWNLOADS_URL -> {
                val rows = if (downloadItems.isEmpty()) "<div class='card'>Chưa có tệp tải xuống nào trong phiên này (tệp tải lưu ở thư mục Downloads của máy).</div>"
                else downloadItems.joinToString("") { "<div class='card'><b>⬇ ${escHtml(it)}</b></div>" }
                "<h2>⬇ Tệp đã tải xuống</h2>$rows"
            }
            PASSWORDS_URL -> {
                val list = vault.list()
                val rows = if (list.isEmpty()) "<div class='card'>Chưa có mật khẩu nào được lưu.</div>"
                else list.joinToString("") { c ->
                    "<div class='card row'><div style='flex:1;overflow:hidden'><b>${escHtml(c.site)}</b><div class='sub'>${escHtml(c.username)}</div></div><button class='btn' onclick='VNBridge.copyCred(${JSONObject.quote(c.password)})'>Chép MK</button><button class='btn danger' onclick='VNBridge.deleteCred(${JSONObject.quote(c.id)})'>Xoá</button></div>"
                }
                "<h2>🔑 Mật khẩu đã lưu</h2>$rows"
            }
            SETTINGS_URL -> """
                <h2>⚙️ Cài đặt VN Surf</h2>
                <div class='sec'>🔑 Tự động điền & Mật khẩu</div>
                <div class='card row'><div><b>Trình quản lý mật khẩu</b><div class='sub'>Mã hoá Keystore, khoá bằng vân tay/PIN máy</div></div><button class='btn' onclick='VNBridge.openPasswords()'>Mở két sắt</button></div>
                <div class='sec'>🎨 Giao diện & Tìm kiếm</div>
                <div class='card row'><div><b>Chế độ nền (Tối / Sáng)</b><div class='sub'>Đổi nền app & tự làm tối các trang web</div></div><button class='btn' onclick="VNBridge.toggleSetting('dark')">${if (darkMode) "🌙 Đang bật Tối" else "☀️ Đang bật Sáng"}</button></div>
                <div class='card row'>
                  <div><b>Công cụ tìm kiếm</b><div class='sub'>Mặc định trên thanh địa chỉ</div></div>
                  <select onchange='VNBridge.setSearchEngine(this.value)' style='background:${if (darkMode) "#35363a" else "#e8eaed"};color:$pFg;border:1px solid rgba(148,163,184,.3);border-radius:10px;padding:7px 10px'>
                    <option ${if (searchEngine=="Google") "selected" else ""}>Google</option>
                    <option ${if (searchEngine=="DuckDuckGo") "selected" else ""}>DuckDuckGo</option>
                    <option ${if (searchEngine=="Bing") "selected" else ""}>Bing</option>
                  </select>
                </div>
                <div class='card row'><div><b>Chế độ Máy tính (Desktop)</b><div class='sub'>Xem giao diện web bản desktop</div></div><button class='btn' onclick="VNBridge.toggleSetting('desktop')">${if (desktopMode) "Đang bật" else "Đã tắt"}</button></div>
                <div class='sec'>🛡️ Quyền riêng tư & Bảo mật</div>
                <div class='card row'><div><b>Chặn quảng cáo & YouTube Ads</b><div class='sub'>Chặn banner, pop-up, chuyển hướng & QC YouTube</div></div><button class='btn' onclick="VNBridge.toggleSetting('adblock')">${if (adblockEnabled) "Đang bật" else "Đã tắt"}</button></div>
                <div class='card row'><div><b>Lịch sử duyệt web</b><div class='sub'>Đang lưu ${historyItems.size} trang</div></div><button class='btn danger' onclick='VNBridge.clearHistory()'>Xoá lịch sử</button></div>
                <div class='card row'><div><b>Xoá Cookie, Cache & Đặt lại</b><div class='sub'>Giải phóng bộ nhớ và đăng xuất các trang web</div></div><button class='btn danger' onclick='VNBridge.clearCacheData()'>Xoá tất cả</button></div>
                <div class='sec'>ℹ️ Giới thiệu VN Surf</div>
                <div class='card'><b>VN Surf v1.2.0 (Android)</b><div class='sub'>Tích hợp YouTube AdBlock Pro & Speed 0x–16x · Đã chặn $totalBlocked quảng cáo & tab nhảy.</div></div>
            """.trimIndent()
            else -> ""
        }
        return "$head$body</body></html>"
    }

    private fun syncYouTubeAdblockAttr() {
        val off = (!adblockEnabled).toString()
        val js = "if(document.documentElement){document.documentElement.setAttribute('data-yt-adblock-disabled','$off');document.documentElement.setAttribute('data-yt-banners-disabled','$off');}"
        tabs.forEach { t ->
            if (t.url.contains("youtube.com")) {
                t.webView.evaluateJavascript(js, null)
            }
        }
    }

    private fun loadLocalLists() {
        val sp = getSharedPreferences("vn_surf_data", Context.MODE_PRIVATE)
        searchEngine = sp.getString("search_engine", "Google") ?: "Google"
        bgPlayEnabled = sp.getBoolean("bg_play", true)
        val hArr = JSONArray(sp.getString("history", "[]") ?: "[]")
        for (i in 0 until hArr.length()) {
            val o = hArr.optJSONObject(i) ?: continue
            historyItems.add(o.optString("t") to o.optString("u"))
        }
    }

    private fun saveLocalLists() {
        val sp = getSharedPreferences("vn_surf_data", Context.MODE_PRIVATE)
        val hArr = JSONArray()
        historyItems.forEach { (t, u) -> hArr.put(JSONObject().apply { put("t", t); put("u", u) }) }
        sp.edit()
            .putString("history", hArr.toString())
            .putString("search_engine", searchEngine)
            .putBoolean("bg_play", bgPlayEnabled)
            .apply()
    }

    private fun refreshTabSwitcher() {
        tabSwitcherTitle.text = "Đang mở ${tabs.size} Tab"
        tabGridAdapter.notifyDataSetChanged()
    }

    private fun showTabsDialog() {
        refreshTabSwitcher()
        tabSwitcherOverlay.visibility = View.VISIBLE
    }

    private fun showMainMenu() {
        val sheet = BottomSheetDialog(this)
        val view = layoutInflater.inflate(R.layout.menu_bottom_sheet, null)
        sheet.setContentView(view)
        (view.parent as? View)?.setBackgroundColor(android.graphics.Color.TRANSPARENT)
        val inner = (view as? ViewGroup)?.getChildAt(0)
        if (inner != null) {
            ViewCompat.setOnApplyWindowInsetsListener(view) { _, insets ->
                val nav = insets.getInsets(WindowInsetsCompat.Type.navigationBars())
                inner.setPadding(dp(16), dp(10), dp(16), dp(20) + nav.bottom)
                insets
            }
        }

        view.findViewById<View>(R.id.bsNewTab).setOnClickListener {
            sheet.dismiss()
            openNewTab(NEWTAB_URL, false)
        }
        view.findViewById<View>(R.id.bsIncognito).setOnClickListener {
            sheet.dismiss()
            openNewTab(NEWTAB_URL, true)
        }
        view.findViewById<View>(R.id.bsTranslate).setOnClickListener {
            sheet.dismiss()
            translateCurrentPage()
        }
        view.findViewById<View>(R.id.bsReload).setOnClickListener {
            sheet.dismiss()
            val tab = currentTab()
            if (tab != null) {
                if (tab.url == NEWTAB_URL) loadNewTabHome(tab)
                else if (tab.url.startsWith("https://vn.")) renderInternalTab(tab)
                else tab.webView.reload()
            }
        }
        view.findViewById<View>(R.id.bsHistory).setOnClickListener {
            sheet.dismiss()
            openInternalTab(HISTORY_URL, "🕒 Lịch sử")
        }
        view.findViewById<View>(R.id.bsDownloads).setOnClickListener {
            sheet.dismiss()
            openInternalTab(DOWNLOADS_URL, "⬇ Tải xuống")
        }
        view.findViewById<View>(R.id.bsFind).setOnClickListener {
            sheet.dismiss()
            currentTab()?.webView?.showFindDialog("", true)
        }
        view.findViewById<View>(R.id.bsPip).setOnClickListener {
            sheet.dismiss()
            enterVideoPip()
        }
        val bgAudioRow = view.findViewById<TextView>(R.id.bsBgAudio)
        bgAudioRow.text = if (bgPlayEnabled) "🎵  Phát nhạc nền khi tắt màn: Đang bật" else "🔇  Phát nhạc nền khi tắt màn: Tắt"
        bgAudioRow.setTextColor(if (bgPlayEnabled) 0xFF34D399.toInt() else 0xFF94A3B8.toInt())
        bgAudioRow.setOnClickListener {
            sheet.dismiss()
            bgPlayEnabled = !bgPlayEnabled
            tabs.forEach { t ->
                t.webView.allowBgAudio = bgPlayEnabled
                t.webView.evaluateJavascript("window.__vnAllowBgPlay=${bgPlayEnabled};", null)
            }
            saveLocalLists()
            Toast.makeText(
                this,
                if (bgPlayEnabled) "🎵 Đã bật phát nhạc nền kể cả khi tắt màn hình!" else "Đã tắt phát nhạc nền",
                Toast.LENGTH_SHORT
            ).show()
        }
        view.findViewById<View>(R.id.bsReader).setOnClickListener {
            sheet.dismiss()
            toggleReaderMode()
        }
        val deskRow = view.findViewById<TextView>(R.id.bsDesktop)
        deskRow.text = if (desktopMode) "🖥️  Trang web cho máy tính: Đang bật" else "🖥️  Trang web cho máy tính: Tắt"
        deskRow.setOnClickListener {
            sheet.dismiss()
            toggleDesktopMode()
        }
        view.findViewById<View>(R.id.bsAutofill).setOnClickListener {
            sheet.dismiss()
            authenticateDevice("Xác minh bằng khoá máy để tự điền mật khẩu") { autofillCurrentPage() }
        }
        view.findViewById<View>(R.id.bsPasswords).setOnClickListener {
            sheet.dismiss()
            authenticateDevice("Xác minh bằng khoá máy để xem Mật khẩu đã lưu") {
                openInternalTab(PASSWORDS_URL, "🔑 Mật khẩu")
            }
        }
        view.findViewById<View>(R.id.bsSettings).setOnClickListener {
            sheet.dismiss()
            openInternalTab(SETTINGS_URL, "⚙️ Cài đặt")
        }
        val adRow = view.findViewById<TextView>(R.id.bsAdblock)
        adRow.text = if (adblockEnabled) "🛡️  Chặn quảng cáo: Đang bật" else "🚫  Chặn quảng cáo: Đang tắt"
        adRow.setTextColor(if (adblockEnabled) 0xFF34D399.toInt() else 0xFFF87171.toInt())
        adRow.setOnClickListener {
            sheet.dismiss()
            adblockEnabled = !adblockEnabled
            syncYouTubeAdblockAttr()
            updateChromeUi()
            if (currentTab()?.url != NEWTAB_URL && currentTab()?.url?.contains("youtube.com") != true) {
                currentTab()?.webView?.reload()
            }
        }
        val darkRow = view.findViewById<TextView>(R.id.bsDark)
        darkRow.text = if (darkMode) "🌙  Nền tối trang web: Đang bật" else "☀️  Nền tối trang web: Đang tắt"
        darkRow.setOnClickListener {
            sheet.dismiss()
            darkMode = !darkMode
            tabs.forEach { applyDarkModeToTab(it) }
            Toast.makeText(this, if (darkMode) "🌙 Đã bật nền tối trang web" else "☀️ Đã tắt nền tối trang web", Toast.LENGTH_SHORT).show()
        }
        sheet.show()
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
            .setTitle("VN Surf - Bảo mật")
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

    private fun enterVideoPip() {
        val tab = currentTab() ?: return
        if (tab.url.startsWith("https://vn.")) {
            Toast.makeText(this, "Hãy mở YouTube hoặc trang có video để bật PiP!", Toast.LENGTH_SHORT).show()
            return
        }
        tab.webView.evaluateJavascript(
            """
            (() => {
              const v = document.querySelector('#movie_player video.html5-main-video') || document.querySelector('video');
              if (!v) return 'NO_VIDEO';
              if (!document.getElementById('__vnPipStyle')) {
                const st = document.createElement('style');
                st.id = '__vnPipStyle';
                st.textContent = `
                  html, body { overflow: hidden !important; background: #000 !important; }
                  video {
                    position: fixed !important;
                    inset: 0 !important;
                    top: 0 !important;
                    left: 0 !important;
                    width: 100vw !important;
                    height: 100vh !important;
                    max-width: 100vw !important;
                    max-height: 100vh !important;
                    object-fit: contain !important;
                    background: #000 !important;
                    z-index: 2147483646 !important;
                    transform: none !important;
                  }
                  #masthead-container, ytm-mobile-topbar-renderer, .mobile-topbar-header,
                  #player-control-overlay, .ytp-chrome-top, .ytp-chrome-bottom,
                  #yt-topright-speedbar, #yt-speed-panel {
                    display: none !important;
                  }
                `;
                (document.head || document.documentElement).appendChild(st);
              }
              try { if (v.paused) v.play(); } catch (_) {}
              return 'OK';
            })();
            """.trimIndent()
        ) { res ->
            if (res?.contains("NO_VIDEO") == true) {
                Toast.makeText(this, "Không tìm thấy video đang mở trên trang này!", Toast.LENGTH_SHORT).show()
                return@evaluateJavascript
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                try {
                    suggestList.visibility = View.GONE
                    tabSwitcherOverlay.visibility = View.GONE
                    topBar.visibility = View.GONE
                    bottomDivider.visibility = View.GONE
                    bottomBar.visibility = View.GONE
                    mainColumn.setPadding(0, 0, 0, 0)
                    val params = PictureInPictureParams.Builder()
                        .setAspectRatio(Rational(16, 9))
                        .build()
                    enterPictureInPictureMode(params)
                } catch (_: Exception) {
                    restoreFromPipUi()
                    Toast.makeText(this, "Thiết bị không cho phép mở cửa sổ nổi PiP", Toast.LENGTH_SHORT).show()
                }
            }
        }
    }

    private fun restoreFromPipUi() {
        topBar.visibility = View.VISIBLE
        bottomDivider.visibility = View.VISIBLE
        bottomBar.visibility = View.VISIBLE
        ViewCompat.requestApplyInsets(rootFrame)
        tabs.forEach { t ->
            t.webView.evaluateJavascript("document.getElementById('__vnPipStyle')?.remove();", null)
        }
    }

    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        if (isInPictureInPictureMode) {
            topBar.visibility = View.GONE
            bottomDivider.visibility = View.GONE
            bottomBar.visibility = View.GONE
            suggestList.visibility = View.GONE
            tabSwitcherOverlay.visibility = View.GONE
            mainColumn.setPadding(0, 0, 0, 0)
        } else {
            restoreFromPipUi()
        }
    }

    override fun onStart() {
        super.onStart()
        try {
            stopService(Intent(this, MediaPlaybackService::class.java))
        } catch (_: Exception) {}
    }

    override fun onStop() {
        super.onStop()
        if (bgPlayEnabled && !isInPictureInPictureMode && !isFinishing) {
            val mediaTab = tabs.firstOrNull {
                it.hasVideo || it.url.contains("youtube.com") || it.url.contains("soundcloud.com") || it.url.contains("zingmp3.vn")
            }
            if (mediaTab != null) {
                try {
                    val svc = Intent(this, MediaPlaybackService::class.java).apply {
                        action = MediaPlaybackService.ACTION_START
                        putExtra(MediaPlaybackService.EXTRA_TITLE, mediaTab.title.ifBlank { "VN Surf Nhạc Nền" })
                    }
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                        startForegroundService(svc)
                    } else {
                        startService(svc)
                    }
                    // Ensure YouTube / HTML5 media keeps playing right across lockscreen transition
                    mainHandler.postDelayed({
                        mediaTab.webView.evaluateJavascript(
                            "(() => { const v = document.querySelector('video,audio'); if (v && v.paused && !v.ended && v.currentTime > 0) { try { v.play(); } catch(_) {} } })();",
                            null
                        )
                    }, 350)
                } catch (_: Exception) {}
            }
        }
    }

    override fun onDestroy() {
        try {
            unregisterReceiver(pauseMediaReceiver)
        } catch (_: Exception) {}
        try {
            stopService(Intent(this, MediaPlaybackService::class.java))
        } catch (_: Exception) {}
        super.onDestroy()
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
