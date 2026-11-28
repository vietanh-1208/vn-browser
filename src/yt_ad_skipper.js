(function () {
  'use strict';

  if (window.__vnYtAdSkipperV52) return;
  const host = (location && location.hostname ? location.hostname : '').toLowerCase();
  if (host !== 'youtube.com' && !host.endsWith('.youtube.com')) return;
  window.__vnYtAdSkipperV52 = true;

  // =========================================================================
  // YOUTUBE ADBLOCK PRO & SPEED CONTROLLER v5.2 (UNIFIED PC + MOBILE ENGINE)
  // 1. Proxy event.isTrusted === true ngay từ document_start (trước base.js)
  // 2. Tự động tắt tiếng, tua 16x & bấm bỏ qua Pre-roll / Mid-roll / Endcap
  // 3. Ẩn sạch quảng cáo trang chủ, cột bên, cảnh báo chặn QC (ytd-* & ytm-*)
  // 4. Thanh điều khiển tốc độ 0x - 16x (TrustedHTML-safe, hỗ trợ chuột & cảm ứng)
  // =========================================================================

  let lastReportedAdCycle = 0;
  function reportAdSkipped() {
    const now = Date.now();
    if (now - lastReportedAdCycle < 1200) return;
    lastReportedAdCycle = now;
    try {
      if (window.ipc && typeof window.ipc.postMessage === 'function') {
        window.ipc.postMessage(JSON.stringify({ type: 'ad_blocked_js' }));
      }
    } catch (e) {}
    try {
      if (window.VNBridge && typeof window.VNBridge.onAdSkipped === 'function') {
        window.VNBridge.onAdSkipped();
      }
    } catch (e) {}
  }

  const SYNTHETIC_TRUSTED_MARK = Symbol('yt_trusted_click');
  const INTERCEPTED_EVENT_TYPES = new Set([
    'pointerdown',
    'mousedown',
    'pointerup',
    'mouseup',
    'click',
    'touchstart',
    'touchend'
  ]);

  function createTrustedEventProxy(nativeEvent) {
    return new Proxy(
      {},
      {
        get(_, prop) {
          if (prop === 'isTrusted') return true;
          const val = nativeEvent[prop];
          return typeof val === 'function' ? val.bind(nativeEvent) : val;
        },
        set(_, prop, value) {
          try {
            nativeEvent[prop] = value;
          } catch (e) {}
          return true;
        },
        has(_, prop) {
          if (prop === 'isTrusted') return true;
          return prop in nativeEvent;
        },
        getPrototypeOf() {
          return Object.getPrototypeOf(nativeEvent);
        },
        getOwnPropertyDescriptor(_, prop) {
          if (prop === 'isTrusted') {
            return { value: true, writable: false, enumerable: true, configurable: true };
          }
          const desc = Object.getOwnPropertyDescriptor(nativeEvent, prop);
          return desc ? { ...desc, configurable: true } : undefined;
        }
      }
    );
  }

  (function installTrustedClickInterceptor() {
    try {
      const origAdd = EventTarget.prototype.addEventListener;
      const origRemove = EventTarget.prototype.removeEventListener;
      const listenerMap = new WeakMap();

      const wrappedAdd = function (type, listener, options) {
        if (listener && INTERCEPTED_EVENT_TYPES.has(type)) {
          let wrapped = listenerMap.get(listener);
          if (!wrapped) {
            if (typeof listener === 'function') {
              wrapped = function (event) {
                if (event && event[SYNTHETIC_TRUSTED_MARK]) {
                  return listener.call(this, createTrustedEventProxy(event));
                }
                return listener.call(this, event);
              };
            } else if (typeof listener === 'object' && typeof listener.handleEvent === 'function') {
              wrapped = {
                handleEvent(event) {
                  if (event && event[SYNTHETIC_TRUSTED_MARK]) {
                    return listener.handleEvent(createTrustedEventProxy(event));
                  }
                  return listener.handleEvent(event);
                }
              };
            }
            if (wrapped) {
              listenerMap.set(listener, wrapped);
            }
          }
          if (wrapped) {
            return origAdd.call(this, type, wrapped, options);
          }
        }
        return origAdd.call(this, type, listener, options);
      };

      const wrappedRemove = function (type, listener, options) {
        if (listener && INTERCEPTED_EVENT_TYPES.has(type)) {
          const wrapped = listenerMap.get(listener);
          if (wrapped) {
            return origRemove.call(this, type, wrapped, options);
          }
        }
        return origRemove.call(this, type, listener, options);
      };

      Object.defineProperty(wrappedAdd, 'toString', {
        value: () => 'function addEventListener() { [native code] }'
      });
      Object.defineProperty(wrappedRemove, 'toString', {
        value: () => 'function removeEventListener() { [native code] }'
      });

      EventTarget.prototype.addEventListener = wrappedAdd;
      EventTarget.prototype.removeEventListener = wrappedRemove;
    } catch (e) {}
  })();

  // =========================================================================
  // CSS GIAO DIỆN & ẨN BANNER QUẢNG CÁO (DESKTOP + MOBILE)
  // =========================================================================
  const YT_STYLE_CSS = `
    html:not([data-yt-banners-disabled="true"]) ytd-ad-slot-renderer,
    html:not([data-yt-banners-disabled="true"]) ytd-in-feed-ad-layout-renderer,
    html:not([data-yt-banners-disabled="true"]) ytd-banner-promo-renderer,
    html:not([data-yt-banners-disabled="true"]) ytd-statement-banner-renderer,
    html:not([data-yt-banners-disabled="true"]) #masthead-ad,
    html:not([data-yt-banners-disabled="true"]) #player-ads,
    html:not([data-yt-banners-disabled="true"]) .ytd-merch-shelf-renderer,
    html:not([data-yt-banners-disabled="true"]) .ytp-ad-overlay-container,
    html:not([data-yt-banners-disabled="true"]) .ytp-ad-message-container,
    html:not([data-yt-banners-disabled="true"]) ytd-action-companion-ad-renderer,
    html:not([data-yt-banners-disabled="true"]) ytd-display-ad-renderer,
    html:not([data-yt-banners-disabled="true"]) ytd-promoted-sparkles-web-renderer,
    html:not([data-yt-banners-disabled="true"]) ytd-promoted-video-renderer,
    html:not([data-yt-banners-disabled="true"]) ytd-compact-promoted-video-renderer,
    html:not([data-yt-banners-disabled="true"]) #fulfilled-layout:has(ytd-ad-slot-renderer),
    html:not([data-yt-banners-disabled="true"]) ytd-rich-item-renderer:has(ytd-ad-slot-renderer),
    html:not([data-yt-banners-disabled="true"]) ytm-promoted-sparkles-web-renderer,
    html:not([data-yt-banners-disabled="true"]) ytm-promoted-video-renderer,
    html:not([data-yt-banners-disabled="true"]) ytm-companion-ad-renderer,
    html:not([data-yt-banners-disabled="true"]) ytm-statement-banner-renderer,
    html:not([data-yt-banners-disabled="true"]) ytm-in-feed-ad-layout-renderer,
    html:not([data-yt-banners-disabled="true"]) ytm-ad-slot-renderer,
    html:not([data-yt-banners-disabled="true"]) .ad-slot-renderer {
      display: none !important;
    }

    tp-yt-paper-dialog:has(ytd-enforcement-message-view-model),
    ytd-enforcement-message-view-model {
      display: none !important;
    }

    #yt-topright-speedbar {
      position: absolute !important;
      top: 14px !important;
      right: 14px !important;
      z-index: 2147483647 !important;
      display: flex !important;
      align-items: center !important;
      gap: 8px !important;
      background: rgba(15, 15, 15, 0.85) !important;
      backdrop-filter: blur(10px) !important;
      -webkit-backdrop-filter: blur(10px) !important;
      border: 1px solid rgba(0, 255, 204, 0.35) !important;
      border-radius: 30px !important;
      padding: 5px 12px !important;
      color: #ffffff !important;
      font-family: Roboto, Arial, sans-serif !important;
      font-size: 13px !important;
      font-weight: 700 !important;
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.65) !important;
      pointer-events: auto !important;
      user-select: none !important;
      opacity: 0.92;
      transform: translateY(0) !important;
      transition: opacity 0.25s cubic-bezier(0, 0, 0.2, 1), transform 0.25s cubic-bezier(0, 0, 0.2, 1) !important;
    }

    .ytp-autohide #yt-topright-speedbar,
    .ytp-autohide #yt-speed-panel,
    #yt-topright-speedbar.vn-yt-mobile-hidden {
      opacity: 0 !important;
      pointer-events: none !important;
      transform: translateY(-8px) !important;
    }

    #yt-topright-speedbar:hover {
      opacity: 1 !important;
      background: rgba(10, 10, 10, 0.96) !important;
      border-color: #00ffcc !important;
      box-shadow: 0 6px 20px rgba(0, 255, 204, 0.35) !important;
      pointer-events: auto !important;
      transform: translateY(0) !important;
    }

    #yt-speed-panel:hover {
      opacity: 1 !important;
      pointer-events: auto !important;
      transform: translateY(0) !important;
    }

    .yt-speed-btn-step {
      background: rgba(255, 255, 255, 0.14) !important;
      border: 1px solid rgba(255, 255, 255, 0.25) !important;
      color: #ffffff !important;
      width: 24px !important;
      height: 24px !important;
      border-radius: 50% !important;
      font-size: 15px !important;
      font-weight: bold !important;
      display: flex !important;
      align-items: center !important;
      justify-content: center !important;
      cursor: pointer !important;
      transition: all 0.15s ease !important;
      line-height: 1 !important;
      padding: 0 !important;
    }

    .yt-speed-btn-step:hover {
      background: #ff0000 !important;
      color: #fff !important;
      border-color: #ff0000 !important;
      transform: scale(1.1) !important;
    }

    #yt-speed-display-text {
      color: #00ffcc !important;
      font-size: 13px !important;
      font-weight: 800 !important;
      min-width: 44px !important;
      text-align: center !important;
      cursor: pointer !important;
      transition: color 0.15s ease !important;
    }

    #yt-top-slider {
      width: 85px !important;
      height: 5px !important;
      accent-color: #00ffcc !important;
      cursor: pointer !important;
      border-radius: 3px !important;
      background: #444 !important;
      outline: none !important;
      margin: 0 !important;
    }

    #yt-speed-btn-menu {
      background: transparent !important;
      border: none !important;
      color: #ccc !important;
      font-size: 15px !important;
      cursor: pointer !important;
      padding: 2px 4px !important;
      display: flex !important;
      align-items: center !important;
      justify-content: center !important;
    }

    #yt-speed-panel {
      position: absolute !important;
      top: 52px !important;
      right: 14px !important;
      z-index: 2147483647 !important;
      background: rgba(18, 18, 18, 0.97) !important;
      backdrop-filter: blur(12px) !important;
      color: #fff !important;
      padding: 14px 16px !important;
      border-radius: 12px !important;
      font-family: Roboto, Arial, sans-serif !important;
      font-size: 13px !important;
      box-shadow: 0 8px 30px rgba(0, 0, 0, 0.85) !important;
      border: 1px solid #444 !important;
      display: flex;
      flex-direction: column !important;
      gap: 10px !important;
      width: 245px !important;
      pointer-events: auto !important;
    }

    .yt-speed-row {
      display: flex !important;
      align-items: center !important;
      justify-content: space-between !important;
    }

    .yt-speed-row input[type="number"] {
      width: 80px !important;
      background: #2b2b2b !important;
      border: 1px solid #555 !important;
      color: #00ffcc !important;
      padding: 5px 8px !important;
      border-radius: 6px !important;
      font-size: 14px !important;
      font-weight: 700 !important;
      text-align: center !important;
      outline: none !important;
    }

    .yt-speed-row input[type="range"] {
      width: 100% !important;
      height: 5px !important;
      accent-color: #ff0000 !important;
      cursor: pointer !important;
    }

    .yt-speed-presets {
      display: grid !important;
      grid-template-columns: repeat(4, 1fr) !important;
      gap: 6px !important;
    }

    .yt-speed-preset-btn {
      background: #2a2a2a !important;
      border: 1px solid #3e3e3e !important;
      color: #eee !important;
      padding: 6px 2px !important;
      font-size: 11px !important;
      font-weight: 600 !important;
      border-radius: 6px !important;
      cursor: pointer !important;
      text-align: center !important;
    }

    .yt-speed-preset-btn:hover {
      background: #ff0000 !important;
      color: #fff !important;
      border-color: #ff0000 !important;
    }

    .yt-speed-preset-btn.btn-16 {
      grid-column: span 4 !important;
      background: #330000 !important;
      border-color: #ff2222 !important;
      color: #ff6666 !important;
      padding: 7px 0 !important;
      font-weight: 700 !important;
    }

    #yt-speed-toast {
      position: absolute !important;
      top: 50% !important;
      left: 50% !important;
      transform: translate(-50%, -50%) !important;
      background: rgba(0, 0, 0, 0.85) !important;
      backdrop-filter: blur(6px);
      color: #00ffcc !important;
      padding: 10px 24px !important;
      border-radius: 30px !important;
      font-size: 20px !important;
      font-weight: 800 !important;
      font-family: Roboto, Arial, sans-serif !important;
      z-index: 2147483647 !important;
      pointer-events: none !important;
      opacity: 0;
      transition: opacity 0.2s ease !important;
      box-shadow: 0 4px 20px rgba(0, 0, 0, 0.7) !important;
      border: 1px solid rgba(0, 255, 204, 0.3) !important;
    }

    @media (max-width: 680px) {
      #yt-topright-speedbar {
        top: 8px !important;
        left: 8px !important;
        right: auto !important;
        padding: 3px 9px !important;
        gap: 5px !important;
        font-size: 12px !important;
      }
      #yt-top-slider {
        display: none !important;
      }
      #yt-speed-panel {
        top: 42px !important;
        left: 8px !important;
        right: auto !important;
        width: 220px !important;
        padding: 10px 12px !important;
      }
    }
  `;

  function ensureStyleInjected() {
    if (document.getElementById('__vnYtSkipperCss')) return;
    const parent = document.head || document.documentElement;
    if (!parent) return;
    const style = document.createElement('style');
    style.id = '__vnYtSkipperCss';
    style.textContent = YT_STYLE_CSS;
    parent.appendChild(style);
  }
  ensureStyleInjected();

  // =========================================================================
  // BỘ XỬ LÝ TUA & BỎ QUA QUẢNG CÁO VIDEO (PRE-ROLL / MID-ROLL / ENDCAP)
  // =========================================================================
  let wasMutedByAd = false;
  let lastSeekedAdKey = '';
  let lastSeekedAt = 0;

  let skipClickAttempts = 0;
  let firstSkipClickAt = 0;
  let lastSkipClickTime = 0;

  let endcapRecoverAttempts = 0;
  let lastEndcapRecoverTime = 0;
  let lastReloadedVideoId = '';

  let lastMainVideoTime = 0;
  let currentVideoId = '';

  let observedPlayer = null;
  let playerObserver = null;
  let hookedVideo = null;
  let rafScheduled = false;

  const SKIP_BUTTON_SELECTORS = [
    '.ytp-skip-ad-button',
    '.ytp-ad-skip-button-modern',
    '.ytp-ad-skip-button',
    '.ytp-ad-skip-button-slot button',
    '.ytp-ad-skip-button-container button',
    'button[id^="skip-button"]',
    '.ytp-ad-player-overlay-skip-or-preview button',
    '.ytp-ad-overlay-close-button',
    '.ytm-skip-ad-button'
  ];

  const ENDCAP_SELECTORS = [
    '.ytp-ad-action-interstitial',
    '.ytp-ad-timed-pie-countdown-container',
    '.ytp-ad-survey-questions'
  ].join(', ');

  function isAdblockDisabled() {
    return (
      document.documentElement &&
      document.documentElement.getAttribute('data-yt-adblock-disabled') === 'true'
    );
  }

  let currentCustomSpeed = 1.0;
  try {
    const savedSpeed = parseFloat(localStorage.getItem('yt_custom_speed'));
    if (!isNaN(savedSpeed) && savedSpeed >= 0 && savedSpeed <= 16) {
      currentCustomSpeed = savedSpeed;
    }
  } catch (e) {}

  function getTargetSpeed() {
    if (document.documentElement) {
      const attr = parseFloat(document.documentElement.getAttribute('data-yt-custom-speed'));
      if (!isNaN(attr) && attr >= 0 && attr <= 16) return attr;
    }
    return currentCustomSpeed;
  }

  function getMoviePlayer() {
    return document.getElementById('movie_player') || document.querySelector('.html5-video-player');
  }

  function getPlayerApi(moviePlayerEl) {
    if (!moviePlayerEl) return null;
    return typeof moviePlayerEl.getPlayer === 'function'
      ? moviePlayerEl.getPlayer()
      : moviePlayerEl;
  }

  function getMainVideo(moviePlayerEl) {
    const root = moviePlayerEl || document;
    return (
      root.querySelector('video.html5-main-video') ||
      document.querySelector('video.html5-main-video') ||
      root.querySelector('video')
    );
  }

  function getCurrentVideoId(playerApi) {
    try {
      if (playerApi && typeof playerApi.getVideoData === 'function') {
        const data = playerApi.getVideoData();
        if (data && data.video_id) return data.video_id;
      }
    } catch (e) {}
    try {
      return new URLSearchParams(window.location.search).get('v') || '';
    } catch (e) {
      return '';
    }
  }

  function isElementVisible(el) {
    if (!el) return false;
    const style = window.getComputedStyle(el);
    if (style.display === 'none' || style.visibility === 'hidden' || parseFloat(style.opacity) === 0) {
      return false;
    }
    const rect = el.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  }

  function resetAdCycleState() {
    lastSeekedAdKey = '';
    lastSeekedAt = 0;
    skipClickAttempts = 0;
    firstSkipClickAt = 0;
    endcapRecoverAttempts = 0;
  }

  function detectAdState(moviePlayerEl) {
    if (!moviePlayerEl || location.pathname.startsWith('/shorts/')) {
      return { isAd: false, isEndScreen: false };
    }

    const hasAdClass =
      moviePlayerEl.classList.contains('ad-showing') ||
      moviePlayerEl.classList.contains('ad-interrupting');

    const endcapEl = moviePlayerEl.querySelector(ENDCAP_SELECTORS);
    const isEndScreen = Boolean(endcapEl && isElementVisible(endcapEl));

    const overlayEl = moviePlayerEl.querySelector(
      '.ytp-ad-player-overlay, .ytp-ad-player-overlay-layout'
    );
    const hasVisibleOverlay = Boolean(overlayEl && isElementVisible(overlayEl));

    const isAd = Boolean(hasAdClass || isEndScreen || hasVisibleOverlay);
    return { isAd, isEndScreen };
  }

  function dispatchTrustedClickSequence(btn) {
    const rect = btn.getBoundingClientRect();
    const clientX = Math.round(rect.left + rect.width / 2);
    const clientY = Math.round(rect.top + rect.height / 2);
    const screenX = clientX + (window.screenX || 0);
    const screenY = clientY + (window.screenY || 0);

    const baseInit = {
      view: window,
      bubbles: true,
      cancelable: true,
      composed: true,
      clientX,
      clientY,
      screenX,
      screenY,
      detail: 1,
      button: 0
    };

    const events = [];
    if (typeof PointerEvent === 'function') {
      events.push(
        new PointerEvent('pointerdown', {
          ...baseInit,
          buttons: 1,
          pointerId: 1,
          pointerType: 'mouse',
          isPrimary: true,
          pressure: 0.5
        })
      );
    }
    events.push(new MouseEvent('mousedown', { ...baseInit, buttons: 1 }));
    if (typeof PointerEvent === 'function') {
      events.push(
        new PointerEvent('pointerup', {
          ...baseInit,
          buttons: 0,
          pointerId: 1,
          pointerType: 'mouse',
          isPrimary: true,
          pressure: 0
        })
      );
    }
    events.push(new MouseEvent('mouseup', { ...baseInit, buttons: 0 }));
    events.push(new MouseEvent('click', { ...baseInit, buttons: 0 }));

    for (const ev of events) {
      ev[SYNTHETIC_TRUSTED_MARK] = true;
      btn.dispatchEvent(ev);
    }
  }

  function tryClickSkipButtons(moviePlayerEl) {
    const now = Date.now();
    if (skipClickAttempts >= 3 || now - lastSkipClickTime < 250) return false;

    const root = moviePlayerEl || document;
    for (const selector of SKIP_BUTTON_SELECTORS) {
      const buttons = root.querySelectorAll(selector);
      for (const btn of buttons) {
        if (isElementVisible(btn)) {
          try {
            dispatchTrustedClickSequence(btn);
            skipClickAttempts += 1;
            if (firstSkipClickAt === 0) firstSkipClickAt = now;
            lastSkipClickTime = now;
            reportAdSkipped();
            return true;
          } catch (e) {}
        }
      }
    }
    return false;
  }

  function processPlayerState() {
    rafScheduled = false;
    try {
      ensureStyleInjected();
      if (isAdblockDisabled()) return;

      const moviePlayerEl = getMoviePlayer();
      if (!moviePlayerEl) return;

      const playerApi = getPlayerApi(moviePlayerEl);
      const video = getMainVideo(moviePlayerEl);

      ensureScopedHooks(moviePlayerEl, video);

      const detectedVid = getCurrentVideoId(playerApi);
      if (detectedVid && detectedVid !== currentVideoId) {
        currentVideoId = detectedVid;
        lastMainVideoTime = 0;
        lastReloadedVideoId = '';
        resetAdCycleState();
      }

      const { isAd, isEndScreen } = detectAdState(moviePlayerEl);

      if (!isAd) {
        resetAdCycleState();
        if (video) {
          if (wasMutedByAd) {
            video.muted = false;
            wasMutedByAd = false;
          }
          if (!video.paused && !video.seeking && !isNaN(video.currentTime) && video.currentTime > 0) {
            lastMainVideoTime = Math.floor(video.currentTime);
          }
          const targetSpeed = getTargetSpeed();
          if (Math.abs(video.playbackRate - targetSpeed) > 0.01) {
            try {
              video.playbackRate = targetSpeed;
            } catch (e) {}
          }
        }
        return;
      }

      const now = Date.now();

      if (video && !video.muted) {
        video.muted = true;
        wasMutedByAd = true;
        reportAdSkipped();
      }

      tryClickSkipButtons(moviePlayerEl);

      const hasValidDuration =
        video && !isNaN(video.duration) && isFinite(video.duration) && video.duration > 0.5;

      if (hasValidDuration && !isEndScreen) {
        if (video.playbackRate !== 16.0) {
          try {
            video.playbackRate = 16.0;
          } catch (e) {}
        }

        if (video.readyState >= 2 && !video.seeking && video.currentTime < video.duration - 0.3) {
          const adKey = `${video.currentSrc || video.src || 'ad'}_${Math.round(video.duration)}`;
          if (lastSeekedAdKey !== adKey) {
            const targetTime = Math.max(0, video.duration - 0.1);
            lastSeekedAdKey = adKey;
            lastSeekedAt = now;
            endcapRecoverAttempts = 0;
            try {
              video.currentTime = targetTime;
              if (video.paused) {
                video.play().catch(() => {});
              }
              reportAdSkipped();
            } catch (e) {}
            return;
          }
        }
      }

      const adVideoFinishedOrUnloaded =
        !video ||
        isNaN(video.duration) ||
        video.ended ||
        video.paused ||
        video.currentTime >= video.duration - 0.5;

      const isStuckAfterFastForward =
        lastSeekedAdKey !== '' && now - lastSeekedAt >= 300 && adVideoFinishedOrUnloaded;

      const isStuckOnSkipButton =
        skipClickAttempts >= 1 &&
        now - firstSkipClickAt >= 300 &&
        (lastSeekedAdKey !== '' || adVideoFinishedOrUnloaded);

      const isStuckAtAdEnd =
        isEndScreen || isStuckAfterFastForward || isStuckOnSkipButton;

      if (isStuckAtAdEnd && endcapRecoverAttempts < 3 && now - lastEndcapRecoverTime > 550) {
        lastEndcapRecoverTime = now;
        endcapRecoverAttempts += 1;

        const isVideoUnloaded = !video || isNaN(video.duration);
        const vid = currentVideoId || getCurrentVideoId(playerApi);

        if (
          endcapRecoverAttempts === 2 &&
          isVideoUnloaded &&
          vid &&
          lastReloadedVideoId !== vid &&
          playerApi &&
          typeof playerApi.loadVideoById === 'function'
        ) {
          lastReloadedVideoId = vid;
          try {
            playerApi.loadVideoById(vid, lastMainVideoTime);
            return;
          } catch (e) {}
        }

        if (playerApi && typeof playerApi.seekTo === 'function') {
          try {
            playerApi.seekTo(lastMainVideoTime, true);
            if (typeof playerApi.playVideo === 'function') {
              playerApi.playVideo();
            }
          } catch (e) {}
        }
      }
    } catch (err) {}
  }

  function scheduleCheck() {
    if (rafScheduled) return;
    rafScheduled = true;
    requestAnimationFrame(processPlayerState);
  }

  function ensureScopedHooks(moviePlayerEl, video) {
    if (moviePlayerEl && moviePlayerEl !== observedPlayer) {
      if (playerObserver) {
        playerObserver.disconnect();
      }
      observedPlayer = moviePlayerEl;
      playerObserver = new MutationObserver(scheduleCheck);
      playerObserver.observe(moviePlayerEl, {
        attributes: true,
        attributeFilter: ['class'],
        childList: true,
        subtree: false
      });
    }

    if (video && video !== hookedVideo) {
      hookedVideo = video;
      ['loadedmetadata', 'durationchange', 'canplay', 'play', 'playing', 'ratechange', 'ended'].forEach(
        (evt) => {
          video.addEventListener(evt, scheduleCheck, { passive: true });
        }
      );
    }
  }

  ['yt-navigate-start', 'yt-navigate-finish', 'yt-page-data-updated'].forEach((evt) => {
    window.addEventListener(
      evt,
      () => {
        if (evt === 'yt-navigate-start') {
          lastMainVideoTime = 0;
          lastReloadedVideoId = '';
          resetAdCycleState();
        }
        scheduleCheck();
      },
      { passive: true }
    );
  });

  setInterval(scheduleCheck, 250);

  // =========================================================================
  // THANH ĐIỀU KHIỂN TỐC ĐỘ VIDEO 0x - 16x (DESKTOP + MOBILE)
  // =========================================================================
  let toastTimeout = null;
  let mobileHideTimer = null;

  function createEl(tag, attrs = {}, children = []) {
    const el = document.createElement(tag);
    for (const [key, val] of Object.entries(attrs)) {
      if (key === 'className') {
        el.className = val;
      } else if (key === 'textContent') {
        el.textContent = val;
      } else if (key === 'style' && typeof val === 'object') {
        Object.assign(el.style, val);
      } else {
        el.setAttribute(key, val);
      }
    }
    for (const child of children) {
      if (typeof child === 'string') {
        el.appendChild(document.createTextNode(child));
      } else if (child) {
        el.appendChild(child);
      }
    }
    return el;
  }

  function showToast(text) {
    try {
      const player = getMoviePlayer() || document.body;
      if (!player) return;
      let toast = document.getElementById('yt-speed-toast');
      if (!toast) {
        toast = createEl('div', { id: 'yt-speed-toast' });
        player.appendChild(toast);
      }
      toast.textContent = text;
      toast.style.opacity = '1';
      if (toastTimeout) clearTimeout(toastTimeout);
      toastTimeout = setTimeout(() => {
        toast.style.opacity = '0';
      }, 750);
    } catch (e) {}
  }

  function setSpeed(speed, notify = true) {
    let s = parseFloat(speed);
    if (isNaN(s)) return;
    s = Math.max(0, Math.min(16.0, Math.round(s * 100) / 100));
    currentCustomSpeed = s;

    try {
      localStorage.setItem('yt_custom_speed', String(s));
    } catch (e) {}

    if (document.documentElement) {
      document.documentElement.setAttribute('data-yt-custom-speed', String(s));
    }

    const mp = getMoviePlayer();
    const video = getMainVideo(mp);
    if (video && !detectAdState(mp).isAd) {
      try {
        video.playbackRate = currentCustomSpeed;
      } catch (e) {}
    }

    const displayText = document.getElementById('yt-speed-display-text');
    if (displayText) displayText.textContent = currentCustomSpeed + 'x';

    const topSlider = document.getElementById('yt-top-slider');
    if (topSlider) topSlider.value = String(currentCustomSpeed);

    const numInput = document.getElementById('yt-speed-panel-num');
    if (numInput && document.activeElement !== numInput) {
      numInput.value = String(currentCustomSpeed);
    }

    const panelSlider = document.getElementById('yt-speed-panel-slider');
    if (panelSlider) panelSlider.value = String(currentCustomSpeed);

    if (notify) {
      showToast('⚡ ' + currentCustomSpeed + 'x');
    }
  }

  function bumpMobileBarVisibility() {
    if (host !== 'm.youtube.com') return;
    const bar = document.getElementById('yt-topright-speedbar');
    if (!bar) return;
    bar.classList.remove('vn-yt-mobile-hidden');
    if (mobileHideTimer) clearTimeout(mobileHideTimer);
    mobileHideTimer = setTimeout(() => {
      const panel = document.getElementById('yt-speed-panel');
      if (!panel || panel.style.display === 'none') {
        bar.classList.add('vn-yt-mobile-hidden');
      }
    }, 3200);
  }

  function togglePanel(e) {
    if (e) e.stopPropagation();
    const panel = document.getElementById('yt-speed-panel');
    if (panel) {
      panel.style.display = panel.style.display === 'none' ? 'flex' : 'none';
      bumpMobileBarVisibility();
    }
  }

  function initTopRightSpeedBar() {
    try {
      ensureStyleInjected();
      if (location.pathname.startsWith('/shorts/')) return;
      const moviePlayer = getMoviePlayer();
      if (!moviePlayer) return;

      if (!document.getElementById('yt-topright-speedbar')) {
        const btnMinus = createEl('button', {
          className: 'yt-speed-btn-step',
          id: 'yt-btn-minus',
          title: 'Giảm tốc độ (-0.25x)',
          textContent: '−'
        });

        const displayText = createEl('span', {
          id: 'yt-speed-display-text',
          title: 'Nhấp để đặt lại 1.0x hoặc mở menu',
          textContent: currentCustomSpeed + 'x'
        });

        const btnPlus = createEl('button', {
          className: 'yt-speed-btn-step',
          id: 'yt-btn-plus',
          title: 'Tăng tốc độ (+0.25x)',
          textContent: '+'
        });

        const topSlider = createEl('input', {
          type: 'range',
          id: 'yt-top-slider',
          min: '0.25',
          max: '16',
          step: '0.25',
          value: String(currentCustomSpeed),
          title: 'Kéo để chỉnh tốc độ (0.25x - 16x)'
        });

        const btnMenu = createEl('button', {
          id: 'yt-speed-btn-menu',
          title: 'Mở bảng chọn nhanh',
          textContent: '☰'
        });

        const children = [btnMinus, displayText, btnPlus, topSlider];
        if (window.VNBridge && typeof window.VNBridge.enterPip === 'function') {
          const btnPip = createEl('button', {
            className: 'yt-speed-btn-step',
            id: 'yt-btn-pip',
            title: 'Mở video nổi (PiP)',
            textContent: '📺',
            style: { fontSize: '12px' }
          });
          btnPip.addEventListener('click', (e) => {
            e.stopPropagation();
            try { window.VNBridge.enterPip(); } catch (_) {}
          });
          children.push(btnPip);
        }
        children.push(btnMenu);

        const bar = createEl('div', { id: 'yt-topright-speedbar' }, children);

        ['click', 'dblclick', 'mousedown', 'mouseup', 'pointerdown', 'pointerup', 'touchstart', 'touchend'].forEach(
          (evt) => {
            bar.addEventListener(evt, (e) => {
              e.stopPropagation();
              bumpMobileBarVisibility();
            });
          }
        );

        btnMinus.addEventListener('click', () => setSpeed(currentCustomSpeed - 0.25));
        btnPlus.addEventListener('click', () => setSpeed(currentCustomSpeed + 0.25));
        displayText.addEventListener('click', () => {
          if (currentCustomSpeed !== 1.0) {
            setSpeed(1.0);
          } else {
            togglePanel();
          }
        });
        topSlider.addEventListener('input', () => setSpeed(topSlider.value));
        btnMenu.addEventListener('click', togglePanel);

        moviePlayer.appendChild(bar);
        moviePlayer.addEventListener('touchstart', bumpMobileBarVisibility, { passive: true, capture: true });
        bumpMobileBarVisibility();
      }

      if (!document.getElementById('yt-speed-panel')) {
        const headerRow = createEl(
          'div',
          {
            className: 'yt-speed-row',
            style: { fontWeight: '700', borderBottom: '1px solid #444', paddingBottom: '6px' }
          },
          [
            createEl('span', { style: { color: '#00ffcc' }, textContent: 'Tốc độ video (0x - 16x)' }),
            createEl('span', { style: { fontSize: '11px', color: '#888' }, textContent: '[ / ]' })
          ]
        );

        const numInput = createEl('input', {
          type: 'number',
          id: 'yt-speed-panel-num',
          min: '0',
          max: '16',
          step: '0.25',
          value: String(currentCustomSpeed)
        });

        const inputRow = createEl('div', { className: 'yt-speed-row' }, [
          createEl('span', { textContent: 'Nhập số:' }),
          numInput
        ]);

        const panelSlider = createEl('input', {
          type: 'range',
          id: 'yt-speed-panel-slider',
          min: '0.25',
          max: '16',
          step: '0.25',
          value: String(currentCustomSpeed)
        });

        const sliderRow = createEl('div', { className: 'yt-speed-row' }, [panelSlider]);

        const presetsData = [
          { label: '0.5x', val: '0.5' },
          { label: '1.0x', val: '1.0' },
          { label: '1.25x', val: '1.25' },
          { label: '1.5x', val: '1.5' },
          { label: '2.0x', val: '2.0' },
          { label: '3.0x', val: '3.0' },
          { label: '4.0x', val: '4.0' },
          { label: '8.0x', val: '8.0' },
          { label: '🚀 Tối đa 16x', val: '16.0', isMax: true }
        ];

        const presetButtons = presetsData.map((item) => {
          const btn = createEl('button', {
            className: item.isMax ? 'yt-speed-preset-btn btn-16' : 'yt-speed-preset-btn',
            textContent: item.label
          });
          btn.addEventListener('click', (e) => {
            e.stopPropagation();
            setSpeed(item.val);
          });
          return btn;
        });

        const presetsGrid = createEl('div', { className: 'yt-speed-presets' }, presetButtons);

        const panel = createEl(
          'div',
          { id: 'yt-speed-panel', style: { display: 'none' } },
          [headerRow, inputRow, sliderRow, presetsGrid]
        );

        ['click', 'dblclick', 'mousedown', 'mouseup', 'pointerdown', 'pointerup', 'touchstart', 'touchend'].forEach(
          (evt) => {
            panel.addEventListener(evt, (e) => e.stopPropagation());
          }
        );

        numInput.addEventListener('change', () => setSpeed(numInput.value));
        numInput.addEventListener('keydown', (e) => e.stopPropagation());
        panelSlider.addEventListener('input', () => setSpeed(panelSlider.value));

        moviePlayer.appendChild(panel);

        document.addEventListener('click', (e) => {
          const bar = document.getElementById('yt-topright-speedbar');
          if (!panel.contains(e.target) && (!bar || !bar.contains(e.target))) {
            panel.style.display = 'none';
          }
        });
      }
    } catch (err) {}
  }

  window.addEventListener('keydown', (e) => {
    const activeEl = document.activeElement;
    if (
      activeEl &&
      (activeEl.tagName === 'INPUT' ||
        activeEl.tagName === 'TEXTAREA' ||
        activeEl.isContentEditable)
    ) {
      return;
    }

    if (e.key === ']') {
      const step = e.shiftKey ? 1.0 : 0.25;
      setSpeed(currentCustomSpeed + step);
    } else if (e.key === '[') {
      const step = e.shiftKey ? 1.0 : 0.25;
      setSpeed(currentCustomSpeed - step);
    } else if (e.key === '\\') {
      setSpeed(1.0);
    }
  });

  ['DOMContentLoaded', 'yt-navigate-finish', 'yt-page-data-updated'].forEach((evt) => {
    window.addEventListener(evt, initTopRightSpeedBar, { passive: true });
  });

  setInterval(initTopRightSpeedBar, 500);
})();
