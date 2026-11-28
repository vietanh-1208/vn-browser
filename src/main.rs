#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adblock;
mod storage;
mod ui_html;
mod updater;

use serde::{Deserialize, Serialize};
use updater::{apply_update, check_updates, UpdateStatus, APP_VERSION};
use std::collections::VecDeque;
use std::process::Command;
use std::sync::{Arc, Mutex};
use storage::{
    downloads_dir, now_secs, profile_dir, resolve_address_input, Bookmark, BrowserData,
    DownloadEntry, DownloadStatus, SavedTab, SearchEngine,
};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Fullscreen, Icon, WindowBuilder};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{NewWindowResponse, PageLoadEvent, Rect, WebContext, WebView, WebViewBuilder};

#[derive(Clone, Debug, Serialize)]
struct Suggestion {
    kind: String,
    label: String,
    url: String,
}

#[derive(Debug)]
enum UserEvent {
    Ipc(String),
    TitleChanged(u64, String),
    Navigated(u64, String),
    BlockedAd(u64),
    PopupBlocked(u64, String, String),
    PageLoad(u64, bool),
    PageError(u64),
    OpenNewTab(String, bool),
    DownloadStarted(String, String, String),
    DownloadCompleted(String, bool),
    Suggestions(u64, Vec<Suggestion>),
    UpdateStateChanged,
    OsAuthDone(bool, OsAuthAction),
}

#[derive(Debug, Clone)]
enum OsAuthAction {
    UnlockPasswords,
    Autofill,
    SetIncognitoPin(String),
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum IpcMsg {
    Ready,
    Suggest { query: String },
    NewTab { incognito: bool },
    SwitchTab { id: u64 },
    CloseTab { id: u64 },
    ReopenTab,
    TogglePin { id: u64 },
    ToggleMute { id: u64 },
    PageError { tab_id: u64 },
    SaveCredential { site: String, username: String, password: String },
    DeleteCredential { id: String },
    SleepBgTabs,
    Navigate { input: String },
    OpenUrlNewTab { url: String },
    Back,
    Forward,
    Reload,
    StopLoading,
    Home,
    Zoom { delta: f64 },
    ToggleBookmark,
    DeleteBookmark { id: String },
    TogglePanel { panel: Option<String> },
    ToggleMenu,
    ToggleFind,
    AdBlockedJs,
    UnlockIncognito { pin: String },
    UnlockPasswords,
    CreateMasterPin { pin: String },
    CopyCredential { id: String },
    RevealCredential { id: String },
    CopyText { text: String },
    PictureInPicture,
    ReaderMode,
    Find { query: String, forward: bool },
    ClearHistory,
    ClearDownloads,
    ClearAllData,
    OpenFile { path: String },
    OpenDownloadsDir,
    PrintPage,
    ToggleFullscreen,
    NextTab,
    FocusOmnibox,
    Devtools,
    SetSetting { key: String, value: serde_json::Value },
    OfferSaveCred { site: String, username: String, password: String },
    AcceptSaveCred,
    AutofillActive,
    ReorderTab { from_id: u64, to_id: u64 },
    SetDefaultBrowser,
    CheckUpdates,
    ApplyUpdate { url: String },
    OpenTranslateMenu,
    TranslatePage { #[serde(default)] lang: String },
}

#[derive(Serialize)]
struct TabView {
    id: u64,
    title: String,
    url: String,
    pinned: bool,
    incognito: bool,
    sleeping: bool,
    loading: bool,
    muted: bool,
    error_msg: bool,
    blocked_count: u32,
    zoom: f64,
    has_saved_cred: bool,
}

#[derive(Serialize)]
struct CredentialView {
    id: String,
    site: String,
    username: String,
}

#[derive(Serialize)]
struct UiState<'a> {
    tabs: Vec<TabView>,
    active_tab_id: u64,
    active_panel: &'a Option<String>,
    find_open: bool,
    menu_open: bool,
    incognito_locked: bool,
    passwords_locked: bool,
    revealed_pw: &'a Option<(String, String)>,
    total_ads_blocked: u64,
    settings: &'a storage::BrowserSettings,
    bookmarks: &'a [Bookmark],
    history: &'a [storage::HistoryEntry],
    downloads: &'a [DownloadEntry],
    credentials: Vec<CredentialView>,
    app_version: &'static str,
    update_status: UpdateStatus,
}

struct TabState {
    id: u64,
    title: String,
    url: String,
    pinned: bool,
    incognito: bool,
    sleeping: bool,
    loading: bool,
    muted: bool,
    error_msg: bool,
    blocked_count: u32,
    zoom: f64,
    last_active: u64,
    webview: Option<WebView>,
}

struct ClosedTab {
    title: String,
    url: String,
    pinned: bool,
    incognito: bool,
}

fn top_chrome_height(
    data: &BrowserData,
    find_open: bool,
    menu_open: bool,
    suggest_count: usize,
    overlay_open: bool,
    win_h: f64,
) -> f64 {
    if overlay_open {
        return win_h.max(200.0);
    }
    let mut h = 40.0 + 44.0 + 2.5;
    if data.settings.show_bookmarks_bar {
        h += 30.0;
    }
    if find_open {
        h += 36.0;
    }
    if menu_open {
        h += 410.0;
    } else if suggest_count > 0 {
        h += ((suggest_count.min(7) as f64) * 38.0 + 24.0).min(290.0);
    }
    h
}

const TAB_SHORTCUTS_SCRIPT: &str = r#"
(() => {
  if (window.__vnShortcutsBound) return;
  window.__vnShortcutsBound = true;
  document.addEventListener('keydown', (e) => {
    const send = (msg) => { if (window.ipc) window.ipc.postMessage(JSON.stringify(msg)); };
    const k = e.key.toLowerCase();
    if (e.ctrlKey && k === 't') { e.preventDefault(); send({ type: 'new_tab', incognito: e.shiftKey }); }
    else if (e.ctrlKey && k === 'w') { e.preventDefault(); send({ type: 'close_tab', id: 0 }); }
    else if (e.ctrlKey && e.shiftKey && k === 't') { e.preventDefault(); send({ type: 'reopen_tab' }); }
    else if (e.ctrlKey && k === 'l') { e.preventDefault(); send({ type: 'focus_omnibox' }); }
    else if (e.ctrlKey && k === 'f') { e.preventDefault(); send({ type: 'toggle_find' }); }
    else if (e.ctrlKey && k === 'h') { e.preventDefault(); send({ type: 'toggle_panel', panel: 'history' }); }
    else if (e.ctrlKey && k === 'j') { e.preventDefault(); send({ type: 'toggle_panel', panel: 'downloads' }); }
    else if (e.ctrlKey && k === 'd') { e.preventDefault(); send({ type: 'toggle_bookmark' }); }
    else if ((e.ctrlKey && k === 'r') || e.key === 'F5') { e.preventDefault(); send({ type: 'reload' }); }
    else if (e.key === 'F11') { e.preventDefault(); send({ type: 'toggle_fullscreen' }); }
    else if (e.key === 'F12') { e.preventDefault(); send({ type: 'devtools' }); }
    else if (e.ctrlKey && (e.key === '=' || e.key === '+')) { e.preventDefault(); send({ type: 'zoom', delta: 0.1 }); }
    else if (e.ctrlKey && e.key === '-') { e.preventDefault(); send({ type: 'zoom', delta: -0.1 }); }
    else if (e.ctrlKey && e.key === '0') { e.preventDefault(); send({ type: 'zoom', delta: 0 }); }
    else if (e.ctrlKey && e.key === 'Tab') { e.preventDefault(); send({ type: 'next_tab' }); }
  }, true);
  document.addEventListener('submit', (e) => {
    const form = e.target;
    if (!form || !form.querySelector) return;
    const pw = form.querySelector('input[type="password"]');
    if (!pw || !pw.value) return;
    const userEl = form.querySelector('input[type="email"], input[type="text"], input[type="tel"], input[name*="user" i], input[name*="mail" i], input[name*="login" i]');
    const username = userEl ? (userEl.value || '').trim() : '';
    const site = location.hostname || '';
    if (site && pw.value && window.ipc) {
      window.ipc.postMessage(JSON.stringify({ type: 'offer_save_cred', site, username, password: pw.value }));
    }
  }, true);
})();
"#;

fn create_tab_webview(
    tab_id: u64,
    url: &str,
    incognito: bool,
    bounds: Rect,
    window: &tao::window::Window,
    normal_ctx: &mut WebContext,
    incog_ctx: &mut WebContext,
    proxy: EventLoopProxy<UserEvent>,
    shared_adblock: Arc<Mutex<(bool, Vec<String>)>>,
) -> Option<WebView> {
    let target = if url.starts_with("vn://") {
        "about:blank"
    } else {
        url
    };
    let ctx = if incognito { incog_ctx } else { normal_ctx };

    let proxy_nav = proxy.clone();
    let proxy_title = proxy.clone();
    let proxy_load = proxy.clone();
    let proxy_nw = proxy.clone();
    let proxy_dl_start = proxy.clone();
    let proxy_dl_end = proxy.clone();
    let proxy_ipc = proxy.clone();
    let ab = Arc::clone(&shared_adblock);
    let ab_nw = Arc::clone(&shared_adblock);
    let cur_url_nav = Arc::new(Mutex::new(url.to_string()));
    let cur_url_nw = Arc::clone(&cur_url_nav);

    let builder = WebViewBuilder::new_with_web_context(ctx)
        .with_bounds(bounds)
        .with_visible(false)
        .with_incognito(incognito)
        .with_initialization_script(adblock::ADBLOCK_COSMETIC_SCRIPT)
        .with_initialization_script(TAB_SHORTCUTS_SCRIPT)
        .with_ipc_handler(move |req| {
            let _ = proxy_ipc.send_event(UserEvent::Ipc(req.body().to_string()));
        })
        .with_url(target)
        .with_on_page_load_handler(move |event, load_url| {
            let loading = matches!(event, PageLoadEvent::Started);
            let _ = proxy_load.send_event(UserEvent::PageLoad(tab_id, loading));
            if !loading && load_url.starts_with("chrome-error:") {
                let _ = proxy_load.send_event(UserEvent::PageError(tab_id));
            }
        })
        .with_document_title_changed_handler(move |title| {
            let _ = proxy_title.send_event(UserEvent::TitleChanged(tab_id, title));
        })
        .with_navigation_handler(move |nav_url| {
            if let Ok(guard) = ab.lock() {
                if adblock::is_blocked_url(&nav_url, &adblock::extract_host(&nav_url), guard.0, &guard.1) {
                    let _ = proxy_nav.send_event(UserEvent::BlockedAd(tab_id));
                    return false;
                }
            }
            if !nav_url.starts_with("about:") {
                if let Ok(mut u) = cur_url_nav.lock() {
                    *u = nav_url.clone();
                }
                let _ = proxy_nav.send_event(UserEvent::Navigated(tab_id, nav_url));
            }
            true
        })
        .with_new_window_req_handler(move |new_url, _features| {
            let src_url = cur_url_nw.lock().map(|u| u.clone()).unwrap_or_default();
            if let Ok(guard) = ab_nw.lock() {
                if adblock::is_blocked_url(&new_url, &adblock::extract_host(&new_url), guard.0, &guard.1) {
                    let _ = proxy_nw.send_event(UserEvent::BlockedAd(tab_id));
                    return NewWindowResponse::Deny;
                }
                if adblock::is_auto_popup_blocked(&src_url, &new_url, guard.0) {
                    let dom = adblock::extract_host(&new_url);
                    let _ = proxy_nw.send_event(UserEvent::PopupBlocked(tab_id, dom, new_url));
                    return NewWindowResponse::Deny;
                }
            }
            let _ = proxy_nw.send_event(UserEvent::OpenNewTab(new_url, incognito));
            NewWindowResponse::Deny
        })
        .with_download_started_handler(move |dl_url, dest| {
            let dir = downloads_dir();
            let _ = std::fs::create_dir_all(&dir);
            let orig_name = dest
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| "download".into());
            let mut final_dest = dir.join(&orig_name);
            if final_dest.exists() {
                let stem = final_dest
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "download".into());
                let ext = final_dest
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                for idx in 1..=999 {
                    let candidate = dir.join(format!("{} ({}){}", stem, idx, ext));
                    if !candidate.exists() {
                        final_dest = candidate;
                        break;
                    }
                }
            }
            let final_name = final_dest
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(orig_name);
            *dest = final_dest.clone();
            let _ = proxy_dl_start.send_event(UserEvent::DownloadStarted(
                dl_url,
                final_name,
                final_dest.to_string_lossy().to_string(),
            ));
            true
        })
        .with_download_completed_handler(move |dl_url, _dest, ok| {
            let _ = proxy_dl_end.send_event(UserEvent::DownloadCompleted(dl_url, ok));
        });

    builder.build_as_child(window).ok()
}

