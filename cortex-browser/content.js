(function() {
  'use strict';

  // Prevent duplicate injection
  if (window.__cortex_bridge_installed) return;
  window.__cortex_bridge_installed = true;

  const BUTTON_ID = 'cortex-inject-btn';

  function createButton() {
    if (document.getElementById(BUTTON_ID)) return;

    const btn = document.createElement('button');
    btn.id = BUTTON_ID;
    btn.innerHTML = '🧠';
    btn.title = 'Inject Cortex context (Ctrl+Shift+Space)';
    btn.onclick = () => {
      btn.style.transform = 'scale(0.9)';
      setTimeout(() => btn.style.transform = 'scale(1)', 150);
      chrome.runtime.sendMessage({ action: 'inject' });
    };

    document.body.appendChild(btn);
  }

  // Observe DOM changes to re-attach button if needed
  const observer = new MutationObserver(() => {
    if (!document.getElementById(BUTTON_ID)) {
      createButton();
    }
  });

  observer.observe(document.body, { childList: true, subtree: true });

  // Initial creation
  setTimeout(createButton, 2000);
})();
