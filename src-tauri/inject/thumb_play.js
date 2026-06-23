(function(){
    let m = document.querySelector('audio, video');
    if (m) {
        if (m.paused) m.play(); else m.pause();
    } else {
        document.querySelector(
            'button[aria-label*="lay"], button[aria-label*="ause"], .play-button, .pause-button, .pct-player-play, .pct-player-pause'
        )?.click();
    }
})();
