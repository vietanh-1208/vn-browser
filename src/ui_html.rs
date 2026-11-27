pub const CHROME_HTML: &str = r##"<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="UTF-8">
<style>
  :root {
    --bg: #0b1120;
    --bg2: #151e32;
    --bg3: #24314d;
    --text: #f8fafc;
    --muted: #94a3b8;
    --accent: #22c55e;
    --accent-soft: rgba(34, 197, 94, 0.16);
    --border: rgba(148, 163, 184, 0.16);
    --danger: #ef4444;
  }
  body.light {
    --bg: #e9eef6;
    --bg2: #ffffff;
    --bg3: #dce3ee;
    --text: #0f172a;
    --muted: #64748b;
    --accent-soft: rgba(22, 163, 74, 0.12);
    --border: rgba(15, 23, 42, 0.12);
    --danger: #dc2626;
  }
  * { box-sizing: border-box; margin: 0; padding: 0; user-select: none; }
  body {
    font-family: "Segoe UI", system-ui, sans-serif;
    background: var(--bg);
    color: var(--text);
    overflow: hidden;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }
  .tabs-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px 4px 10px;
    height: 40px;
    background: var(--bg);
    overflow-x: auto;
  }
  .tabs-bar::-webkit-scrollbar { height: 0; }
  .brand-dot {
    width: 24px;
    height: 24px;
    border-radius: 8px;
    background: linear-gradient(135deg, var(--accent), #0ea5e9);
    color: #fff;
    font-weight: 800;
    font-size: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    box-shadow: 0 2px 8px var(--accent-soft);
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    min-width: 115px;
    max-width: 210px;
    padding: 0 10px;
    border-radius: 10px;
    background: transparent;
    color: var(--muted);
    font-size: 12.5px;
    cursor: pointer;
    border: 1px solid transparent;
    flex: 1 1 150px;
    transition: background 0.15s, border-color 0.15s;
  }
  .tab.pinned { flex: 0 0 auto; min-width: 44px; max-width: 115px; }
  .tab:hover { background: var(--bg3); color: var(--text); }
  .tab.active {
    background: var(--bg2);
    color: var(--text);
    border-color: var(--border);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.18);
  }
  .tab.incognito.active { background: #2e1065; color: #f5f3ff; border-color: rgba(192,132,252,0.35); }
  .tab-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tab-close {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    opacity: 0.65;
  }
  .tab-close:hover { background: rgba(239, 68, 68, 0.25); color: var(--danger); opacity: 1; }
  .spin {
    width: 13px;
    height: 13px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: vnspin 0.7s linear infinite;
    flex-shrink: 0;
  }
  @keyframes vnspin { to { transform: rotate(360deg); } }

  .icon-btn {
    border: none;
    background: transparent;
    color: var(--text);
    width: 30px;
    height: 30px;
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    flex-shrink: 0;
    transition: background 0.12s;
  }
  .icon-btn:hover { background: var(--bg3); }
  .ram-btn {
    border: 1px solid var(--border);
    background: var(--bg2);
    color: var(--accent);
    height: 28px;
    padding: 0 10px;
    border-radius: 14px;
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
    margin-left: auto;
  }
  .ram-btn:hover { background: var(--accent-soft); border-color: var(--accent); }

  .nav-bar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 44px;
    padding: 0 10px;
    background: var(--bg2);
    border-bottom: 1px solid var(--border);
  }
  .load-track {
    position: relative;
    height: 2.5px;
    width: 100%;
    background: transparent;
    overflow: hidden;
    flex-shrink: 0;
  }
  .load-bar {
    height: 100%;
    width: 0%;
    background: linear-gradient(90deg, var(--accent), #38bdf8);
    transition: width 0.25s ease, opacity 0.25s ease;
    opacity: 0;
  }
  .load-bar.loading {
    opacity: 1;
    width: 78%;
    animation: vnloadpulse 1.6s ease-in-out infinite;
  }
  .load-bar.done {
    opacity: 0;
    width: 100%;
  }
  @keyframes vnloadpulse {
    0% { filter: brightness(1); }
    50% { filter: brightness(1.35); }
    100% { filter: brightness(1); }
  }

  .omnibox {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 12px;
    background: var(--bg);
    border: 1.5px solid var(--border);
    border-radius: 18px;
    transition: border-color 0.15s, box-shadow 0.15s;
  }
  .omnibox:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }
  .omnibox input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    user-select: text;
  }
  .badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--accent-soft);
    color: var(--accent);
    cursor: pointer;
    white-space: nowrap;
    font-weight: 600;
  }
  .bookmarks-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    background: var(--bg2);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
  }
  .bookmarks-bar.hidden { display: none; }
  .bm-chip {
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 12px;
    padding: 3px 10px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
  }
  .bm-chip:hover { background: var(--bg3); }
  .find-bar {
    display: none;
    align-items: center;
    gap: 6px;
    height: 36px;
    padding: 0 12px;
    background: var(--bg2);
    border-bottom: 1px solid var(--border);
  }
  .find-bar.open { display: flex; }
  .find-bar input {
    width: 240px;
    height: 26px;
    padding: 0 10px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    outline: none;
    user-select: text;
  }

  .suggest-wrap {
    display: none;
    position: fixed;
    top: 86px;
    left: 0;
    right: 0;
    z-index: 60;
    pointer-events: none;
    padding: 6px 14px 10px 14px;
    background: transparent;
  }
  .suggest-wrap.open { display: flex; justify-content: center; }
  .suggest-box {
    pointer-events: auto;
    width: min(680px, 92%);
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 6px;
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.35);
    max-height: 270px;
    overflow-y: auto;
  }
  .suggest-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: 9px;
    cursor: pointer;
    font-size: 13px;
  }
  .suggest-item:hover, .suggest-item.sel { background: var(--accent-soft); }
  .suggest-kind { width: 20px; text-align: center; flex-shrink: 0; }
  .suggest-label { font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .suggest-sub { color: var(--muted); font-size: 11.5px; margin-left: auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 42%; }

  .content-area {
    flex: 1;
    overflow-y: auto;
    padding: 28px;
    background: radial-gradient(circle at top center, var(--accent-soft), transparent 45%), var(--bg);
    display: none;
  }
  .content-area.open { display: block; }
  .panel { max-width: 860px; margin: 0 auto; }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 18px;
  }
  .panel-header h2 { font-size: 22px; }
  .btn {
    border: none;
    padding: 8px 15px;
    border-radius: 9px;
    font-size: 13px;
    cursor: pointer;
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }
  .btn.soft { background: var(--bg3); color: var(--text); font-weight: 500; }
  .btn.danger { background: rgba(239, 68, 68, 0.18); color: var(--danger); }
  .card {
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 14px 16px;
    margin-bottom: 10px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .newtab-hero {
    max-width: 680px;
    margin: 44px auto 0 auto;
    text-align: center;
  }
  .newtab-logo {
    font-size: 40px;
    font-weight: 800;
    margin-bottom: 8px;
    letter-spacing: -0.6px;
  }
  .newtab-logo span { color: var(--accent); }
  .newtab-search {
    display: flex;
    gap: 8px;
    margin: 22px 0 28px 0;
  }
  .newtab-search input {
    flex: 1;
    height: 46px;
    border-radius: 23px;
    border: 1.5px solid var(--border);
    background: var(--bg2);
    color: var(--text);
    padding: 0 20px;
    font-size: 15px;
    outline: none;
    user-select: text;
  }
  .newtab-search input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
  .dial-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
    text-align: left;
  }
  .dial-card {
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 13px;
    padding: 14px;
    cursor: pointer;
    transition: transform 0.12s, border-color 0.12s;
  }
  .dial-card:hover { transform: translateY(-2px); border-color: var(--accent); }
  .dial-title { font-weight: 600; font-size: 13.5px; margin-bottom: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dial-url { font-size: 11.5px; color: var(--muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .search-input {
    width: 100%;
    height: 36px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg2);
    color: var(--text);
    padding: 0 12px;
    margin-bottom: 14px;
    outline: none;
    user-select: text;
  }
  .swatch-row { display: flex; gap: 8px; }
  .swatch {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    cursor: pointer;
    border: 2px solid transparent;
  }
  .swatch.active { border-color: var(--text); transform: scale(1.1); }
  .toast {
    position: fixed;
    bottom: 14px;
    right: 16px;
    background: var(--bg2);
    color: var(--text);
    border: 1px solid var(--accent);
    padding: 8px 14px;
    border-radius: 10px;
    font-size: 12.5px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.35);
    display: none;
    z-index: 99;
  }
  .toast.show { display: block; }
  .menu-wrap {
    display: none;
    position: fixed;
    top: 86px;
    left: 0;
    right: 0;
    z-index: 60;
    pointer-events: none;
    justify-content: flex-end;
    padding: 4px 12px 10px 12px;
  }
  .menu-wrap.open { display: flex; }
  .menu-box {
    pointer-events: auto;
    width: 265px;
    background: var(--bg2);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 6px;
    box-shadow: 0 14px 32px rgba(0,0,0,0.42);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 13px;
    cursor: pointer;
    color: var(--text);
  }
  .menu-item:hover { background: var(--accent-soft); }
  .menu-k { font-size: 11px; color: var(--muted); }
  .menu-sep { height: 1px; background: var(--border); margin: 4px 8px; }
  select {
    background: var(--bg);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 6px 10px;
  }
</style>
</head>
<body>
  <div class="tabs-bar" id="tabsBar"></div>
  <div class="nav-bar">
    <button class="icon-btn" title="Quay lại" onclick="send({type:'back'})"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg></button>
    <button class="icon-btn" title="Tiến tới" onclick="send({type:'forward'})"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14M12 5l7 7-7 7"/></svg></button>
    <button class="icon-btn" id="reloadBtn" title="Tải lại / Dừng (Ctrl+R)" onclick="onReloadOrStop()"><svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-2.64-6.36"/><path d="M21 3v6h-6"/></svg></button>
    <button class="icon-btn" title="Trang chủ" onclick="send({type:'home'})"><svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 10.5L12 3l9 7.5V21a1 1 0 0 1-1 1h-5v-7h-6v7H4a1 1 0 0 1-1-1z"/></svg></button>
    <div class="omnibox">
      <span id="sslBadge" title="Bảo mật">🔒</span>
      <input id="urlInput" type="text" placeholder="Nhập địa chỉ web hoặc tìm kiếm..." />
      <span class="badge" id="autofillBadge" style="display:none" title="Điền nhanh tài khoản đã lưu" onclick="send({type:'autofill_active'})">🔑 Điền TK</span>
      <span class="badge" id="adBadge" title="Chặn quảng cáo & tracker" onclick="send({type:'toggle_panel',panel:'settings'})">🛡️ 0</span>
      <button class="icon-btn" style="width:24px;height:24px" title="Lưu dấu trang (Ctrl+D)" id="bmStar" onclick="send({type:'toggle_bookmark'})">☆</button>
    </div>
    <button class="icon-btn" title="Tải xuống (Ctrl+J)" onclick="send({type:'toggle_panel',panel:'downloads'})"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg></button>
    <button class="icon-btn" title="Dấu trang" onclick="send({type:'toggle_panel',panel:'bookmarks'})"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg></button>
    <button class="icon-btn" title="Menu VN Browser" onclick="send({type:'toggle_menu'})"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><circle cx="12" cy="5" r="1.5" fill="currentColor"/><circle cx="12" cy="12" r="1.5" fill="currentColor"/><circle cx="12" cy="19" r="1.5" fill="currentColor"/></svg></button>
  </div>
  <div class="load-track"><div class="load-bar" id="loadBar"></div></div>
  <div class="bookmarks-bar" id="bookmarksBar"></div>
  <div class="find-bar" id="findBar">
    <input id="findInput" type="text" placeholder="Tìm chữ trên trang..." />
    <button class="btn soft" onclick="findInPage(false)">Trước</button>
    <button class="btn soft" onclick="findInPage(true)">Sau</button>
    <button class="icon-btn" onclick="send({type:'toggle_find'})">✕</button>
  </div>
  <div class="suggest-wrap" id="suggestWrap"><div class="suggest-box" id="suggestBox"></div></div>
  <div class="menu-wrap" id="menuWrap"><div class="menu-box" id="menuBox"></div></div>
  <div class="content-area" id="contentArea"></div>
  <div class="toast" id="toast"></div>

<script>
let STATE = null;
function send(msg) {
  if (window.ipc) window.ipc.postMessage(JSON.stringify(msg));
}

function showToast(msg) {
  const t = document.getElementById('toast');
  t.textContent = msg;
  t.classList.add('show');
  clearTimeout(window.__vnToastTimer);
  window.__vnToastTimer = setTimeout(() => t.classList.remove('show'), 2600);
}
window.__vnToast = showToast;
window.__vnPopupBlocked = function(domain, url) {
  const t = document.getElementById('toast');
  t.innerHTML = `<span>🛡️ Đã chặn tab tự động mở: <b>${esc(domain)}</b></span> <button class="btn" style="padding:3px 9px;margin-left:8px;font-size:11.5px" data-url="${esc(url)}" onclick="send({type:'open_url_new_tab',url:this.dataset.url}); document.getElementById('toast').classList.remove('show')">Vẫn mở</button>`;
  t.classList.add('show');
  clearTimeout(window.__vnToastTimer);
  window.__vnToastTimer = setTimeout(() => t.classList.remove('show'), 4500);
};
window.__vnOfferSaveCred = function(site, user) {
  const t = document.getElementById('toast');
  t.innerHTML = `<span>🔑 Lưu mật khẩu cho <b>${esc(site)}</b> (${esc(user)})?</span> <button class="btn" style="padding:3px 9px;margin-left:8px;font-size:11.5px" onclick="send({type:'accept_save_cred'}); document.getElementById('toast').classList.remove('show')">Lưu</button> <button class="btn soft" style="padding:3px 8px;margin-left:4px;font-size:11.5px" onclick="document.getElementById('toast').classList.remove('show')">Không</button>`;
  t.classList.add('show');
  clearTimeout(window.__vnToastTimer);
  window.__vnToastTimer = setTimeout(() => t.classList.remove('show'), 9000);
};
let DRAG_TAB_ID = null;

function onReloadOrStop() {
  const active = STATE && STATE.tabs.find(t => t.id === STATE.active_tab_id);
  if (active && active.loading) send({ type: 'stop_loading' });
  else send({ type: 'reload' });
}

const urlInput = document.getElementById('urlInput');
const suggestWrap = document.getElementById('suggestWrap');
const suggestBox = document.getElementById('suggestBox');
let SUGGESTS = [];
let SUGGEST_IDX = -1;
let suggestTimer = null;

function renderSuggests() {
  if (!SUGGESTS.length || document.activeElement !== urlInput) {
    suggestWrap.classList.remove('open');
    suggestBox.innerHTML = '';
    return;
  }
  suggestWrap.classList.add('open');
  suggestBox.innerHTML = SUGGESTS.map((s, i) => `
    <div class="suggest-item ${i === SUGGEST_IDX ? 'sel' : ''}" data-idx="${i}" onmousedown="event.preventDefault(); pickSuggest(${i})">
      <span class="suggest-kind">${s.kind === 'bookmark' ? '★' : (s.kind === 'history' ? '🕒' : '🔍')}</span>
      <span class="suggest-label">${esc(s.label)}</span>
      ${s.url && s.url !== s.label ? `<span class="suggest-sub">${esc(s.url)}</span>` : ''}
    </div>
  `).join('');
}

function pickSuggest(i) {
  const s = SUGGESTS[i];
  if (!s) return;
  const target = s.url || s.label;
  SUGGESTS = [];
  SUGGEST_IDX = -1;
  renderSuggests();
  send({ type: 'navigate', input: target });
  urlInput.blur();
}

window.__vnShowSuggestions = function(items) {
  SUGGESTS = items || [];
  SUGGEST_IDX = -1;
  renderSuggests();
};

urlInput.addEventListener('input', () => {
  clearTimeout(suggestTimer);
  const q = urlInput.value.trim();
  if (!q) {
    SUGGESTS = [];
    renderSuggests();
    send({ type: 'suggest', query: '' });
    return;
  }
  suggestTimer = setTimeout(() => send({ type: 'suggest', query: q }), 110);
});

urlInput.addEventListener('blur', () => {
  setTimeout(() => {
    SUGGESTS = [];
    SUGGEST_IDX = -1;
    renderSuggests();
    send({ type: 'suggest', query: '' });
  }, 120);
});

urlInput.addEventListener('keydown', (e) => {
  if (e.key === 'ArrowDown' && SUGGESTS.length) {
    e.preventDefault();
    SUGGEST_IDX = (SUGGEST_IDX + 1) % SUGGESTS.length;
    renderSuggests();
  } else if (e.key === 'ArrowUp' && SUGGESTS.length) {
    e.preventDefault();
    SUGGEST_IDX = (SUGGEST_IDX - 1 + SUGGESTS.length) % SUGGESTS.length;
    renderSuggests();
  } else if (e.key === 'Escape') {
    SUGGESTS = [];
    SUGGEST_IDX = -1;
    renderSuggests();
    send({ type: 'suggest', query: '' });
    urlInput.blur();
  } else if (e.key === 'Enter') {
    if (SUGGEST_IDX >= 0 && SUGGESTS[SUGGEST_IDX]) {
      pickSuggest(SUGGEST_IDX);
    } else {
      SUGGESTS = [];
      renderSuggests();
      send({ type: 'suggest', query: '' });
      send({ type: 'navigate', input: urlInput.value });
      urlInput.blur();
    }
  }
});
urlInput.addEventListener('focus', () => urlInput.select());

const findInput = document.getElementById('findInput');
findInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') findInPage(!e.shiftKey);
  if (e.key === 'Escape') send({ type: 'toggle_find' });
});
function findInPage(forward) {
  send({ type: 'find', query: findInput.value, forward });
}

