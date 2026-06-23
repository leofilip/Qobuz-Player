(function() {
    const titlebar = document.getElementById('custom-titlebar');
    if (titlebar) {
        titlebar.style.setProperty('background', '{}', 'important');
    }

    const buttons = document.querySelectorAll('.titlebar-button');
    if (buttons.length > 0) {
        buttons.forEach(btn => {
            btn.style.setProperty('color', '{}', 'important');
        });
    }
})();