fn push_state(
    chrome: &WebView,
    tabs: &[TabState],
    active_tab_id: u64,
    active_panel: &Option<String>,
    find_open: bool,
    menu_open: bool,
    incognito_locked: bool,
    passwords_locked: bool,
    revealed_pw: &Option<(String, String)>,
    data: &BrowserData,
    update_state: &Arc<Mutex<UpdateStatus>>,
) {
    let tab_views: Vec<TabView> = tabs
        .iter()
        .map(|t| {
            let host = adblock::extract_host(&t.url);
            let has_saved_cred = !host.is_empty()
                && data.credentials.iter().any(|c| {
                    c.site.eq_ignore_ascii_case(&host)
                        || host.ends_with(&format!(".{}", c.site))
                        || c.site.ends_with(&format!(".{}", host))
                });
            TabView {
                id: t.id,
                title: t.title.clone(),
                url: t.url.clone(),
                pinned: t.pinned,
                incognito: t.incognito,
                sleeping: t.sleeping,
                loading: t.loading,
                muted: t.muted,
                error_msg: t.error_msg,
                blocked_count: t.blocked_count,
                zoom: t.zoom,
                has_saved_cred,
            }
        })
        .collect();
    let update_status = update_state
        .lock()
        .map(|g| g.clone())
        .unwrap_or(UpdateStatus::Idle);
    let st = UiState {
        tabs: tab_views,
        active_tab_id,
        active_panel,
        find_open,
        menu_open,
        incognito_locked,
        passwords_locked,
        revealed_pw,
        total_ads_blocked: data.total_ads_blocked,
        settings: &data.settings,
        bookmarks: &data.bookmarks,
        history: &data.history,
        downloads: &data.downloads,
        credentials: data
            .credentials
            .iter()
            .map(|c| CredentialView {
                id: c.id.clone(),
                site: c.site.clone(),
                username: c.username.clone(),
            })
            .collect(),
        app_version: APP_VERSION,
        update_status,
    };
    if let Ok(json) = serde_json::to_string(&st) {
        let script = format!("window.__vnUpdateState && window.__vnUpdateState({});", json);
        let _ = chrome.evaluate_script(&script);
    }
}

fn layout_views(
    window: &tao::window::Window,
    chrome: &WebView,
    tabs: &mut [TabState],
    active_tab_id: u64,
    active_panel: &Option<String>,
    find_open: bool,
    menu_open: bool,
    incognito_locked: bool,
    suggest_count: usize,
    data: &BrowserData,
) {
    let scale = window.scale_factor();
    let phys = window.inner_size();
    let win_w = phys.width as f64 / scale;
    let win_h = phys.height as f64 / scale;

    let active_tab = tabs.iter().find(|t| t.id == active_tab_id);
    let active_is_newtab = active_tab.map(|t| t.url.starts_with("vn://")).unwrap_or(false);
    let active_is_locked_incog = incognito_locked && active_tab.map(|t| t.incognito).unwrap_or(false);
    let active_has_error = active_tab.map(|t| t.error_msg).unwrap_or(false);
    let overlay_open = active_panel.is_some() || active_is_newtab || active_is_locked_incog || active_has_error;
    let top_h = top_chrome_height(data, find_open, menu_open, suggest_count, overlay_open, win_h);

    let _ = chrome.set_bounds(Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(win_w, top_h).into(),
    });

    let content_rect = Rect {
        position: LogicalPosition::new(0.0, top_h).into(),
        size: LogicalSize::new(win_w, (win_h - top_h).max(0.0)).into(),
    };

    for t in tabs.iter_mut() {
        let should_show = t.id == active_tab_id && !overlay_open && !t.sleeping;
        if let Some(wv) = &t.webview {
            let _ = wv.set_bounds(content_rect);
            let _ = wv.set_visible(should_show);
        }
    }
}