document.addEventListener('keydown', (e) => {
  if (e.ctrlKey && e.key.toLowerCase() === 't') { e.preventDefault(); send({ type: 'new_tab', incognito: e.shiftKey }); }
  else if (e.ctrlKey && e.key.toLowerCase() === 'w') { e.preventDefault(); if (STATE) send({ type: 'close_tab', id: STATE.active_tab_id }); }
  else if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 't') { e.preventDefault(); send({ type: 'reopen_tab' }); }
  else if (e.ctrlKey && e.key.toLowerCase() === 'l') { e.preventDefault(); urlInput.focus(); urlInput.select(); }
  else if (e.ctrlKey && e.key.toLowerCase() === 'h') { e.preventDefault(); send({ type: 'toggle_panel', panel: 'history' }); }
  else if (e.ctrlKey && e.key.toLowerCase() === 'j') { e.preventDefault(); send({ type: 'toggle_panel', panel: 'downloads' }); }
  else if (e.ctrlKey && e.key.toLowerCase() === 'd') { e.preventDefault(); send({ type: 'toggle_bookmark' }); }
  else if (e.ctrlKey && e.key.toLowerCase() === 'f') { e.preventDefault(); send({ type: 'toggle_find' }); }
  else if (e.ctrlKey && e.key.toLowerCase() === 'r') { e.preventDefault(); send({ type: 'reload' }); }
});

