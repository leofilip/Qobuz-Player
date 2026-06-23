(function() {
    if (document.getElementById('qobuz-settings-overlay')) {
        return;
    }

    const isDark = document.documentElement.classList.contains('theme-dark');
    const isLight = document.documentElement.classList.contains('theme-light');
    const currentTheme = isLight ? 'light' : 'dark';

    const themeColors = {
        dark: { bg: '#1a1a1a', text: '#e0e0e0', btnBg: '#333', btnHover: '#444' },
        light: { bg: '#f5f5f5', text: '#242424', btnBg: '#e0e0e0', btnHover: '#d0d0d0' }
    };
    const colors = themeColors[currentTheme];

    const overlay = document.createElement('div');
    overlay.id = 'qobuz-settings-overlay';
    overlay.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: ${colors.bg};
        z-index: 9999999;
        display: flex;
        flex-direction: column;
        margin-top: 32px;
        overflow-y: auto;
    `;

    const closeBtn = document.createElement('button');
    closeBtn.id = 'qobuz-settings-back-btn';
    closeBtn.innerHTML = '← Back to Player';
    closeBtn.style.cssText = `
        position: fixed;
        top: 44px;
        left: 12px;
        padding: 8px 16px;
        background: ${colors.btnBg};
        color: ${colors.text};
        border: none;
        border-radius: 6px;
        cursor: pointer;
        font-size: 14px;
        font-weight: 500;
        z-index: 10000000;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    `;
    closeBtn.onmouseover = function() { this.style.background = colors.btnHover; };
    closeBtn.onmouseout = function() { this.style.background = colors.btnBg; };
    closeBtn.onclick = function() {
        const overlay = document.getElementById('qobuz-settings-overlay');
        if (overlay) {
            document.body.removeChild(overlay);
        }
        const btn = document.getElementById('qobuz-settings-back-btn');
        if (btn) {
            document.body.removeChild(btn);
        }
    };

    const styleElement = document.createElement('div');
    styleElement.innerHTML = `{}`;
    overlay.appendChild(styleElement);

    const contentDiv = document.createElement('div');
    contentDiv.innerHTML = `{}`;
    overlay.appendChild(contentDiv);

    document.body.appendChild(overlay);
    document.body.appendChild(closeBtn);

    const scripts = contentDiv.querySelectorAll('script');
    scripts.forEach(script => {
        const newScript = document.createElement('script');
        newScript.textContent = '(function(){' + script.textContent + '})();';
        document.body.appendChild(newScript);
    });
})();
