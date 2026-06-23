(function(){
    let selectors = [
        'button[aria-label*="revious"]',
        'button[aria-label*="Previous"]',
        'button[aria-label*="PREVIOUS"]',
        'button[title*="revious"]',
        'button[title*="Previous"]',
        '.pct-player-previous',
        '.player__action-previous',
        'button[class*="previous"]',
        'button[class*="prev"]',
        'button[class*="back"]',
        '[data-testid*="previous"]',
        '[data-testid*="prev"]',
        'button.pct-player-previous',
        'span.pct-player-previous'
    ];
    for (let s of selectors) {
        let el = document.querySelector(s);
        if (el) { el.click(); return; }
    }
})();