fn set_clipboard_text(text: &str) -> bool {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
        };
        use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = utf16.len() * std::mem::size_of::<u16>();
        let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes);
        if hmem.is_null() {
            return false;
        }
        let ptr = GlobalLock(hmem) as *mut u16;
        if ptr.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
        GlobalUnlock(hmem);

        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return false;
        }
        EmptyClipboard();
        let res = SetClipboardData(13, hmem); // 13 = CF_UNICODETEXT
        CloseClipboard();
        return !res.is_null();
    }
    #[cfg(not(windows))]
    {
        let _ = text;
        false
    }
}

fn open_in_shell(target: &str) {
    #[cfg(target_os = "windows")]
    let _ = Command::new("explorer").arg(target).spawn();
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(target).spawn();
    #[cfg(target_os = "linux")]
    let _ = Command::new("xdg-open").arg(target).spawn();
}

#[cfg(target_os = "windows")]
fn verify_os_user(reason: &str) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null = [Windows.Security.Credentials.UI.UserConsentVerifier, Windows.Security.Credentials.UI, ContentType=WindowsRuntime]
$asTaskGeneric = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {{ $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' }})[0]
$availOp = [Windows.Security.Credentials.UI.UserConsentVerifier]::CheckAvailabilityAsync()
$availTask = $asTaskGeneric.MakeGenericMethod([Windows.Security.Credentials.UI.UserConsentVerifierAvailability]).Invoke($null, @($availOp))
$availTask.Wait()
if ($availTask.Result -eq [Windows.Security.Credentials.UI.UserConsentVerifierAvailability]::Available) {{
    $verOp = [Windows.Security.Credentials.UI.UserConsentVerifier]::RequestVerificationAsync({})
    $verTask = $asTaskGeneric.MakeGenericMethod([Windows.Security.Credentials.UI.UserConsentVerificationResult]).Invoke($null, @($verOp))
    $verTask.Wait()
    if ($verTask.Result -eq [Windows.Security.Credentials.UI.UserConsentVerificationResult]::Verified) {{ exit 0 }} else {{ exit 1 }}
}}
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class WinCredCheck {{
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct CREDUI_INFO {{
        public int cbSize;
        public IntPtr hwndParent;
        public string pszMessageText;
        public string pszCaptionText;
        public IntPtr hbmBanner;
    }}
    [DllImport("credui.dll", CharSet = CharSet.Unicode)]
    public static extern int CredUIPromptForWindowsCredentials(ref CREDUI_INFO info, int authError, ref uint authPackage, IntPtr inAuthBuffer, uint inAuthBufferSize, out IntPtr outAuthBuffer, out uint outAuthBufferSize, ref bool save, uint flags);
    [DllImport("credui.dll", CharSet = CharSet.Unicode)]
    public static extern bool CredUnPackAuthenticationBuffer(uint flags, IntPtr authBuffer, uint authBufferSize, StringBuilder userName, ref int maxUser, StringBuilder domainName, ref int maxDomain, StringBuilder password, ref int maxPassword);
    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern bool LogonUser(string lpszUsername, string lpszDomain, string lpszPassword, int dwLogonType, int dwLogonProvider, out IntPtr phToken);
    [DllImport("kernel32.dll")]
    public static extern bool CloseHandle(IntPtr hObject);
    [DllImport("ole32.dll")]
    public static extern void CoTaskMemFree(IntPtr ptr);
    public static bool Verify(string msg) {{
        var info = new CREDUI_INFO {{ cbSize = Marshal.SizeOf(typeof(CREDUI_INFO)), pszCaptionText = "VN Browser", pszMessageText = msg }};
        uint pkg = 0; IntPtr outBuf; uint outSize; bool save = false;
        int rc = CredUIPromptForWindowsCredentials(ref info, 0, ref pkg, IntPtr.Zero, 0, out outBuf, out outSize, ref save, 0x20);
        if (rc != 0) return false;
        try {{
            var u = new StringBuilder(256); var d = new StringBuilder(256); var p = new StringBuilder(256);
            int mu = 256, md = 256, mp = 256;
            if (!CredUnPackAuthenticationBuffer(0, outBuf, outSize, u, ref mu, d, ref md, p, ref mp)) return false;
            IntPtr tok;
            string dom = d.Length > 0 ? d.ToString() : ".";
            bool ok = LogonUser(u.ToString(), dom, p.ToString(), 2, 0, out tok);
            if (ok) CloseHandle(tok);
            return ok;
        }} finally {{ CoTaskMemFree(outBuf); }}
    }}
}}
"@
if ([WinCredCheck]::Verify({})) {{ exit 0 }} else {{ exit 1 }}
"#,
        serde_json::to_string(reason).unwrap_or_else(|_| "\"VN Browser\"".into()),
        serde_json::to_string(reason).unwrap_or_else(|_| "\"VN Browser\"".into())
    );
    Command::new("powershell.exe")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn verify_os_user(_reason: &str) -> bool {
    true
}

fn spawn_os_auth(
    auth_pending: &mut bool,
    reason: &'static str,
    action: OsAuthAction,
    proxy: &EventLoopProxy<UserEvent>,
) {
    if *auth_pending {
        return;
    }
    *auth_pending = true;
    let p = proxy.clone();
    std::thread::spawn(move || {
        let ok = verify_os_user(reason);
        let _ = p.send_event(UserEvent::OsAuthDone(ok, action));
    });
}

fn autofill_active_tab(tabs: &[TabState], active_tab_id: u64, data: &BrowserData, chrome: &WebView) {
    if let Some(tab) = tabs.iter().find(|t| t.id == active_tab_id) {
        let host = adblock::extract_host(&tab.url);
        if let Some(cred) = data.credentials.iter().find(|c| {
            c.site.eq_ignore_ascii_case(&host)
                || host.ends_with(&format!(".{}", c.site))
                || c.site.ends_with(&format!(".{}", host))
        }) {
            let pw_plain = storage::decrypt_secret(&cred.password);
            if let Some(wv) = &tab.webview {
                let js = format!(
                    r#"(() => {{
  const setVal = (el, val) => {{
    if (!el) return;
    const desc = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
    if (desc && desc.set) desc.set.call(el, val); else el.value = val;
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
  }};
  const pwEl = document.querySelector('input[type="password"]');
  const form = pwEl ? pwEl.form : document;
  const userEl = (form || document).querySelector('input[type="email"], input[type="text"], input[type="tel"], input[name*="user" i], input[name*="mail" i], input[name*="login" i]');
  if (userEl) setVal(userEl, {});
  if (pwEl) setVal(pwEl, {});
}})();"#,
                    serde_json::to_string(&cred.username).unwrap_or_default(),
                    serde_json::to_string(&pw_plain).unwrap_or_default()
                );
                let _ = wv.evaluate_script(&js);
                let _ = chrome.evaluate_script(&format!(
                    "window.__vnToast && window.__vnToast({});",
                    serde_json::to_string(&format!("🔑 Đã điền tài khoản cho {}", cred.site)).unwrap_or_default()
                ));
            }
        } else {
            let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('Không tìm thấy tài khoản phù hợp cho trang này');");
        }
    }
}

