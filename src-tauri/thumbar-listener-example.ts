// Example renderer snippet: listens for native thumbar events and attempts
// to trigger play/pause in the page if possible. Drop this into your
// renderer bundle (or run from the devtools console) to enable a simple
// integration when you can't modify the page source.

// If your renderer already has a global `window.__TAURI__` import, use that
// instead — this example assumes the Tauri event API is available as
// `window.__TAURI__.event.listen` (Tauri v2+).

(async () => {
  // Prefer the modern Tauri global if present; otherwise try the legacy
  // `window.__TAURI__` namespace.
  const events = (window as any).__TAURI__?.event ?? (window as any).tauri?.event;
  if (!events || !events.listen) {
    console.warn('Tauri event API not detected; include this file in your renderer or run it from the console');
    return;
  }

  function tryPlayPause() {
    try {
      const media = document.querySelector('audio, video') as HTMLMediaElement | null;
      if (media) {
        if (media.paused) media.play(); else media.pause();
        return;
      }
      // include the page-specific selectors you provided and pause variants
      const selectors = [
        'span.pct.player__action-play.pct-player-play',
        'span.pct.player__action-pause.pct-player-pause',
        '.player__action-play',
        '.player__action-pause',
        '.pct-player-play',
        '.pct-player-pause',
        'button[aria-label*="play"]','button[aria-label*="Play"]','button[aria-label*="pause"]','button[aria-label*="Pause"]',
        'button.play','button.pause','.play-button','.pause-button','[data-testid*="play"]'
      ];
      for (const s of selectors) {
        const el = document.querySelector(s) as HTMLElement | null;
        if (el) { el.click(); return; }
      }

      // Fallback: click any button containing the word "play"
      const buttons = Array.from(document.querySelectorAll('button')) as HTMLElement[];
      for (const b of buttons) {
        if (/play/i.test(b.innerText || '')) { b.click(); return; }
      }
    } catch (e) {
      console.warn('thumbar example: play/pause attempt failed', e);
    }
  }

  // Helper to call the Tauri command that updates the native thumbar
  // play/pause icon. Use whichever `invoke` surface is available.
  const invoke = (window as any).__TAURI__?.invoke ?? (window as any).tauri?.invoke;
  async function setNativePlaying(isPlaying: boolean) {
    try {
      if (invoke) {
          // native_set_playing removed; renderer is authoritative and does not
          // call into native for static icons. Keep local state and UI in sync
          // within the renderer instead.
      }
    } catch (e) {
      // Non-fatal; just log so devs can see if the host command fails.
      console.warn('thumbar example: native_set_playing invoke failed', e);
    }
  }

  // Listen for the native `thumbar-playpause` event and try to toggle
  // playback in-page. This preserves the original example behaviour.
  events.listen('thumbar-playpause', () => {
    tryPlayPause();
    setTimeout(tryPlayPause, 200);
    setTimeout(tryPlayPause, 600);
  });

  // Keep the native thumbar icon in sync with actual HTMLMediaElement
  // playback state. This implements the "renderer-driven" Option A:
  // when the page starts/stops playback, notify the native thumbar.
  function attachMediaListeners(media: HTMLMediaElement) {
    const onPlay = () => setNativePlaying(true);
    const onPause = () => setNativePlaying(false);
    media.addEventListener('play', onPlay);
    media.addEventListener('pause', onPause);
    // update current state immediately
    setNativePlaying(!media.paused && !media.ended);
    // store handlers on the element so we can remove them later if needed
    (media as any).__thumbar_listeners = { onPlay, onPause };
  }

  function detachMediaListeners(media: HTMLMediaElement) {
    const h = (media as any).__thumbar_listeners;
    if (h) {
      media.removeEventListener('play', h.onPlay);
      media.removeEventListener('pause', h.onPause);
      delete (media as any).__thumbar_listeners;
    }
  }

  // Attach to existing media elements on the page.
  const existing = Array.from(document.querySelectorAll('audio, video')) as HTMLMediaElement[];
  for (const m of existing) attachMediaListeners(m);

  // Watch for future media elements (single-page apps may add/remove them).
  const mo = new MutationObserver((records) => {
    for (const r of records) {
      for (const n of Array.from(r.addedNodes)) {
        if (n instanceof HTMLMediaElement) attachMediaListeners(n);
        else if (n instanceof HTMLElement) {
          const found = Array.from(n.querySelectorAll('audio, video')) as HTMLMediaElement[];
          for (const f of found) attachMediaListeners(f);
        }
      }
      for (const n of Array.from(r.removedNodes)) {
        if (n instanceof HTMLMediaElement) detachMediaListeners(n);
        else if (n instanceof HTMLElement) {
          const found = Array.from(n.querySelectorAll('audio, video')) as HTMLMediaElement[];
          for (const f of found) detachMediaListeners(f);
        }
      }
    }
  });
  mo.observe(document.body || document.documentElement || document, { childList: true, subtree: true });

  console.log('thumbar-listener-example loaded (media observers attached)');
})();
