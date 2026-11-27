use std::sync::{Arc, Mutex};
use std::thread;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API: &str = "https://api.github.com/repos/vietanh-1208/vn-browser/releases/latest";

#[derive(Clone, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available { version: String, url: String },
    Downloading,
    Error { message: String },
}

pub fn parse_semver(s: &str) -> (u32, u32, u32) {
    let clean = s.trim().trim_start_matches('v').trim_start_matches('V');
    let mut parts = clean.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

pub fn check_updates(state: Arc<Mutex<UpdateStatus>>, on_done: impl Fn() + Send + 'static) {
    if let Ok(mut s) = state.lock() {
        *s = UpdateStatus::Checking;
    }
    thread::spawn(move || {
        let res = (|| -> Result<UpdateStatus, String> {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .map_err(|e| e.to_string())?;
            let resp = client
                .get(GITHUB_API)
                .header("User-Agent", "VN-Browser/1.1")
                .header("Accept", "application/vnd.github+json")
                .send()
                .map_err(|_| "Không kết nối được GitHub.".to_string())?;
            if !resp.status().is_success() {
                return Err(format!("GitHub trả về mã {}", resp.status()));
            }
            let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
            let tag = json["tag_name"]
                .as_str()
                .unwrap_or("")
                .trim_start_matches('v')
                .to_string();
            let dl_url = json["assets"]
                .as_array()
                .and_then(|arr| {
                    arr.iter()
                        .find(|a| {
                            a["name"]
                                .as_str()
                                .unwrap_or("")
                                .to_lowercase()
                                .ends_with(".exe")
                        })
                        .and_then(|a| a["browser_download_url"].as_str())
                })
                .unwrap_or("")
                .to_string();
            if !tag.is_empty() && !dl_url.is_empty() && parse_semver(&tag) > parse_semver(APP_VERSION) {
                Ok(UpdateStatus::Available { version: tag, url: dl_url })
            } else {
                Ok(UpdateStatus::UpToDate)
            }
        })();
        if let Ok(mut s) = state.lock() {
            *s = match res {
                Ok(st) => st,
                Err(message) => UpdateStatus::Error { message },
            };
        }
        on_done();
    });
}

pub fn apply_update(url: String, state: Arc<Mutex<UpdateStatus>>, on_done: impl Fn() + Send + 'static) {
    #[cfg(not(windows))]
    {
        let _ = url;
        let _ = std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" })
            .arg("https://github.com/vietanh-1208/vn-browser/releases/latest")
            .spawn();
        if let Ok(mut s) = state.lock() {
            *s = UpdateStatus::Idle;
        }
        on_done();
        return;
    }
    #[cfg(windows)]
    {
        if let Ok(mut s) = state.lock() {
            *s = UpdateStatus::Downloading;
        }
        on_done();
        thread::spawn(move || {
            let res = (|| -> Result<(), String> {
                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(120))
                    .build()
                    .map_err(|e| e.to_string())?;
                let mut resp = client
                    .get(&url)
                    .header("User-Agent", "VN-Browser/1.1")
                    .send()
                    .map_err(|e| e.to_string())?;
                if !resp.status().is_success() {
                    return Err(format!("HTTP {}", resp.status()));
                }
                let tmp = std::env::temp_dir().join("vn-browser-update.exe");
                let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
                std::io::copy(&mut resp, &mut f).map_err(|e| e.to_string())?;
                drop(f);
                let dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                std::process::Command::new(&tmp)
                    .arg("/silent")
                    .arg(dir.to_string_lossy().to_string())
                    .spawn()
                    .map_err(|e| e.to_string())?;
                std::process::exit(0);
            })();
            if let Err(message) = res {
                if let Ok(mut s) = state.lock() {
                    *s = UpdateStatus::Error { message };
                }
                on_done();
            }
        });
    }
}
