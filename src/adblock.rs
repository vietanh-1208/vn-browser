const AD_TRACKER_DOMAINS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "adservice.google.com",
    "pagead2.googlesyndication.com",
    "tpc.googlesyndication.com",
    "adnxs.com",
    "taboola.com",
    "outbrain.com",
    "popads.net",
    "popcash.net",
    "propellerads.com",
    "scorecardresearch.com",
    "quantserve.com",
    "criteo.com",
    "pubmatic.com",
    "rubiconproject.com",
    "openx.net",
    "moatads.com",
    "ads-twitter.com",
    "analytics.tiktok.com",
    "hotjar.com",
    "mixpanel.com",
    "zedo.com",
    "mgid.com",
    "revcontent.com",
    "fly88",
    "ok9",
    "123b",
    "ee88",
    "88m",
    "88i",
    "qq88",
    "xx8",
    "uu88",
    "rr88",
    "u888",
    "kubet",
    "jun88",
    "hi88",
    "f8bet",
    "shbet",
    "new88",
    "mb66",
    "789bet",
    "78win",
    "go88",
    "sunwin",
    "hitclub",
    "rikvip",
    "s666",
    "qh88",
    "mu88",
    "fb88",
    "w88",
    "m88",
    "vn88",
    "bk8",
    "sv388",
    "ae888",
    "bong88",
    "sbobet",
];

pub fn extract_host(url: &str) -> String {
    let without_scheme = url.split("://").nth(1).unwrap_or(url);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

pub fn root_domain(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        host.to_string()
    }
}

/// Distinguishes forced cross-domain popup/popunder ad tabs from legitimate same-site or search-engine tabs.
pub fn is_auto_popup_blocked(src_url: &str, new_url: &str, enabled: bool) -> bool {
    if !enabled {
        return false;
    }
    let src_host = extract_host(src_url);
    let new_host = extract_host(new_url);
    if src_host.is_empty() || new_host.is_empty() {
        return false;
    }
    // Allow same root domain (e.g. youtube.com -> studio.youtube.com)
    if root_domain(&src_host) == root_domain(&new_host) {
        return false;
    }
    // Allow search engines & developer/social hubs where external links are intentional
    let trusted_sources = ["google.", "bing.com", "duckduckgo.com", "github.com", "discord.com"];
    if trusted_sources.iter().any(|t| src_host.contains(t)) {
        return false;
    }
    true
}

pub fn is_blocked_url(url: &str, page_host: &str, enabled: bool, whitelist: &[String]) -> bool {
    if !enabled {
        return false;
    }
    if page_host.contains("youtube.com") || page_host.contains("youtu.be") {
        return false;
    }
    if whitelist
        .iter()
        .any(|w| page_host == w || page_host.ends_with(&format!(".{}", w)))
    {
        return false;
    }
    let lower = url.to_lowercase();
    if lower.contains("affiliatecode=") || lower.contains("aff_id=") || lower.contains("/popunder") {
        return true;
    }
    let host = extract_host(url);
    if host.is_empty() {
        return false;
    }
    AD_TRACKER_DOMAINS
        .iter()
        .any(|d| host == *d || host.ends_with(&format!(".{}", d)) || host.contains(d))
}