function esc(s) {
  return String(s || '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
}

const ACCENTS = [
  { name: 'VN Emerald', hex: '#22c55e' },
  { name: 'Chrome Blue', hex: '#3b82f6' },
  { name: 'Neon Purple', hex: '#a855f7' },
  { name: 'Amber Gold', hex: '#f59e0b' },
  { name: 'Rose Pink', hex: '#ec4899' },
];

window.__vnUpdateState = function(st) {
  STATE = st;
  document.body.classList.toggle('light', !st.settings.dark_mode);
  const acc = st.settings.accent_color || '#22c55e';
  document.documentElement.style.setProperty('--accent', acc);

  const tabsBar = document.getElementById('tabsBar');
  tabsBar.innerHTML = `<div class="brand-dot" style="background:transparent;box-shadow:none" title="VN Browser"><svg width="22" height="22" viewBox="0 0 64 64"><circle cx="32" cy="32" r="29" fill="#0ea5e9"/><path d="M32 3A29 29 0 0 1 58 19H32Z" fill="#10b981"/><path d="M32 3A29 29 0 0 0 6 46L22 32Z" fill="#2563eb"/><circle cx="32" cy="32" r="15" fill="#fff"/><circle cx="32" cy="32" r="11.5" fill="#0ea5e9"/><path d="M21 32h22M32 21v22" stroke="#fff" stroke-width="2.5"/></svg></div>` + st.tabs.map(t => `
    <div class="tab ${t.id === st.active_tab_id ? 'active' : ''} ${t.pinned ? 'pinned' : ''} ${t.incognito ? 'incognito' : ''}"
         draggable="true"
         ondragstart="DRAG_TAB_ID=${t.id}"
         ondragover="event.preventDefault()"
         ondrop="if(DRAG_TAB_ID && DRAG_TAB_ID !== ${t.id}) send({type:'reorder_tab',from_id:DRAG_TAB_ID,to_id:${t.id}})"
         onclick="send({type:'switch_tab', id:${t.id}})"
         oncontextmenu="event.preventDefault(); send({type:'toggle_pin', id:${t.id}})">
      ${t.loading ? '<span class="spin"></span>' : `<span>${t.incognito ? '🕶' : (t.sleeping ? '💤' : (t.pinned ? '📌' : '🌐'))}</span>`}
      <span class="tab-title" title="${esc(t.title)}">${esc(t.title || 'Tab mới')}</span>
      ${t.url !== 'vn://newtab' ? `<span class="tab-close" title="${t.muted ? 'Bật tiếng tab' : 'Tắt tiếng tab'}" onclick="event.stopPropagation(); send({type:'toggle_mute', id:${t.id}})">${t.muted ? '🔇' : '🔊'}</span>` : ''}
      ${!t.pinned ? `<span class="tab-close" onclick="event.stopPropagation(); send({type:'close_tab', id:${t.id}})">×</span>` : ''}
    </div>
  `).join('') + `<button class="icon-btn" title="Tab mới (Ctrl+T)" onclick="send({type:'new_tab',incognito:false})">＋</button>`
    + `<button class="icon-btn" title="Mở lại tab vừa đóng (Ctrl+Shift+T)" onclick="send({type:'reopen_tab'})">↺</button>`;

  const active = st.tabs.find(t => t.id === st.active_tab_id);
  if (active && document.activeElement !== urlInput) {
    urlInput.value = active.url === 'vn://newtab' ? '' : active.url;
  }
  document.getElementById('sslBadge').textContent = active && active.url.startsWith('https://') ? '🔒' : (active && active.incognito ? '🕶' : '🌐');
  document.getElementById('autofillBadge').style.display = (active && active.has_saved_cred) ? 'inline-block' : 'none';
  document.getElementById('adBadge').textContent = (st.settings.adblock_enabled ? '🛡️ ' : '⚪ ') + (active ? active.blocked_count : 0);
  document.getElementById('reloadBtn').innerHTML = (active && active.loading)
    ? '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round"><path d="M18 6L6 18M6 6l12 12"/></svg>'
    : '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-2.64-6.36"/><path d="M21 3v6h-6"/></svg>';
  const zoomPct = Math.round((active ? active.zoom : 1.0) * 100);
  const menuWrap = document.getElementById('menuWrap');
  const menuBox = document.getElementById('menuBox');
  menuWrap.classList.toggle('open', Boolean(st.menu_open));
  if (st.menu_open) {
    menuBox.innerHTML = `
      <div class="menu-item" onclick="send({type:'new_tab',incognito:false})"><span>➕ Tab mới</span><span class="menu-k">Ctrl+T</span></div>
      <div class="menu-item" onclick="send({type:'new_tab',incognito:true})"><span>🕶 Tab Ẩn danh</span><span class="menu-k">Ctrl+Shift+T</span></div>
      <div class="menu-sep"></div>
      <div class="menu-item" onclick="send({type:'toggle_panel',panel:'history'})"><span>🕒 Lịch sử</span><span class="menu-k">Ctrl+H</span></div>
      <div class="menu-item" onclick="send({type:'toggle_panel',panel:'downloads'})"><span>⬇ Tải xuống</span><span class="menu-k">Ctrl+J</span></div>
      <div class="menu-item" onclick="send({type:'toggle_panel',panel:'bookmarks'})"><span>★ Dấu trang</span><span class="menu-k">Ctrl+D</span></div>
      <div class="menu-item" onclick="send({type:'toggle_panel',panel:'passwords'})"><span>🔑 Mật khẩu đã lưu</span></div>
      <div class="menu-sep"></div>
      <div class="menu-item" style="cursor:default">
        <span>🔍 Thu phóng</span>
        <span style="display:flex;gap:6px;align-items:center">
          <button class="btn soft" style="padding:2px 8px" onclick="send({type:'zoom',delta:-0.1})">−</button>
          <span style="min-width:38px;text-align:center;cursor:pointer" onclick="send({type:'zoom',delta:0})">${zoomPct}%</span>
          <button class="btn soft" style="padding:2px 8px" onclick="send({type:'zoom',delta:0.1})">+</button>
        </span>
      </div>
      <div class="menu-item" onclick="send({type:'toggle_find'})"><span>🔎 Tìm trong trang</span><span class="menu-k">Ctrl+F</span></div>
      <div class="menu-item" onclick="send({type:'picture_in_picture'})"><span>📺 Tách video nổi (PiP)</span></div>
      <div class="menu-item" onclick="send({type:'reader_mode'})"><span>📖 Chế độ đọc sách</span></div>
      <div class="menu-item" onclick="send({type:'print_page'})"><span>🖨 In / Lưu PDF</span><span class="menu-k">Ctrl+P</span></div>
      <div class="menu-item" onclick="send({type:'toggle_fullscreen'})"><span>⛶ Toàn màn hình</span><span class="menu-k">F11</span></div>
      <div class="menu-item" onclick="send({type:'devtools'})"><span>⚙️ Công cụ lập trình</span><span class="menu-k">F12</span></div>
      <div class="menu-sep"></div>
      <div class="menu-item" onclick="send({type:'toggle_panel',panel:'settings'})"><span>🎨 Cài đặt & Giao diện</span></div>
    `;
  }
  const loadBar = document.getElementById('loadBar');
  if (active && active.loading) {
    loadBar.className = 'load-bar loading';
  } else {
    loadBar.className = 'load-bar done';
  }

  const isBm = active && st.bookmarks.some(b => b.url === active.url);
  document.getElementById('bmStar').textContent = isBm ? '★' : '☆';

  const bmBar = document.getElementById('bookmarksBar');
  bmBar.classList.toggle('hidden', !st.settings.show_bookmarks_bar);
  bmBar.innerHTML = st.bookmarks.map(b => `<button class="bm-chip" data-url="${esc(b.url)}" onclick="send({type:'navigate',input:this.dataset.url})">★ ${esc(b.title)}</button>`).join('');

  const findBar = document.getElementById('findBar');
  findBar.classList.toggle('open', st.find_open);
  if (st.find_open) setTimeout(() => findInput.focus(), 20);

  const area = document.getElementById('contentArea');
  const showIncogLock = !st.active_panel && active && active.incognito && st.incognito_locked;
  const showPageError = !st.active_panel && !showIncogLock && active && Boolean(active.error_msg);
  const showNewTab = !st.active_panel && !showIncogLock && !showPageError && active && active.url === 'vn://newtab';
  const open = Boolean(st.active_panel) || showNewTab || showIncogLock || showPageError;
  area.classList.toggle('open', open);
  if (!open) return;

  if (showPageError) {
    area.innerHTML = `
      <div class="newtab-hero" style="margin-top:75px;max-width:520px">
        <div style="font-size:52px;margin-bottom:10px">📡</div>
        <h2 style="margin-bottom:8px">Không thể kết nối tới trang web</h2>
        <div style="color:var(--muted);font-size:13.5px;margin-bottom:20px;line-height:1.6">
          <b>${esc(active.url)}</b> từ chối kết nối hoặc không phản hồi.<br>
          Trang có thể đang bảo trì, đổi tên miền, hoặc bị nhà mạng chặn ở tầng IP/cổng kết nối.
        </div>
        <div style="display:flex;gap:10px;justify-content:center">
          <button class="btn" onclick="send({type:'reload'})">↻ Thử tải lại</button>
          <button class="btn soft" onclick="send({type:'toggle_panel',panel:'settings'})">🌐 Cấu hình Proxy</button>
          <button class="btn soft" onclick="send({type:'home'})">🏠 Về Trang mới</button>
        </div>
      </div>`;
    return;
  }

  if (showIncogLock) {
    area.innerHTML = `
      <div class="newtab-hero" style="margin-top:80px;max-width:420px">
        <div style="font-size:48px;margin-bottom:10px">🔒</div>
        <h2 style="margin-bottom:6px">Tab Ẩn danh đã được khoá</h2>
        <div style="color:var(--muted);font-size:13px;margin-bottom:18px">Nhập mã PIN để mở khoá các tab ẩn danh</div>
        <div class="newtab-search" style="margin-bottom:12px">
          <input id="pinInput" type="password" placeholder="Nhập mã PIN..." onkeydown="if(event.key==='Enter') send({type:'unlock_incognito',pin:this.value})" />
          <button class="btn" onclick="send({type:'unlock_incognito',pin:document.getElementById('pinInput').value})">Mở</button>
        </div>
      </div>`;
    setTimeout(() => { const p = document.getElementById('pinInput'); if (p) p.focus(); }, 20);
    return;
  }

  if (st.active_panel === 'history') {
    area.innerHTML = `
      <div class="panel">
        <div class="panel-header">
          <h2>🕒 Lịch sử duyệt web</h2>
          <div style="display:flex;gap:8px">
            <button class="btn danger" onclick="send({type:'clear_history'})">Xoá toàn bộ</button>
            <button class="btn soft" onclick="send({type:'toggle_panel',panel:null})">Đóng</button>
          </div>
        </div>
        <input class="search-input" placeholder="Tìm trong lịch sử..." oninput="filterList(this.value, 'hist-item')" />
        ${st.history.length ? st.history.map(h => `
          <div class="card row hist-item" data-q="${esc((h.title + ' ' + h.url).toLowerCase())}">
            <div style="overflow:hidden;cursor:pointer;flex:1" data-url="${esc(h.url)}" onclick="send({type:'navigate',input:this.dataset.url})">
              <div style="font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${esc(h.title)}</div>
              <div style="font-size:12px;color:var(--muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${esc(h.url)}</div>
            </div>
          </div>
        `).join('') : '<div class="card">Chưa có lịch sử nào.</div>'}
      </div>`;
  } else if (st.active_panel === 'bookmarks') {
    area.innerHTML = `
      <div class="panel">
        <div class="panel-header">
          <h2>★ Dấu trang (Bookmarks)</h2>
          <button class="btn soft" onclick="send({type:'toggle_panel',panel:null})">Đóng</button>
        </div>
        ${st.bookmarks.map(b => `
          <div class="card row">
            <div style="overflow:hidden;cursor:pointer;flex:1" data-url="${esc(b.url)}" onclick="send({type:'navigate',input:this.dataset.url})">
              <div style="font-weight:600">${esc(b.title)}</div>
              <div style="font-size:12px;color:var(--muted)">${esc(b.url)}</div>
            </div>
            <button class="btn danger" data-id="${esc(b.id)}" onclick="send({type:'delete_bookmark',id:this.dataset.id})">Xoá</button>
          </div>
        `).join('')}
      </div>`;
  } else if (st.active_panel === 'passwords') {
    if (st.passwords_locked) {
      area.innerHTML = `
        <div class="newtab-hero" style="margin-top:60px;max-width:460px">
          <div style="font-size:44px;margin-bottom:10px">🛡️</div>
          <h2 style="margin-bottom:6px">Mật khẩu được bảo vệ bằng mật khẩu máy</h2>
          <div style="color:var(--muted);font-size:13px;margin-bottom:18px">Xác minh bằng mật khẩu Windows, mã PIN hoặc Windows Hello để xem, sao chép và tự điền mật khẩu đã lưu.</div>
          <button class="btn" onclick="send({type:'unlock_passwords'})">🔓 Xác minh bằng mật khẩu máy</button>
        </div>`;
      return;
    }
    const rev = st.revealed_pw;
    area.innerHTML = `
      <div class="panel">
        <div class="panel-header">
          <div>
            <h2>🔑 Trình quản lý mật khẩu</h2>
            <div style="font-size:12px;color:var(--muted)">Mã hoá theo tài khoản Windows (DPAPI) · Tự động khoá khi rời mục</div>
          </div>
          <button class="btn soft" onclick="send({type:'toggle_panel',panel:null})">Đóng</button>
        </div>
        <input class="search-input" placeholder="Tìm kiếm trang web hoặc tên đăng nhập..." oninput="filterList(this.value, 'pw-item')" />
        <div class="card" style="margin-bottom:16px">
          <div style="font-weight:600;margin-bottom:10px;font-size:13.5px">+ Thêm mật khẩu mới</div>
          <div style="display:flex;gap:8px;flex-wrap:wrap">
            <input id="pwSite" class="search-input" style="flex:1;min-width:160px;margin:0" placeholder="Trang web (vd: facebook.com)" />
            <input id="pwUser" class="search-input" style="flex:1;min-width:140px;margin:0" placeholder="Tên người dùng / Email" />
            <input id="pwPass" type="password" class="search-input" style="flex:1;min-width:130px;margin:0" placeholder="Mật khẩu" />
            <button class="btn" onclick="send({type:'save_credential',site:document.getElementById('pwSite').value,username:document.getElementById('pwUser').value,password:document.getElementById('pwPass').value})">Lưu</button>
          </div>
        </div>
        ${(st.credentials && st.credentials.length) ? st.credentials.map(c => {
          const isRev = rev && rev[0] === c.id;
          return `
          <div class="card pw-item" data-q="${esc((c.site + ' ' + c.username).toLowerCase())}" style="display:flex;flex-direction:column;gap:10px">
            <div class="row">
              <div style="display:flex;align-items:center;gap:10px">
                <div style="width:32px;height:32px;border-radius:8px;background:var(--accent-soft);color:var(--accent);display:flex;align-items:center;justify-content:center;font-weight:700">${esc((c.site[0]||'?').toUpperCase())}</div>
                <div>
                  <div style="font-weight:600;font-size:14.5px">${esc(c.site)}</div>
                  <div style="font-size:12px;color:var(--muted)">Đã bảo vệ bởi DPAPI</div>
                </div>
              </div>
              <button class="btn danger" data-id="${esc(c.id)}" onclick="send({type:'delete_credential',id:this.dataset.id})">Xoá</button>
            </div>
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;padding-top:8px;border-top:1px solid var(--border)">
              <div style="background:var(--bg);padding:8px 12px;border-radius:8px;display:flex;align-items:center;justify-content:space-between;gap:8px">
                <div style="overflow:hidden"><div style="font-size:11px;color:var(--muted)">Tên người dùng</div><div style="font-size:13px;overflow:hidden;text-overflow:ellipsis">${esc(c.username)}</div></div>
                <button class="btn soft" style="padding:4px 9px;font-size:11.5px" data-u="${esc(c.username)}" onclick="send({type:'copy_text',text:this.dataset.u})">Sao chép</button>
              </div>
              <div style="background:var(--bg);padding:8px 12px;border-radius:8px;display:flex;align-items:center;justify-content:space-between;gap:8px">
                <div style="overflow:hidden"><div style="font-size:11px;color:var(--muted)">Mật khẩu</div><div style="font-size:13px;font-family:monospace">${isRev ? esc(rev[1]) : '••••••••••••'}</div></div>
                <div style="display:flex;gap:4px">
                  <button class="btn soft" style="padding:4px 8px;font-size:11.5px" data-id="${esc(c.id)}" onclick="send({type:'reveal_credential',id:this.dataset.id})">${isRev ? 'Ẩn' : '👁'}</button>
                  <button class="btn" style="padding:4px 9px;font-size:11.5px" data-id="${esc(c.id)}" onclick="send({type:'copy_credential',id:this.dataset.id})">Sao chép</button>
                </div>
              </div>
            </div>
          </div>`;
        }).join('') : '<div class="card">Chưa có mật khẩu nào được lưu.</div>'}
      </div>`;
  } else if (st.active_panel === 'downloads') {
    area.innerHTML = `
      <div class="panel">
        <div class="panel-header">
          <h2>⬇ Quản lý Tải xuống</h2>
          <div style="display:flex;gap:8px">
            <button class="btn soft" onclick="send({type:'open_downloads_dir'})">Mở thư mục tải</button>
            <button class="btn danger" onclick="send({type:'clear_downloads'})">Xoá danh sách</button>
            <button class="btn soft" onclick="send({type:'toggle_panel',panel:null})">Đóng</button>
          </div>
        </div>
        ${st.downloads.length ? st.downloads.map(d => `
          <div class="card row">
            <div style="overflow:hidden;flex:1">
              <div style="font-weight:600">${esc(d.filename)}</div>
              <div style="font-size:12px;color:var(--muted)">${esc(d.path)} · <b>${d.status === 'Completed' ? '✓ Hoàn tất' : (d.status === 'Failed' ? '✕ Lỗi' : '⬇ Đang tải...')}</b></div>
            </div>
            ${d.status === 'Completed' ? `<button class="btn" data-path="${esc(d.path)}" onclick="send({type:'open_file',path:this.dataset.path})">Mở file</button>` : ''}
          </div>
        `).join('') : '<div class="card">Chưa có tệp nào được tải xuống.</div>'}
      </div>`;
  } else if (st.active_panel === 'settings') {
    area.innerHTML = `
      <div class="panel">
        <div class="panel-header">
          <h2>☰ Cài đặt & Giao diện VN Browser</h2>
          <button class="btn soft" onclick="send({type:'toggle_panel',panel:null})">Đóng</button>
        </div>
        <div class="card row">
          <div><b>Màu nhấn chủ đạo (Phong cách riêng)</b><div style="font-size:12px;color:var(--muted)">Tuỳ chỉnh màu sắc nút, thanh tiến trình và viền</div></div>
          <div class="swatch-row">
            ${ACCENTS.map(a => `<div class="swatch ${acc===a.hex?'active':''}" style="background:${a.hex}" title="${a.name}" data-hex="${a.hex}" onclick="send({type:'set_setting',key:'accent_color',value:this.dataset.hex})"></div>`).join('')}
          </div>
        </div>
        <div class="card row">
          <div><b>Giao diện nền</b><div style="font-size:12px;color:var(--muted)">Chuyển giữa chế độ Tối và Sáng</div></div>
          <button class="btn soft" onclick="send({type:'set_setting',key:'dark_mode',value:${!st.settings.dark_mode}})">${st.settings.dark_mode ? '🌙 Đang bật Tối' : '☀️ Đang bật Sáng'}</button>
        </div>
        <div class="card row">
          <div><b>Công cụ tìm kiếm mặc định</b></div>
          <select onchange="send({type:'set_setting',key:'search_engine',value:this.value})">
            <option value="Google" ${st.settings.search_engine==='Google'?'selected':''}>Google</option>
            <option value="DuckDuckGo" ${st.settings.search_engine==='DuckDuckGo'?'selected':''}>DuckDuckGo</option>
            <option value="Bing" ${st.settings.search_engine==='Bing'?'selected':''}>Bing</option>
          </select>
        </div>
        <div class="card row">
          <div><b>Chặn quảng cáo & Tracker (AdBlock)</b><div style="font-size:12px;color:var(--muted)">Tự động chặn mạng quảng cáo và popup thừa</div></div>
          <button class="btn ${st.settings.adblock_enabled?'':'soft'}" onclick="send({type:'set_setting',key:'adblock_enabled',value:${!st.settings.adblock_enabled}})">${st.settings.adblock_enabled ? '🛡️ Đang bật' : 'Đã tắt'}</button>
        </div>
        <div class="card row">
          <div><b>Hiện thanh Dấu trang</b></div>
          <button class="btn soft" onclick="send({type:'set_setting',key:'show_bookmarks_bar',value:${!st.settings.show_bookmarks_bar}})">${st.settings.show_bookmarks_bar ? 'Đang hiện' : 'Đang ẩn'}</button>
        </div>
        <div class="card row">
          <div><b>Tự động ngủ tab nền (Tiết kiệm RAM)</b><div style="font-size:12px;color:var(--muted)">Giải phóng RAM các tab lâu không bấm tới</div></div>
          <div style="display:flex;gap:8px;align-items:center">
            ${st.settings.sleep_minutes > 0 ? `
              <select onchange="send({type:'set_setting',key:'sleep_minutes',value:Number(this.value)})">
                <option value="3" ${st.settings.sleep_minutes===3?'selected':''}>Sau 3 phút</option>
                <option value="5" ${st.settings.sleep_minutes===5?'selected':''}>Sau 5 phút</option>
                <option value="15" ${st.settings.sleep_minutes===15?'selected':''}>Sau 15 phút</option>
                <option value="30" ${st.settings.sleep_minutes===30?'selected':''}>Sau 30 phút</option>
              </select>
            ` : ''}
            <button class="btn ${st.settings.sleep_minutes > 0 ? '' : 'soft'}" onclick="send({type:'set_setting',key:'sleep_minutes',value:${st.settings.sleep_minutes > 0 ? 0 : 3}})">${st.settings.sleep_minutes > 0 ? '⚡ Đang bật' : 'Đã tắt'}</button>
          </div>
        </div>
        <div class="card row">
          <div><b>Khôi phục các tab phiên trước khi mở lại</b></div>
          <button class="btn soft" onclick="send({type:'set_setting',key:'restore_session',value:${!st.settings.restore_session}})">${st.settings.restore_session ? 'Đang bật' : 'Đã tắt'}</button>
        </div>
        <div class="card row">
          <div><b>Vượt chặn DNS nhà mạng (DoH 1.1.1.1)</b><div style="font-size:12px;color:var(--muted)">Tự động dùng Cloudflare DNS mã hoá để vào web bị nhà mạng chặn DNS (áp dụng khi mở lại app)</div></div>
          <button class="btn ${st.settings.doh_enabled?'':'soft'}" onclick="send({type:'set_setting',key:'doh_enabled',value:${!st.settings.doh_enabled}})">${st.settings.doh_enabled ? '🌐 Đang bật' : 'Đã tắt'}</button>
        </div>
        <div class="card row">
          <div><b>Proxy vượt chặn nâng cao (tuỳ chọn)</b><div style="font-size:12px;color:var(--muted)">Ví dụ: http://127.0.0.1:8080 hoặc socks5://127.0.0.1:1080 (áp dụng khi mở lại app)</div></div>
          <div style="display:flex;gap:6px">
            <input id="proxyInput" class="search-input" style="width:220px;margin:0;height:32px" placeholder="Để trống = Trực tiếp" value="${esc(st.settings.proxy_url||'')}" />
            <button class="btn soft" onclick="send({type:'set_setting',key:'proxy_url',value:document.getElementById('proxyInput').value})">Lưu</button>
          </div>
        </div>
        <div class="card row">
          <div><b>Khoá Tab Ẩn danh bằng mã PIN</b><div style="font-size:12px;color:var(--muted)">${st.settings.incognito_pin ? '🔒 Đang bật mã PIN bảo vệ Tab Ẩn danh' : 'Chưa đặt mã PIN'}</div></div>
          <div style="display:flex;gap:6px">
            <input id="setPinInput" type="password" class="search-input" style="width:140px;margin:0;height:32px" placeholder="Nhập PIN mới..." />
            <button class="btn" onclick="send({type:'set_setting',key:'incognito_pin',value:document.getElementById('setPinInput').value})">Đặt PIN</button>
            ${st.settings.incognito_pin ? `<button class="btn danger" onclick="send({type:'set_setting',key:'incognito_pin',value:''})">Gỡ PIN</button>` : ''}
          </div>
        </div>
        <div class="card row">
          <div><b>Đặt làm trình duyệt mặc định</b><div style="font-size:12px;color:var(--muted)">Mở nhanh cài đặt Default Apps của Windows để chọn VN Browser</div></div>
          <button class="btn soft" onclick="send({type:'set_default_browser'})">🌐 Đặt mặc định</button>
        </div>
        <div class="card row">
          <div><b>Cập nhật VN Browser (v${esc(st.app_version||'1.1.0')})</b><div style="font-size:12px;color:var(--muted)">${
            st.update_status && st.update_status.status === 'checking' ? 'Đang kiểm tra bản mới trên GitHub...' :
            st.update_status && st.update_status.status === 'downloading' ? 'Đang tải và tự cài bản cập nhật...' :
            st.update_status && st.update_status.status === 'up_to_date' ? '✓ Bạn đang dùng phiên bản mới nhất!' :
            st.update_status && st.update_status.status === 'available' ? '🎉 Có bản mới v' + esc(st.update_status.version) + '!' :
            st.update_status && st.update_status.status === 'error' ? esc(st.update_status.message) :
            'Kiểm tra và tự động cập nhật từ GitHub Releases'
          }</div></div>
          ${st.update_status && st.update_status.status === 'available'
            ? `<button class="btn" data-url="${esc(st.update_status.url)}" onclick="send({type:'apply_update',url:this.dataset.url})">⬇ Cập nhật ngay</button>`
            : `<button class="btn soft" onclick="send({type:'check_updates'})">🔄 Kiểm tra cập nhật</button>`}
        </div>
        <div class="card row">
          <div><b>Xoá dữ liệu duyệt web</b><div style="font-size:12px;color:var(--muted)">Xoá toàn bộ lịch sử, danh sách tải xuống và phiên tab đã lưu</div></div>
          <button class="btn danger" onclick="send({type:'clear_all_data'})">Xoá dữ liệu</button>
        </div>
      </div>`;
  } else if (showNewTab) {
    area.innerHTML = `
      <div class="newtab-hero">
        <div style="display:flex;justify-content:center;margin-bottom:14px">
          <svg width="72" height="72" viewBox="0 0 64 64" fill="none">
            <circle cx="32" cy="32" r="29" fill="#0ea5e9"/>
            <path d="M32 3A29 29 0 0 1 59 22H32Z" fill="${acc}"/>
            <path d="M32 3A29 29 0 0 0 5 44L23 32Z" fill="#2563eb"/>
            <circle cx="32" cy="32" r="16" fill="#ffffff"/>
            <circle cx="32" cy="32" r="12.5" fill="#0ea5e9"/>
            <ellipse cx="32" cy="32" rx="6.5" ry="12.5" stroke="#ffffff" stroke-width="2.2"/>
            <line x1="19.5" y1="32" x2="44.5" y2="32" stroke="#ffffff" stroke-width="2.2"/>
          </svg>
        </div>
        <div class="newtab-logo">VN <span>Browser</span> ${active.incognito ? '🕶' : ''}</div>
        <div style="color:var(--muted);font-size:13.5px">${active.incognito ? 'Chế độ Ẩn danh: Không lưu lịch sử và cookie sau khi đóng tab.' : 'Siêu nhẹ RAM · Tích hợp AdBlock & Ngủ tab nền thông minh.'}</div>
        <div class="newtab-search">
          <input id="ntSearch" type="text" placeholder="Tìm kiếm trên ${esc(st.settings.search_engine)} hoặc nhập URL..." onkeydown="if(event.key==='Enter') send({type:'navigate',input:this.value})" />
          <button class="btn" onclick="send({type:'navigate',input:document.getElementById('ntSearch').value})">Đi</button>
        </div>
        <div class="dial-grid">
          ${st.bookmarks.map(b => `
            <div class="dial-card" data-url="${esc(b.url)}" onclick="send({type:'navigate',input:this.dataset.url})">
              <div class="dial-title">★ ${esc(b.title)}</div>
              <div class="dial-url">${esc(b.url)}</div>
            </div>
          `).join('')}
        </div>
        <div style="margin-top:26px;display:flex;justify-content:center;gap:14px;flex-wrap:wrap">
          <div class="badge" style="padding:6px 14px;font-size:12px">🛡️ Đã chặn ${st.total_ads_blocked || 0} quảng cáo & pop-up</div>
          <div class="badge" style="padding:6px 14px;font-size:12px">🌐 DoH 1.1.1.1: ${st.settings.doh_enabled ? 'Đang bật' : 'Tắt'}</div>
        </div>
      </div>`;
  }
};

function filterList(q, cls) {
  const ql = q.toLowerCase();
  document.querySelectorAll('.' + cls).forEach(el => {
    el.style.display = el.dataset.q.includes(ql) ? 'flex' : 'none';
  });
}
send({ type: 'ready' });
</script>
</body>
</html>
"##;
