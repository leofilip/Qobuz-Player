function injectTitlebar() {
    const existing = document.getElementById('custom-titlebar');
    if (existing) existing.remove();

    const style = document.createElement('style');
    style.id = 'custom-titlebar-style';
    style.textContent = `
        #custom-titlebar {
            position: fixed !important;
            top: 0 !important;
            left: 0 !important;
            right: 0 !important;
            width: 100% !important;
            height: 32px !important;
            background: #181818 !important;
            display: flex !important;
            align-items: center !important;
            justify-content: flex-end !important;
            z-index: 2147483647 !important;
            -webkit-app-region: drag;
            user-select: none;
            -webkit-user-select: none;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif !important;
        }
        #custom-titlebar-controls {
            display: flex !important;
            align-items: center !important;
            -webkit-app-region: no-drag;
            height: 100% !important;
        }
        .titlebar-button {
            width: 46px !important;
            height: 32px !important;
            display: flex !important;
            align-items: center !important;
            justify-content: center !important;
            background: transparent !important;
            border: none !important;
            color: #e0e0e0 !important;
            cursor: pointer !important;
            transition: background-color 0.15s !important;
            padding: 0 !important;
            pointer-events: auto !important;
            -webkit-app-region: no-drag;
        }
        .titlebar-button:hover {
            background-color: rgba(255, 255, 255, 0.1) !important;
        }
        .titlebar-button.close:hover {
            background-color: #e81123 !important;
        }
        .titlebar-button svg {
            fill: currentColor !important;
            opacity: 0.9 !important;
        }
        .titlebar-button:hover svg {
            opacity: 1 !important;
        }
        .titlebar-button.settings-btn {
            margin-right: 8px !important;
        }
        .titlebar-button.settings-btn:hover {
            background-color: rgba(0, 102, 204, 0.3) !important;
        }

        .ui-app {
            margin-top: 32px !important;
        }

        .ui-layout-001--panel-outer-bottom {
            margin-bottom: 32px !important;
        }
    `;
    document.head.appendChild(style);

    const titlebar = document.createElement('div');
    titlebar.id = 'custom-titlebar';
    titlebar.innerHTML = `
        <div id="custom-titlebar-controls">
            <button class="titlebar-button settings-btn" id="titlebar-settings" title="Settings">
                <svg width="16" height="16" viewBox="0 0 512 512">
                    <path d="M496,293.984c9.031-0.703,16-8.25,16-17.297v-41.375c0-9.063-6.969-16.594-16-17.313l-54.828-4.281 c-3.484-0.266-6.484-2.453-7.828-5.688l-18.031-43.516c-1.344-3.219-0.781-6.906,1.5-9.547l35.75-41.813 c5.875-6.891,5.5-17.141-0.922-23.547l-29.25-29.25c-6.406-6.406-16.672-6.813-23.547-0.922l-41.813,35.75 c-2.641,2.266-6.344,2.844-9.547,1.516l-43.531-18.047c-3.219-1.328-5.422-4.375-5.703-7.828l-4.266-54.813 C293.281,6.969,285.75,0,276.688,0h-41.375c-9.063,0-16.594,6.969-17.297,16.016l-4.281,54.813c-0.266,3.469-2.469,6.5-5.688,7.828 l-43.531,18.047c-3.219,1.328-6.906,0.75-9.563-1.516l-41.797-35.75c-6.875-5.891-17.125-5.484-23.547,0.922l-29.25,29.25 c-6.406,6.406-6.797,16.656-0.922,23.547l35.75,41.813c2.25,2.641,2.844,6.328,1.5,9.547l-18.031,43.516 c-1.313,3.234-4.359,5.422-7.813,5.688L16,218c-9.031,0.719-16,8.25-16,17.313v41.359c0,9.063,6.969,16.609,16,17.313l54.844,4.266 c3.453,0.281,6.5,2.484,7.813,5.703l18.031,43.516c1.344,3.219,0.75,6.922-1.5,9.563l-35.75,41.813 c-5.875,6.875-5.484,17.125,0.922,23.547l29.25,29.25c6.422,6.406,16.672,6.797,23.547,0.906l41.797-35.75 c2.656-2.25,6.344-2.844,9.563-1.5l43.531,18.031c3.219,1.344,5.422,4.359,5.688,7.844l4.281,54.813 c0.703,9.031,8.234,16.016,17.297,16.016h41.375c9.063,0,16.594-6.984,17.297-16.016l4.266-54.813 c0.281-3.484,2.484-6.5,5.703-7.844l43.531-18.031c3.203-1.344,6.922-0.75,9.547,1.5l41.813,35.75 c6.875,5.891,17.141,5.5,23.547-0.906l29.25-29.25c6.422-6.422,6.797-16.672,0.922-23.547l-35.75-41.813 c-2.25-2.641-2.844-6.344-1.5-9.563l18.031-43.516c1.344-3.219,4.344-5.422,7.828-5.703L496,293.984z M256,342.516 c-23.109,0-44.844-9-61.188-25.328c-16.344-16.359-25.344-38.078-25.344-61.203c0-23.109,9-44.844,25.344-61.172 c16.344-16.359,38.078-25.344,61.188-25.344c23.125,0,44.844,8.984,61.188,25.344c16.344,16.328,25.344,38.063,25.344,61.172 c0,23.125-9,44.844-25.344,61.203C300.844,333.516,279.125,342.516,256,342.516z"/>
                </svg>
            </button>
            <button class="titlebar-button minimize" id="titlebar-minimize" title="Minimize">
                <svg width="10" height="1" viewBox="0 0 10 1">
                    <path d="M0 0h10v1H0z"/>
                </svg>
            </button>
            <button class="titlebar-button maximize" id="titlebar-maximize" title="Maximize">
                <svg width="10" height="10" viewBox="0 0 10 10">
                    <path d="M0 0v10h10V0H0zm1 1h8v8H1V1z"/>
                </svg>
            </button>
            <button class="titlebar-button close" id="titlebar-close" title="Close">
                <svg width="10" height="10" viewBox="0 0 10 10">
                    <path d="M0 0l10 10M10 0L0 10" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>
        </div>
    `;

    document.body.appendChild(titlebar);

    setTimeout(() => {
        if (typeof window.__TAURI__ === 'undefined') return;

        const { getCurrentWindow } = window.__TAURI__.window;
        const { invoke } = window.__TAURI__.core;
        const currentWindow = getCurrentWindow();

        document.getElementById('titlebar-minimize').onclick = () => invoke('minimize_window');
        document.getElementById('titlebar-maximize').onclick = async () => {
            if (await currentWindow.isMaximized()) {
                currentWindow.unmaximize();
            } else {
                currentWindow.maximize();
            }
        };
        document.getElementById('titlebar-close').onclick = () => currentWindow.close();
        document.getElementById('titlebar-settings').onclick = () => invoke('open_settings_window');

        function detectAndApplyTheme() {
            const html = document.documentElement;
            const isDark = html.classList.contains('theme-dark');
            const isLight = html.classList.contains('theme-light');

            if (isDark) {
                invoke('apply_theme_from_string', { theme: 'dark' });
            } else if (isLight) {
                invoke('apply_theme_from_string', { theme: 'light' });
            } else {
                invoke('apply_theme_from_string', { theme: 'dark' });
            }
        }

        setTimeout(detectAndApplyTheme, 1000);

        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
                    detectAndApplyTheme();
                }
            });
        });

        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['class']
        });
    }, 300);
}

function waitAndInject() {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => setTimeout(injectTitlebar, 500));
    } else {
        setTimeout(injectTitlebar, 500);
    }

    window.addEventListener('load', () => setTimeout(injectTitlebar, 500));
}

waitAndInject();