fn make_window_icon() -> Option<Icon> {
    let (w, h) = (32u32, 32u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let c = 16.0_f32;
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let dx = fx - c;
            let dy = fy - c;
            let d = (dx * dx + dy * dy).sqrt();
            let i = ((y * w + x) * 4) as usize;
            if d <= 14.5 {
                if d <= 7.0 {
                    rgba[i] = 14;
                    rgba[i + 1] = 165;
                    rgba[i + 2] = 233;
                    rgba[i + 3] = 255;
                } else if d <= 9.2 {
                    rgba[i] = 255;
                    rgba[i + 1] = 255;
                    rgba[i + 2] = 255;
                    rgba[i + 3] = 255;
                } else {
                    let ang = dy.atan2(dx);
                    let col = if ang < -0.5 {
                        [16, 185, 129, 255]
                    } else if ang < 1.6 {
                        [14, 165, 233, 255]
                    } else {
                        [37, 99, 235, 255]
                    };
                    rgba[i..i + 4].copy_from_slice(&col);
                }
            }
        }
    }
    Icon::from_rgba(rgba, w, h).ok()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let mut data = BrowserData::load();
    let mut browser_args = vec!["--disable-background-networking".to_string()];
    if data.settings.doh_enabled {
        browser_args.push("--enable-features=DnsOverHttps".into());
        browser_args.push("--dns-over-https-mode=secure".into());
        browser_args.push("--dns-over-https-templates=https://cloudflare-dns.com/dns-query".into());
    }
    if !data.settings.proxy_url.trim().is_empty() {
        browser_args.push(format!("--proxy-server={}", data.settings.proxy_url.trim()));
    }
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", browser_args.join(" "));

    let mut wb = WindowBuilder::new()
        .with_title("VN Surf")
        .with_visible(false)
        .with_inner_size(LogicalSize::new(
            data.settings.window_width.max(640.0),
            data.settings.window_height.max(480.0),
        ))
        .with_maximized(data.settings.window_maximized);
    if let Some(ic) = make_window_icon() {
        wb = wb.with_window_icon(Some(ic));
    }
    if let (Some(x), Some(y)) = (data.settings.window_x, data.settings.window_y) {
        if x > -3000 && y > -3000 && x < 6000 && y < 4000 {
            wb = wb.with_position(tao::dpi::PhysicalPosition::new(x, y));
        }
    }
    let window = wb.build(&event_loop)?;
    let mut normal_ctx = WebContext::new(Some(profile_dir()));
    let mut incog_ctx = WebContext::new(None);
    let shared_adblock = Arc::new(Mutex::new((
        data.settings.adblock_enabled,
        data.settings.adblock_whitelist.clone(),
    )));

    let ipc_proxy = proxy.clone();
    let chrome = WebViewBuilder::new_with_web_context(&mut normal_ctx)
        .with_background_color((32, 33, 36, 255))
        .with_html(ui_html::CHROME_HTML)
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(1280.0, 112.0).into(),
        })
        .with_ipc_handler(move |req| {
            let _ = ipc_proxy.send_event(UserEvent::Ipc(req.body().to_string()));
        })
        .build_as_child(&window)?;

    let mut next_tab_id: u64 = 1;
    let mut tabs: Vec<TabState> = Vec::new();
    let mut closed_tabs: VecDeque<ClosedTab> = VecDeque::new();
    let mut active_tab_id: u64 = 0;
    let mut active_panel: Option<String> = None;
    let mut find_open = false;
    let mut menu_open = false;
    let mut incognito_locked = !data.settings.incognito_pin.is_empty();
    let mut passwords_locked = true;
    let mut revealed_pw: Option<(String, String)> = None;
    let mut suggest_count: usize = 0;
    let mut suggest_seq: u64 = 0;
    let update_state = Arc::new(Mutex::new(UpdateStatus::Idle));
    let mut pending_save_cred: Option<(String, String, String)> = None;
    let mut auth_pending = false;
    let mut window_shown = false;

    let initial_tabs: Vec<SavedTab> = if data.settings.restore_session && !data.last_session.is_empty() {
        data.last_session.clone()
    } else {
        vec![SavedTab {
            title: "Tab mới".into(),
            url: data.settings.homepage.clone(),
            pinned: false,
        }]
    };

    for st in initial_tabs {
        let id = next_tab_id;
        next_tab_id += 1;
        let rect = Rect {
            position: LogicalPosition::new(0.0, 112.0).into(),
            size: LogicalSize::new(1280.0, 708.0).into(),
        };
        let wv = if st.url == "vn://newtab" {
            None
        } else {
            create_tab_webview(
                id,
                &st.url,
                false,
                rect,
                &window,
                &mut normal_ctx,
                &mut incog_ctx,
                proxy.clone(),
                Arc::clone(&shared_adblock),
            )
        };
        tabs.push(TabState {
            id,
            title: st.title,
            url: st.url,
            pinned: st.pinned,
            incognito: false,
            sleeping: false,
            loading: false,
            muted: false,
            error_msg: false,
            blocked_count: 0,
            zoom: 1.0,
            last_active: now_secs(),
            webview: wv,
        });
        if active_tab_id == 0 {
            active_tab_id = id;
        }
    }

    let mut last_sleep_check = now_secs();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Auto-sleep background tabs check
        let now = now_secs();
        if now.saturating_sub(last_sleep_check) >= 30 {
            last_sleep_check = now;
            if data.settings.sleep_minutes > 0 {
                let threshold = data.settings.sleep_minutes * 60;
                let mut changed = false;
                for t in tabs.iter_mut() {
                    if t.id != active_tab_id
                        && !t.pinned
                        && !t.sleeping
                        && t.webview.is_some()
                        && now.saturating_sub(t.last_active) >= threshold
                    {
                        t.webview = None;
                        t.sleeping = true;
                        changed = true;
                    }
                }
                if changed {
                    push_state(&chrome, &tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, passwords_locked, &revealed_pw, &data, &update_state);
                }
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let maximized = window.is_maximized();
                data.settings.window_maximized = maximized;
                if !maximized {
                    let scale = window.scale_factor();
                    let sz = window.inner_size();
                    data.settings.window_width = (sz.width as f64 / scale).max(640.0);
                    data.settings.window_height = (sz.height as f64 / scale).max(480.0);
                    if let Ok(pos) = window.outer_position() {
                        data.settings.window_x = Some(pos.x);
                        data.settings.window_y = Some(pos.y);
                    }
                }
                data.last_session = tabs
                    .iter()
                    .filter(|t| !t.incognito)
                    .map(|t| SavedTab {
                        title: t.title.clone(),
                        url: t.url.clone(),
                        pinned: t.pinned,
                    })
                    .collect();
                data.save();
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                layout_views(&window, &chrome, &mut tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, suggest_count, &data);
            }
            Event::UserEvent(ue) => {
                match ue {
                    UserEvent::TitleChanged(tid, title) => {
                        if let Some(t) = tabs.iter_mut().find(|t| t.id == tid) {
                            if !title.trim().is_empty() {
                                t.title = title.clone();
                                if !t.incognito {
                                    data.record_history(&title, &t.url);
                                }
                            }
                        }
                    }
                    UserEvent::Navigated(tid, url) => {
                        if let Some(t) = tabs.iter_mut().find(|t| t.id == tid) {
                            t.url = url.clone();
                            t.blocked_count = 0;
                            t.error_msg = false;
                            if !t.incognito {
                                data.record_history(&t.title, &url);
                                data.save();
                            }
                        }
                    }
                    UserEvent::BlockedAd(tid) => {
                        data.total_ads_blocked += 1;
                        if let Some(t) = tabs.iter_mut().find(|t| t.id == tid) {
                            t.blocked_count += 1;
                        }
                    }
                    UserEvent::PopupBlocked(tid, dom, url) => {
                        data.total_ads_blocked += 1;
                        if let Some(t) = tabs.iter_mut().find(|t| t.id == tid) {
                            t.blocked_count += 1;
                        }
                        let _ = chrome.evaluate_script(&format!(
                            "window.__vnPopupBlocked && window.__vnPopupBlocked({}, {});",
                            serde_json::to_string(&dom).unwrap_or_default(),
                            serde_json::to_string(&url).unwrap_or_default()
                        ));
                    }
                    UserEvent::PageLoad(tid, loading) => {
                        if let Some(t) = tabs.iter_mut().find(|t| t.id == tid) {
                            t.loading = loading;
                            if !loading {
                                if let Some(wv) = &t.webview {
                                    let _ = wv.evaluate_script(&format!(
                                        "if (location.protocol === 'chrome-error:' || (document.body && /ERR_CONNECTION_REFUSED|ERR_NAME_NOT_RESOLVED|can't reach this page/i.test(document.body.innerText))) {{ window.ipc && window.ipc.postMessage(JSON.stringify({{type:'page_error',tab_id:{}}})); }}",
                                        tid
                                    ));
                                }
                            }
                        }
                    }
                    UserEvent::PageError(tid) => {
                        if let Some(t) = tabs.iter_mut().find(|t| t.id == tid) {
                            t.error_msg = true;
                            t.loading = false;
                        }
                    }
                    UserEvent::OpenNewTab(url, incognito) => {
                        let id = next_tab_id;
                        next_tab_id += 1;
                        let rect = Rect {
                            position: LogicalPosition::new(0.0, 112.0).into(),
                            size: LogicalSize::new(1280.0, 708.0).into(),
                        };
                        let wv = create_tab_webview(
                            id,
                            &url,
                            incognito,
                            rect,
                            &window,
                            &mut normal_ctx,
                            &mut incog_ctx,
                            proxy.clone(),
                            Arc::clone(&shared_adblock),
                        );
                        tabs.push(TabState {
                            id,
                            title: url.clone(),
                            url,
                            pinned: false,
                            incognito,
                            sleeping: false,
                            loading: false,
                            muted: false,
                            error_msg: false,
                            blocked_count: 0,
                            zoom: 1.0,
                            last_active: now_secs(),
                            webview: wv,
                        });
                        active_tab_id = id;
                        active_panel = None;
                    }
                    UserEvent::DownloadStarted(url, filename, path) => {
                        let _ = chrome.evaluate_script(&format!(
                            "window.__vnToast && window.__vnToast({});",
                            serde_json::to_string(&format!("⬇ Đang tải {}...", filename)).unwrap_or_default()
                        ));
                        data.downloads.insert(
                            0,
                            DownloadEntry {
                                id: format!("dl_{}", now_secs()),
                                filename,
                                url,
                                path,
                                status: DownloadStatus::Downloading,
                                started_at: now_secs(),
                            },
                        );
                        data.save();
                    }
                    UserEvent::DownloadCompleted(url, ok) => {
                        if let Some(d) = data.downloads.iter_mut().find(|d| d.url == url && d.status == DownloadStatus::Downloading) {
                            d.status = if ok { DownloadStatus::Completed } else { DownloadStatus::Failed };
                            let msg = if ok {
                                format!("✓ Đã tải xong: {}", d.filename)
                            } else {
                                format!("✕ Tải thất bại: {}", d.filename)
                            };
                            let _ = chrome.evaluate_script(&format!(
                                "window.__vnToast && window.__vnToast({});",
                                serde_json::to_string(&msg).unwrap_or_default()
                            ));
                            data.save();
                        }
                    }
                    UserEvent::Suggestions(seq, items) => {
                        if seq != suggest_seq {
                            return;
                        }
                        suggest_count = items.len();
                        if let Ok(json) = serde_json::to_string(&items) {
                            let _ = chrome.evaluate_script(&format!(
                                "window.__vnShowSuggestions && window.__vnShowSuggestions({});",
                                json
                            ));
                        }
                        layout_views(&window, &chrome, &mut tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, suggest_count, &data);
                        return;
                    }
                    UserEvent::UpdateStateChanged => {}
                    UserEvent::OsAuthDone(ok, action) => {
                        auth_pending = false;
                        if ok {
                            match action {
                                OsAuthAction::UnlockPasswords => {
                                    passwords_locked = false;
                                }
                                OsAuthAction::Autofill => {
                                    passwords_locked = false;
                                    autofill_active_tab(&tabs, active_tab_id, &data, &chrome);
                                }
                                OsAuthAction::SetIncognitoPin(new_pin) => {
                                    data.settings.incognito_pin = new_pin;
                                    incognito_locked = !data.settings.incognito_pin.is_empty();
                                    data.save();
                                    let msg = if incognito_locked {
                                        "🔒 Đã cập nhật mã PIN khoá Tab Ẩn danh!"
                                    } else {
                                        "🔓 Đã gỡ mã PIN Tab Ẩn danh."
                                    };
                                    let _ = chrome.evaluate_script(&format!(
                                        "window.__vnToast && window.__vnToast({});",
                                        serde_json::to_string(msg).unwrap_or_default()
                                    ));
                                }
                            }
                        } else {
                            let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('🔒 Xác minh mật khẩu máy thất bại hoặc đã huỷ!');");
                        }
                    }
                    UserEvent::Ipc(raw) => {
                        let Ok(msg) = serde_json::from_str::<IpcMsg>(&raw) else { return };
                        if !matches!(msg, IpcMsg::ToggleMenu | IpcMsg::Zoom { .. } | IpcMsg::AdBlockedJs | IpcMsg::Ready) {
                            menu_open = false;
                        }
                        match msg {
                            IpcMsg::Ready => {
                                if !window_shown {
                                    window_shown = true;
                                    layout_views(&window, &chrome, &mut tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, suggest_count, &data);
                                    push_state(&chrome, &tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, passwords_locked, &revealed_pw, &data, &update_state);
                                    window.set_visible(true);
                                }
                            }
                            IpcMsg::Suggest { query } => {
                                suggest_seq += 1;
                                let seq = suggest_seq;
                                let q = query.trim().to_string();
                                if q.is_empty() {
                                    suggest_count = 0;
                                    layout_views(&window, &chrome, &mut tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, suggest_count, &data);
                                    return;
                                }
                                let ql = q.to_lowercase();
                                let mut local_items: Vec<Suggestion> = Vec::new();
                                for b in &data.bookmarks {
                                    if b.title.to_lowercase().contains(&ql) || b.url.to_lowercase().contains(&ql) {
                                        local_items.push(Suggestion {
                                            kind: "bookmark".into(),
                                            label: b.title.clone(),
                                            url: b.url.clone(),
                                        });
                                        if local_items.len() >= 3 {
                                            break;
                                        }
                                    }
                                }
                                for h in &data.history {
                                    if local_items.len() >= 4 {
                                        break;
                                    }
                                    if h.url.contains("google.com/goto?") || h.url.contains("google.com/url?") {
                                        continue;
                                    }
                                    if (h.title.to_lowercase().contains(&ql) || h.url.to_lowercase().contains(&ql))
                                        && !local_items.iter().any(|x| x.url == h.url)
                                    {
                                        local_items.push(Suggestion {
                                            kind: "history".into(),
                                            label: h.title.clone(),
                                            url: h.url.clone(),
                                        });
                                    }
                                }
                                let p_sug = proxy.clone();
                                std::thread::spawn(move || {
                                    let mut items = local_items;
                                    let enc: String = q
                                        .bytes()
                                        .map(|b| match b {
                                            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                                            b' ' => "+".to_string(),
                                            _ => format!("%{:02X}", b),
                                        })
                                        .collect();
                                    let url = format!("https://suggestqueries.google.com/complete/search?client=firefox&q={}", enc);
                                    if let Ok(client) = reqwest::blocking::Client::builder()
                                        .timeout(std::time::Duration::from_secs(3))
                                        .build()
                                    {
                                        if let Ok(resp) = client.get(&url).send() {
                                            if let Ok(val) = resp.json::<serde_json::Value>() {
                                                if let Some(arr) = val.get(1).and_then(|v| v.as_array()) {
                                                    for entry in arr {
                                                        if items.len() >= 7 {
                                                            break;
                                                        }
                                                        if let Some(s) = entry.as_str() {
                                                            items.push(Suggestion {
                                                                kind: "search".into(),
                                                                label: s.to_string(),
                                                                url: s.to_string(),
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let _ = p_sug.send_event(UserEvent::Suggestions(seq, items));
                                });
                                return;
                            }
                            IpcMsg::NewTab { incognito } => {
                                suggest_seq += 1;
                                suggest_count = 0;
                                let id = next_tab_id;
                                next_tab_id += 1;
                                tabs.push(TabState {
                                    id,
                                    title: if incognito { "Tab Ẩn danh".into() } else { "Tab mới".into() },
                                    url: "vn://newtab".into(),
                                    pinned: false,
                                    incognito,
                                    sleeping: false,
                                    loading: false,
                                    muted: false,
                                    error_msg: false,
                                    blocked_count: 0,
                                    zoom: 1.0,
                                    last_active: now_secs(),
                                    webview: None,
                                });
                                active_tab_id = id;
                                active_panel = None;
                            }
                            IpcMsg::SwitchTab { id } => {
                                suggest_seq += 1;
                                suggest_count = 0;
                                active_tab_id = id;
                                active_panel = None;
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == id) {
                                    t.last_active = now_secs();
                                    if t.sleeping && t.url != "vn://newtab" {
                                        t.sleeping = false;
                                        let rect = Rect {
                                            position: LogicalPosition::new(0.0, 112.0).into(),
                                            size: LogicalSize::new(1280.0, 708.0).into(),
                                        };
                                        t.webview = create_tab_webview(
                                            t.id,
                                            &t.url,
                                            t.incognito,
                                            rect,
                                            &window,
                                            &mut normal_ctx,
                                            &mut incog_ctx,
                                            proxy.clone(),
                                            Arc::clone(&shared_adblock),
                                        );
                                    }
                                }
                            }
                            IpcMsg::CloseTab { id } => {
                                let target_id = if id == 0 { active_tab_id } else { id };
                                if let Some(pos) = tabs.iter().position(|t| t.id == target_id) {
                                    let removed = tabs.remove(pos);
                                    if !removed.incognito {
                                        closed_tabs.push_front(ClosedTab {
                                            title: removed.title,
                                            url: removed.url,
                                            pinned: removed.pinned,
                                            incognito: removed.incognito,
                                        });
                                        if closed_tabs.len() > 20 {
                                            closed_tabs.pop_back();
                                        }
                                    }
                                    if tabs.is_empty() {
                                        let nid = next_tab_id;
                                        next_tab_id += 1;
                                        tabs.push(TabState {
                                            id: nid,
                                            title: "Tab mới".into(),
                                            url: "vn://newtab".into(),
                                            pinned: false,
                                            incognito: false,
                                            sleeping: false,
                                            loading: false,
                                            muted: false,
                                            error_msg: false,
                                            blocked_count: 0,
                                            zoom: 1.0,
                                            last_active: now_secs(),
                                            webview: None,
                                        });
                                        active_tab_id = nid;
                                    } else if active_tab_id == target_id {
                                        let next_idx = pos.min(tabs.len() - 1);
                                        active_tab_id = tabs[next_idx].id;
                                    }
                                }
                            }
                            IpcMsg::ReopenTab => {
                                if let Some(ct) = closed_tabs.pop_front() {
                                    let id = next_tab_id;
                                    next_tab_id += 1;
                                    let rect = Rect {
                                        position: LogicalPosition::new(0.0, 112.0).into(),
                                        size: LogicalSize::new(1280.0, 708.0).into(),
                                    };
                                    let wv = if ct.url == "vn://newtab" {
                                        None
                                    } else {
                                        create_tab_webview(
                                            id,
                                            &ct.url,
                                            ct.incognito,
                                            rect,
                                            &window,
                                            &mut normal_ctx,
                                            &mut incog_ctx,
                                            proxy.clone(),
                                            Arc::clone(&shared_adblock),
                                        )
                                    };
                                    tabs.push(TabState {
                                        id,
                                        title: ct.title,
                                        url: ct.url,
                                        pinned: ct.pinned,
                                        incognito: ct.incognito,
                                        sleeping: false,
                                        loading: false,
                                        muted: false,
                                        error_msg: false,
                                        blocked_count: 0,
                                        zoom: 1.0,
                                        last_active: now_secs(),
                                        webview: wv,
                                    });
                                    active_tab_id = id;
                                    active_panel = None;
                                }
                            }
                            IpcMsg::TogglePin { id } => {
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == id) {
                                    t.pinned = !t.pinned;
                                }
                            }
                            IpcMsg::ToggleMute { id } => {
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == id) {
                                    t.muted = !t.muted;
                                    if let Some(wv) = &t.webview {
                                        let _ = wv.evaluate_script(&format!(
                                            "window.__vnMuted = {}; document.querySelectorAll('video, audio').forEach(el => el.muted = window.__vnMuted);",
                                            t.muted
                                        ));
                                    }
                                }
                            }
                            IpcMsg::PageError { tab_id } => {
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == tab_id) {
                                    t.error_msg = true;
                                    t.loading = false;
                                }
                            }
                            IpcMsg::SaveCredential { site, username, password } => {
                                if !site.trim().is_empty() && !username.trim().is_empty() && !password.is_empty() {
                                    data.credentials.insert(
                                        0,
                                        storage::SavedCredential {
                                            id: format!("pw_{}", now_secs()),
                                            site: site.trim().to_string(),
                                            username: username.trim().to_string(),
                                            password: storage::encrypt_secret(&password),
                                        },
                                    );
                                    data.save();
                                    let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('🔐 Đã mã hoá & lưu mật khẩu!');");
                                }
                            }
                            IpcMsg::CopyCredential { id } => {
                                if passwords_locked {
                                    let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('🔒 Cần xác minh mật khẩu máy trước!');");
                                } else if let Some(c) = data.credentials.iter().find(|c| c.id == id) {
                                    let plain = storage::decrypt_secret(&c.password);
                                    if set_clipboard_text(&plain) {
                                        let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('✓ Đã sao chép mật khẩu!');");
                                    } else {
                                        let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('❌ Không thể ghi vào Clipboard!');");
                                    }
                                }
                            }
                            IpcMsg::RevealCredential { id } => {
                                if !passwords_locked {
                                    if revealed_pw.as_ref().map(|(rid, _)| rid == &id).unwrap_or(false) {
                                        revealed_pw = None;
                                    } else if let Some(c) = data.credentials.iter().find(|c| c.id == id) {
                                        revealed_pw = Some((id, storage::decrypt_secret(&c.password)));
                                    }
                                }
                            }
                            IpcMsg::CopyText { text } => {
                                if set_clipboard_text(&text) {
                                    let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('✓ Đã sao chép!');");
                                }
                            }
                            IpcMsg::DeleteCredential { id } => {
                                data.credentials.retain(|c| c.id != id);
                                data.save();
                            }
                            IpcMsg::SleepBgTabs => {
                                let mut count = 0usize;
                                for t in tabs.iter_mut() {
                                    if t.id != active_tab_id && !t.pinned && !t.sleeping && t.webview.is_some() {
                                        t.webview = None;
                                        t.sleeping = true;
                                        count += 1;
                                    }
                                }
                                let msg = if count > 0 {
                                    format!("⚡ Đã cho {} tab nền ngủ để tiết kiệm RAM!", count)
                                } else {
                                    "⚡ Hiện chưa có tab nền nào đang mở để ngủ.".into()
                                };
                                let _ = chrome.evaluate_script(&format!(
                                    "window.__vnToast && window.__vnToast({});",
                                    serde_json::to_string(&msg).unwrap_or_default()
                                ));
                            }
                            IpcMsg::OpenUrlNewTab { url } => {
                                let inc = tabs.iter().find(|t| t.id == active_tab_id).map(|t| t.incognito).unwrap_or(false);
                                let _ = proxy.send_event(UserEvent::OpenNewTab(url, inc));
                            }
                            IpcMsg::Navigate { input } => {
                                suggest_seq += 1;
                                suggest_count = 0;
                                let target = resolve_address_input(&input, &data.settings.search_engine);
                                active_panel = None;
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == active_tab_id) {
                                    t.url = target.clone();
                                    t.sleeping = false;
                                    t.error_msg = false;
                                    t.last_active = now_secs();
                                    if target == "vn://newtab" {
                                        t.webview = None;
                                        t.title = "Tab mới".into();
                                    } else if let Some(wv) = &t.webview {
                                        let _ = wv.load_url(&target);
                                    } else {
                                        let rect = Rect {
                                            position: LogicalPosition::new(0.0, 112.0).into(),
                                            size: LogicalSize::new(1280.0, 708.0).into(),
                                        };
                                        t.webview = create_tab_webview(
                                            t.id,
                                            &target,
                                            t.incognito,
                                            rect,
                                            &window,
                                            &mut normal_ctx,
                                            &mut incog_ctx,
                                            proxy.clone(),
                                            Arc::clone(&shared_adblock),
                                        );
                                    }
                                }
                            }
                            IpcMsg::Back => {
                                if let Some(wv) = tabs.iter().find(|t| t.id == active_tab_id).and_then(|t| t.webview.as_ref()) {
                                    let _ = wv.evaluate_script("history.back()");
                                }
                            }
                            IpcMsg::Forward => {
                                if let Some(wv) = tabs.iter().find(|t| t.id == active_tab_id).and_then(|t| t.webview.as_ref()) {
                                    let _ = wv.evaluate_script("history.forward()");
                                }
                            }
                            IpcMsg::Reload => {
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == active_tab_id) {
                                    t.error_msg = false;
                                    if let Some(wv) = &t.webview {
                                        let _ = wv.evaluate_script("location.reload()");
                                    }
                                }
                            }
                            IpcMsg::StopLoading => {
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == active_tab_id) {
                                    t.loading = false;
                                    if let Some(wv) = &t.webview {
                                        let _ = wv.evaluate_script("window.stop()");
                                    }
                                }
                            }
                            IpcMsg::Home => {
                                active_panel = None;
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == active_tab_id) {
                                    t.url = "vn://newtab".into();
                                    t.title = "Tab mới".into();
                                    t.webview = None;
                                }
                            }
                            IpcMsg::Zoom { delta } => {
                                if let Some(t) = tabs.iter_mut().find(|t| t.id == active_tab_id) {
                                    t.zoom = if delta == 0.0 { 1.0 } else { (t.zoom + delta).clamp(0.5, 2.5) };
                                    if let Some(wv) = &t.webview {
                                        let _ = wv.zoom(t.zoom);
                                    }
                                }
                            }
                            IpcMsg::ToggleBookmark => {
                                if let Some(t) = tabs.iter().find(|t| t.id == active_tab_id) {
                                    if !t.url.starts_with("vn://") {
                                        if let Some(idx) = data.bookmarks.iter().position(|b| b.url == t.url) {
                                            data.bookmarks.remove(idx);
                                        } else {
                                            data.bookmarks.push(Bookmark {
                                                id: format!("bm_{}", now_secs()),
                                                title: t.title.clone(),
                                                url: t.url.clone(),
                                                created_at: now_secs(),
                                            });
                                        }
                                        data.save();
                                    }
                                }
                            }
                            IpcMsg::DeleteBookmark { id } => {
                                data.bookmarks.retain(|b| b.id != id);
                                data.save();
                            }
                            IpcMsg::TogglePanel { panel } => {
                                suggest_count = 0;
                                menu_open = false;
                                passwords_locked = true;
                                revealed_pw = None;
                                if let Some(p) = panel {
                                    let vn_url = format!("vn://{}", p);
                                    if let Some(existing) = tabs.iter().find(|t| t.url == vn_url) {
                                        active_tab_id = existing.id;
                                    } else {
                                        let nid = next_tab_id;
                                        next_tab_id += 1;
                                        let title = match p.as_str() {
                                            "downloads" => "⬇ Tải xuống",
                                            "history" => "🕒 Lịch sử",
                                            "bookmarks" => "★ Dấu trang",
                                            "passwords" => "🔑 Mật khẩu",
                                            "settings" => "⚙️ Cài đặt",
                                            _ => "VN Surf",
                                        }
                                        .to_string();
                                        tabs.push(TabState {
                                            id: nid,
                                            title,
                                            url: vn_url,
                                            pinned: false,
                                            incognito: false,
                                            sleeping: false,
                                            loading: false,
                                            muted: false,
                                            error_msg: false,
                                            blocked_count: 0,
                                            zoom: 1.0,
                                            last_active: now_secs(),
                                            webview: None,
                                        });
                                        active_tab_id = nid;
                                    }
                                    active_panel = None;
                                    if p == "passwords" {
                                        spawn_os_auth(
                                            &mut auth_pending,
                                            "VN Surf: Xác minh mật khẩu máy để mở khoá Mật khẩu đã lưu",
                                            OsAuthAction::UnlockPasswords,
                                            &proxy,
                                        );
                                    }
                                } else {
                                    active_panel = None;
                                    if let Some(pos) = tabs.iter().position(|t| t.id == active_tab_id) {
                                        if tabs[pos].url.starts_with("vn://") && tabs[pos].url != "vn://newtab" {
                                            if tabs.len() > 1 {
                                                tabs.remove(pos);
                                                active_tab_id = tabs[pos.min(tabs.len() - 1)].id;
                                            } else {
                                                tabs[pos].url = "vn://newtab".into();
                                                tabs[pos].title = "Tab mới".into();
                                            }
                                        }
                                    }
                                }
                            }
                            IpcMsg::ToggleMenu => {
                                menu_open = !menu_open;
                            }
                            IpcMsg::AdBlockedJs => {
                                if data.settings.adblock_enabled {
                                    data.total_ads_blocked += 1;
                                    if let Some(t) = tabs.iter_mut().find(|t| t.id == active_tab_id) {
                                        t.blocked_count += 1;
                                    }
                                }
                            }
                            IpcMsg::ToggleFind => {
                                menu_open = false;
                                find_open = !find_open;
                            }
                            IpcMsg::Find { query, forward } => {
                                if let Some(wv) = tabs.iter().find(|t| t.id == active_tab_id).and_then(|t| t.webview.as_ref()) {
                                    if let Ok(q) = serde_json::to_string(&query) {
                                        let _ = wv.evaluate_script(&format!(
                                            "window.find({}, false, {}, true, false, false, false);",
                                            q, !forward
                                        ));
                                    }
                                }
                            }
                            IpcMsg::ClearHistory => {
                                data.history.clear();
                                data.save();
                            }
                            IpcMsg::ClearDownloads => {
                                data.downloads.clear();
                                data.save();
                            }
                            IpcMsg::ClearAllData => {
                                data.history.clear();
                                data.downloads.clear();
                                data.last_session.clear();
                                data.save();
                            }
                            IpcMsg::OpenFile { path } => {
                                open_in_shell(&path);
                            }
                            IpcMsg::OpenDownloadsDir => {
                                open_in_shell(&downloads_dir().to_string_lossy());
                            }
                            IpcMsg::PrintPage => {
                                if let Some(wv) = tabs.iter().find(|t| t.id == active_tab_id).and_then(|t| t.webview.as_ref()) {
                                    let _ = wv.evaluate_script("window.print()");
                                }
                            }
                            IpcMsg::ToggleFullscreen => {
                                if window.fullscreen().is_some() {
                                    window.set_fullscreen(None);
                                } else {
                                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                }
                            }
                            IpcMsg::NextTab => {
                                if tabs.len() > 1 {
                                    if let Some(pos) = tabs.iter().position(|t| t.id == active_tab_id) {
                                        let next = (pos + 1) % tabs.len();
                                        active_tab_id = tabs[next].id;
                                        active_panel = None;
                                    }
                                }
                            }
                            IpcMsg::UnlockIncognito { pin } => {
                                if pin == data.settings.incognito_pin {
                                    incognito_locked = false;
                                } else {
                                    let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('❌ Sai mã PIN!');");
                                }
                            }
                            IpcMsg::UnlockPasswords => {
                                spawn_os_auth(
                                    &mut auth_pending,
                                    "VN Browser: Xác minh mật khẩu máy để mở khoá Mật khẩu đã lưu",
                                    OsAuthAction::UnlockPasswords,
                                    &proxy,
                                );
                            }
                            IpcMsg::CreateMasterPin { pin } => {
                                let p = pin.trim().to_string();
                                if p.len() < 4 {
                                    let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('⚠️ Mã PIN phải có ít nhất 4 ký tự!');");
                                } else {
                                    data.settings.incognito_pin = p;
                                    data.save();
                                    passwords_locked = false;
                                    incognito_locked = false;
                                    let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('🔐 Đã tạo mã PIN bảo vệ!');");
                                }
                            }
                            IpcMsg::PictureInPicture => {
                                if let Some(wv) = tabs.iter().find(|t| t.id == active_tab_id).and_then(|t| t.webview.as_ref()) {
                                    let _ = wv.evaluate_script("const v = document.querySelector('video'); if (v && document.pictureInPictureEnabled) { if (document.pictureInPictureElement) document.exitPictureInPicture(); else v.requestPictureInPicture().catch(()=>{}); }");
                                }
                            }
                            IpcMsg::ReaderMode => {
                                if let Some(wv) = tabs.iter().find(|t| t.id == active_tab_id).and_then(|t| t.webview.as_ref()) {
                                    let _ = wv.evaluate_script("(() => { if (document.getElementById('__vnReader')) { document.getElementById('__vnReader').remove(); return; } const art = document.querySelector('article, main, [role=main]') || document.body; const ov = document.createElement('div'); ov.id='__vnReader'; ov.style='position:fixed;inset:0;z-index:2147483647;background:#0f172a;color:#e2e8f0;overflow:auto;padding:48px 20px;font-family:Georgia,serif;line-height:1.8;font-size:19px;'; ov.innerHTML='<div style=\"max-width:740px;margin:0 auto\"><button onclick=\"this.closest(\\'#__vnReader\\').remove()\" style=\"position:fixed;top:16px;right:24px;padding:8px 14px;border-radius:8px;border:none;background:#22c55e;color:#fff;font-weight:700;cursor:pointer\">✕ Thoát chế độ đọc</button><h1 style=\"font-size:28px;margin-bottom:20px\">'+document.title+'</h1><div>'+art.innerHTML+'</div></div>'; document.body.appendChild(ov); })()");
                                }
                            }
                            IpcMsg::FocusOmnibox => {
                                let _ = chrome.focus();
                                let _ = chrome.evaluate_script("const u = document.getElementById('urlInput'); if (u) { u.focus(); u.select(); }");
                            }
                            IpcMsg::Devtools => {
                                if let Some(wv) = tabs.iter().find(|t| t.id == active_tab_id).and_then(|t| t.webview.as_ref()) {
                                    if wv.is_devtools_open() {
                                        wv.close_devtools();
                                    } else {
                                        wv.open_devtools();
                                    }
                                }
                            }
                            IpcMsg::SetSetting { key, value } => {
                                match key.as_str() {
                                    "search_engine" => {
                                        data.settings.search_engine = match value.as_str().unwrap_or("Google") {
                                            "DuckDuckGo" => SearchEngine::DuckDuckGo,
                                            "Bing" => SearchEngine::Bing,
                                            _ => SearchEngine::Google,
                                        };
                                    }
                                    "dark_mode" => {
                                        data.settings.dark_mode = value.as_bool().unwrap_or(true);
                                    }
                                    "accent_color" => {
                                        if let Some(hex) = value.as_str() {
                                            data.settings.accent_color = hex.to_string();
                                        }
                                    }
                                    "adblock_enabled" => {
                                        data.settings.adblock_enabled = value.as_bool().unwrap_or(true);
                                        if let Ok(mut ab) = shared_adblock.lock() {
                                            ab.0 = data.settings.adblock_enabled;
                                        }
                                    }
                                    "show_bookmarks_bar" => {
                                        data.settings.show_bookmarks_bar = value.as_bool().unwrap_or(true);
                                    }
                                    "sleep_minutes" => {
                                        data.settings.sleep_minutes = value.as_u64().unwrap_or(15);
                                    }
                                    "restore_session" => {
                                        data.settings.restore_session = value.as_bool().unwrap_or(true);
                                    }
                                    "doh_enabled" => {
                                        data.settings.doh_enabled = value.as_bool().unwrap_or(true);
                                        let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('🌐 Đã lưu cài đặt DoH (khởi động lại app để áp dụng)');");
                                    }
                                    "proxy_url" => {
                                        data.settings.proxy_url = value.as_str().unwrap_or("").trim().to_string();
                                        let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('🌐 Đã lưu Proxy (khởi động lại app để áp dụng)');");
                                    }
                                    "incognito_pin" => {
                                        let new_pin = value.as_str().unwrap_or("").trim().to_string();
                                        if !data.settings.incognito_pin.is_empty() {
                                            spawn_os_auth(
                                                &mut auth_pending,
                                                "VN Surf: Xác minh mật khẩu máy để thay đổi hoặc gỡ mã PIN Ẩn danh",
                                                OsAuthAction::SetIncognitoPin(new_pin),
                                                &proxy,
                                            );
                                        } else {
                                            data.settings.incognito_pin = new_pin;
                                            incognito_locked = !data.settings.incognito_pin.is_empty();
                                            let msg = if incognito_locked {
                                                "🔒 Đã bật mã PIN khoá Tab Ẩn danh!"
                                            } else {
                                                "🔓 Đã gỡ mã PIN Tab Ẩn danh."
                                            };
                                            let _ = chrome.evaluate_script(&format!(
                                                "window.__vnToast && window.__vnToast({});",
                                                serde_json::to_string(msg).unwrap_or_default()
                                            ));
                                        }
                                    }
                                    _ => {}
                                }
                                data.save();
                            }
                            IpcMsg::OfferSaveCred { site, username, password } => {
                                let is_incog = tabs.iter().find(|t| t.id == active_tab_id).map(|t| t.incognito).unwrap_or(false);
                                if !is_incog && !site.is_empty() && !password.is_empty() {
                                    let already = data.credentials.iter().any(|c| {
                                        c.site.eq_ignore_ascii_case(&site)
                                            && c.username == username
                                            && storage::decrypt_secret(&c.password) == password
                                    });
                                    if !already {
                                        pending_save_cred = Some((site.clone(), username.clone(), password));
                                        let _ = chrome.evaluate_script(&format!(
                                            "window.__vnOfferSaveCred && window.__vnOfferSaveCred({}, {});",
                                            serde_json::to_string(&site).unwrap_or_default(),
                                            serde_json::to_string(&username).unwrap_or_default()
                                        ));
                                    }
                                }
                            }
                            IpcMsg::AcceptSaveCred => {
                                if let Some((site, username, password)) = pending_save_cred.take() {
                                    let enc = storage::encrypt_secret(&password);
                                    if let Some(existing) = data.credentials.iter_mut().find(|c| c.site.eq_ignore_ascii_case(&site) && c.username == username) {
                                        existing.password = enc;
                                    } else {
                                        data.credentials.insert(0, storage::SavedCredential {
                                            id: format!("cred_{}", now_secs()),
                                            site: site.clone(),
                                            username,
                                            password: enc,
                                        });
                                    }
                                    data.save();
                                    let _ = chrome.evaluate_script(&format!(
                                        "window.__vnToast && window.__vnToast({});",
                                        serde_json::to_string(&format!("🔑 Đã lưu mật khẩu cho {}", site)).unwrap_or_default()
                                    ));
                                }
                            }
                            IpcMsg::AutofillActive => {
                                if passwords_locked {
                                    spawn_os_auth(
                                        &mut auth_pending,
                                        "VN Browser: Xác minh mật khẩu máy để tự điền tài khoản",
                                        OsAuthAction::Autofill,
                                        &proxy,
                                    );
                                } else {
                                    autofill_active_tab(&tabs, active_tab_id, &data, &chrome);
                                }
                            }
                            IpcMsg::ReorderTab { from_id, to_id } => {
                                if from_id != to_id {
                                    if let (Some(from_idx), Some(to_idx)) = (
                                        tabs.iter().position(|t| t.id == from_id),
                                        tabs.iter().position(|t| t.id == to_id),
                                    ) {
                                        let item = tabs.remove(from_idx);
                                        tabs.insert(to_idx, item);
                                    }
                                }
                            }
                            IpcMsg::SetDefaultBrowser => {
                                #[cfg(target_os = "windows")]
                                {
                                    let _ = Command::new("cmd").args(["/c", "start", "ms-settings:defaultapps"]).spawn();
                                }
                                let _ = chrome.evaluate_script("window.__vnToast && window.__vnToast('🌐 Đã mở Cài đặt Ứng dụng mặc định!');");
                            }
                            IpcMsg::CheckUpdates => {
                                let p = proxy.clone();
                                check_updates(Arc::clone(&update_state), move || {
                                    let _ = p.send_event(UserEvent::UpdateStateChanged);
                                });
                            }
                            IpcMsg::ApplyUpdate { url } => {
                                let p = proxy.clone();
                                apply_update(url, Arc::clone(&update_state), move || {
                                    let _ = p.send_event(UserEvent::UpdateStateChanged);
                                });
                            }
                            IpcMsg::OpenTranslateMenu => {
                                suggest_count = 0;
                                menu_open = true;
                            }
                            IpcMsg::TranslatePage { lang } => {
                                menu_open = false;
                                let target_lang = if lang.trim().is_empty() { "vi" } else { lang.trim() };
                                if let Some(tab) = tabs.iter_mut().find(|t| t.id == active_tab_id) {
                                    if tab.url.starts_with("http://") || tab.url.starts_with("https://") {
                                        let enc: String = tab
                                            .url
                                            .bytes()
                                            .map(|b| match b {
                                                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                                                _ => format!("%{:02X}", b),
                                            })
                                            .collect();
                                        let tr_url = format!("https://translate.google.com/translate?sl=auto&tl={}&u={}", target_lang, enc);
                                        tab.url = tr_url.clone();
                                        if let Some(wv) = &tab.webview {
                                            let _ = wv.load_url(&tr_url);
                                        }
                                        let _ = chrome.evaluate_script(&format!(
                                            "window.__vnToast && window.__vnToast({});",
                                            serde_json::to_string(&format!("🌐 Đang dịch trang sang [{}]...", target_lang)).unwrap_or_default()
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                layout_views(&window, &chrome, &mut tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, suggest_count, &data);
                push_state(&chrome, &tabs, active_tab_id, &active_panel, find_open, menu_open, incognito_locked, passwords_locked, &revealed_pw, &data, &update_state);
            }
            _ => {}
        }
    });
}