pub const ADBLOCK_COSMETIC_SCRIPT: &str = r#"
(() => {
  if (window.__vnAdblockInjected) return;
  window.__vnAdblockInjected = true;
  if (location.hostname.includes('youtube.com') || location.hostname.includes('youtu.be')) return;
  const report = () => {
    try { if (window.ipc) window.ipc.postMessage(JSON.stringify({ type: 'ad_blocked_js' })); } catch (_) {}
  };

  const AD_RE = /(doubleclick\.net|googlesyndication\.com|googleadservices\.com|adservice\.google|pagead|taboola|outbrain|popads|popcash|propellerads|adnxs|criteo|mgid|revcontent|affiliatecode=|fly88|ok9|123b|ee88|88m|88i|qq88|xx8|uu88|rr88|u888|kubet|jun88|hi88|f8bet|shbet|new88|mb66|789bet|78win|go88|sunwin|hitclub|rikvip|s666|qh88|mu88|fb88|w88|m88|vn88|bk8|sv388|ae888|bong88|sbobet)/i;

  // 1. Block ad/tracker network calls via fetch & XMLHttpRequest
  const origFetch = window.fetch;
  if (origFetch) {
    window.fetch = function(input) {
      const url = typeof input === 'string' ? input : (input && input.url) || '';
      if (AD_RE.test(url)) {
        report();
        return Promise.resolve(new Response('', { status: 204 }));
      }
      return origFetch.apply(this, arguments);
    };
  }
  const origOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function(method, url) {
    this.__vnUrl = String(url || '');
    return origOpen.apply(this, arguments);
  };
  const origSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.send = function() {
    if (this.__vnUrl && AD_RE.test(this.__vnUrl)) {
      report();
      return;
    }
    return origSend.apply(this, arguments);
  };

  // 2. Block unrequested popups & click-hijack redirects
  const origWinOpen = window.open;
  window.open = function(url) {
    if (!url || AD_RE.test(String(url))) {
      report();
      return null;
    }
    return origWinOpen.apply(this, arguments);
  };
  document.addEventListener('click', (e) => {
    const a = e.target && e.target.closest ? e.target.closest('a[href]') : null;
    if (a && AD_RE.test(a.href)) {
      e.preventDefault();
      e.stopImmediatePropagation();
      report();
    }
  }, true);

  // 3. Cosmetic CSS rules (General Web + Movie Popups/Banners)
  const injectCss = () => {
    if (document.getElementById('__vnAdStyle')) return;
    const style = document.createElement('style');
    style.id = '__vnAdStyle';
    style.textContent = `
      ins.adsbygoogle,
      [id^="google_ads_iframe"],
      [id*="taboola-"],
      [class*="outbrain"],
      [id*="catfish"], [class*="catfish"],
      [id*="ads-banner"], [class*="ads-banner"],
      [id*="banner-ads"], [class*="banner-ads"],
      [id*="popup-ads"], [class*="popup-ads"],
      [class*="ad-overlay"], [id*="ad-overlay"],
      a[href*="affiliateCode="],
      a[href*="fly88"], a[href*="ok9"], a[href*="123b"], a[href*="ee88"],
      a[href*="88m"], a[href*="88i"], a[href*="qq88"], a[href*="xx8"],
      a[href*="kubet"], a[href*="jun88"], a[href*="hi88"], a[href*="f8bet"],
      a[href*="shbet"], a[href*="new88"], a[href*="mb66"], a[href*="789bet"],
      a[href*="78win"], a[href*="go88"], a[href*="sunwin"], a[href*="hitclub"] {
        display: none !important;
        visibility: hidden !important;
        pointer-events: none !important;
      }
    `;
    (document.head || document.documentElement).appendChild(style);
  };
  injectCss();

  // 4. Floating Banner Sweeper
  const sweep = () => {
    injectCss();
    document.querySelectorAll('a[href]').forEach(a => {
      if (AD_RE.test(a.href)) {
        const box = a.closest('div[style*="fixed"], div[style*="z-index"], .modal, [id*="popup"], [class*="popup"], [class*="banner"], [id*="banner"]') || a;
        if (box && box.style.display !== 'none') {
          box.style.setProperty('display', 'none', 'important');
          report();
        }
      }
    });
  };
  setInterval(sweep, 200);
  if (document.documentElement) {
    new MutationObserver(sweep).observe(document.documentElement, { childList: true, subtree: true });
  }
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adblock_rules() {
        assert!(is_blocked_url(
            "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js",
            "example.com",
            true,
            &[]
        ));
        assert!(is_blocked_url(
            "https://www.qq88abj.com/register?affiliateCode=z15dl",
            "motchillm.net",
            true,
            &[]
        ));
        assert!(!is_blocked_url(
            "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js",
            "example.com",
            false,
            &[]
        ));
        assert!(!is_blocked_url(
            "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js",
            "example.com",
            true,
            &["example.com".to_string()]
        ));
        assert!(!is_blocked_url(
            "https://www.youtube.com/watch?v=123",
            "youtube.com",
            true,
            &[]
        ));
        assert!(is_auto_popup_blocked(
            "https://motchillm.net/phim/123",
            "https://random-ad-xyz.com/promo",
            true
        ));
        assert!(!is_auto_popup_blocked(
            "https://www.google.com/search?q=github",
            "https://github.com/vietanh-1208",
            true
        ));
        assert!(!is_auto_popup_blocked(
            "https://www.youtube.com/watch?v=1",
            "https://studio.youtube.com/",
            true
        ));
    }
}
