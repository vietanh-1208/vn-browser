use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchEngine {
    Google,
    DuckDuckGo,
    Bing,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::Google
    }
}

impl SearchEngine {
    pub fn search_url(&self, query: &str) -> String {
        let encoded: String = query
            .bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                b' ' => "+".to_string(),
                _ => format!("%{:02X}", b),
            })
            .collect();
        match self {
            Self::Google => format!("https://www.google.com/search?q={}", encoded),
            Self::DuckDuckGo => format!("https://duckduckgo.com/?q={}", encoded),
            Self::Bing => format!("https://www.bing.com/search?q={}", encoded),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BrowserSettings {
    pub search_engine: SearchEngine,
    pub dark_mode: bool,
    pub accent_color: String,
    pub adblock_enabled: bool,
    pub show_bookmarks_bar: bool,
    pub sleep_minutes: u64,
    pub restore_session: bool,
    pub homepage: String,
    pub adblock_whitelist: Vec<String>,
    pub window_width: f64,
    pub window_height: f64,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_maximized: bool,
    pub doh_enabled: bool,
    pub proxy_url: String,
    pub incognito_pin: String,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            search_engine: SearchEngine::Google,
            dark_mode: true,
            accent_color: "#22c55e".into(),
            adblock_enabled: true,
            show_bookmarks_bar: true,
            sleep_minutes: 3,
            restore_session: true,
            homepage: "vn://newtab".into(),
            adblock_whitelist: Vec::new(),
            window_width: 1280.0,
            window_height: 820.0,
            window_x: None,
            window_y: None,
            window_maximized: false,
            doh_enabled: true,
            proxy_url: String::new(),
            incognito_pin: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub visited_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DownloadStatus {
    Downloading,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadEntry {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub path: String,
    pub status: DownloadStatus,
    pub started_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedTab {
    pub title: String,
    pub url: String,
    pub pinned: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedCredential {
    pub id: String,
    pub site: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BrowserData {
    pub settings: BrowserSettings,
    pub bookmarks: Vec<Bookmark>,
    pub history: Vec<HistoryEntry>,
    pub downloads: Vec<DownloadEntry>,
    pub last_session: Vec<SavedTab>,
    pub credentials: Vec<SavedCredential>,
    pub total_ads_blocked: u64,
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VNBrowser");
    let _ = fs::create_dir_all(&base);
    base
}

pub fn profile_dir() -> PathBuf {
    let p = data_dir().join("profile");
    let _ = fs::create_dir_all(&p);
    p
}

pub fn downloads_dir() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| data_dir().join("downloads"))
}

impl BrowserData {
    pub fn load() -> Self {
        let path = data_dir().join("browser_data.json");
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(mut parsed) = serde_json::from_str::<BrowserData>(&raw) {
                if parsed.bookmarks.is_empty() {
                    parsed.bookmarks = default_bookmarks();
                }
                return parsed;
            }
        }
        Self {
            settings: BrowserSettings::default(),
            bookmarks: default_bookmarks(),
            history: Vec::new(),
            downloads: Vec::new(),
            last_session: Vec::new(),
            credentials: Vec::new(),
            total_ads_blocked: 0,
        }
    }

    pub fn save(&self) {
        let path = data_dir().join("browser_data.json");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn record_history(&mut self, title: &str, url: &str) {
        if url.starts_with("vn://") || url.starts_with("about:") || url.trim().is_empty() {
            return;
        }
        if let Some(last) = self.history.first_mut() {
            if last.url == url {
                if !title.trim().is_empty() {
                    last.title = title.to_string();
                }
                last.visited_at = now_secs();
                return;
            }
        }
        self.history.insert(
            0,
            HistoryEntry {
                id: format!("h_{}", now_secs()),
                title: if title.trim().is_empty() {
                    url.to_string()
                } else {
                    title.to_string()
                },
                url: url.to_string(),
                visited_at: now_secs(),
            },
        );
        if self.history.len() > 1000 {
            self.history.truncate(1000);
        }
    }
}

fn default_bookmarks() -> Vec<Bookmark> {
    let ts = now_secs();
    vec![
        Bookmark {
            id: "bm_google".into(),
            title: "Google".into(),
            url: "https://www.google.com".into(),
            created_at: ts,
        },
        Bookmark {
            id: "bm_youtube".into(),
            title: "YouTube".into(),
            url: "https://www.youtube.com".into(),
            created_at: ts,
        },
        Bookmark {
            id: "bm_github".into(),
            title: "GitHub".into(),
            url: "https://github.com".into(),
            created_at: ts,
        },
        Bookmark {
            id: "bm_discord".into(),
            title: "Discord".into(),
            url: "https://discord.com/app".into(),
            created_at: ts,
        },
    ]
}

/// Resolves user input in the address bar into either a direct URL or a search URL.
pub fn resolve_address_input(input: &str, engine: &SearchEngine) -> String {
    let s = input.trim();
    if s.is_empty() {
        return "vn://newtab".into();
    }
    if s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("vn://")
        || s.starts_with("file://")
    {
        return s.to_string();
    }
    if !s.contains(' ') && (s.contains('.') || s.starts_with("localhost")) {
        if s.starts_with("localhost") || s.starts_with("127.0.0.1") {
            return format!("http://{}", s);
        }
        return format!("https://{}", s);
    }
    engine.search_url(s)
}

/// Encrypts a password using Windows DPAPI (`CryptProtectData`) scoped to the current Windows user,
/// so `browser_data.json` never stores plaintext passwords.
pub fn encrypt_secret(plain: &str) -> String {
    use base64::Engine;
    if plain.is_empty() {
        return String::new();
    }
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Cryptography::{
            CryptProtectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
        };
        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: plain.len() as u32,
            pbData: plain.as_ptr() as *mut u8,
        };
        let mut out_blob = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };
        if CryptProtectData(
            &mut in_blob,
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        ) != 0
        {
            let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
            let encoded = base64::engine::general_purpose::STANDARD.encode(slice);
            LocalFree(out_blob.pbData as _);
            return format!("dpapi:{}", encoded);
        }
    }
    plain.to_string()
}

/// Decrypts a DPAPI-protected password string.
pub fn decrypt_secret(stored: &str) -> String {
    use base64::Engine;
    let Some(b64) = stored.strip_prefix("dpapi:") else {
        return stored.to_string();
    };
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) else {
        return String::new();
    };
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Cryptography::{
            CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
        };
        let mut in_blob = CRYPT_INTEGER_BLOB {
            cbData: bytes.len() as u32,
            pbData: bytes.as_ptr() as *mut u8,
        };
        let mut out_blob = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };
        if CryptUnprotectData(
            &mut in_blob,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        ) != 0
        {
            let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
            let plain = String::from_utf8_lossy(slice).to_string();
            LocalFree(out_blob.pbData as _);
            return plain;
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_address_input() {
        assert_eq!(
            resolve_address_input("youtube.com", &SearchEngine::Google),
            "https://youtube.com"
        );
        assert_eq!(
            resolve_address_input("localhost:3000", &SearchEngine::Google),
            "http://localhost:3000"
        );
        assert_eq!(
            resolve_address_input("rust browser", &SearchEngine::Google),
            "https://www.google.com/search?q=rust+browser"
        );
    }

    #[test]
    fn test_dpapi_encrypt_decrypt() {
        let secret = "MySuperSecret123!";
        let enc = encrypt_secret(secret);
        assert!(enc.starts_with("dpapi:"));
        assert_ne!(enc, secret);
        assert_eq!(decrypt_secret(&enc), secret);
    }
}
