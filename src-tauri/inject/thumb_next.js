(function(){
    let selectors = [
        'button[aria-label*="ext"]',
        'button[aria-label*="Next"]',
        '.pct-player-next',
        'button[class*="next"]',
        '[data-testid*="next"]'
    ];
    for (let s of selectors) {
        let el = document.querySelector(s);
        if (el) { el.click(); return; }
    }
})();
