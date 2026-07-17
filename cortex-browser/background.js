const CORTEX_URL = 'http://localhost:8787/active-context';

chrome.commands.onCommand.addListener((command, tab) => {
  if (command === 'inject-cortex') {
    injectContext(tab);
  }
});

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'inject') {
    injectContext(sender.tab).then(sendResponse);
    return true;
  }
});

async function injectContext(tab) {
  try {
    const response = await fetch(CORTEX_URL, {
      method: 'GET',
      headers: { 'Accept': 'text/plain' }
    });

    if (!response.ok) {
      throw new Error(`Cortex returned ${response.status}`);
    }

    const context = await response.text();

    await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: insertTextIntoChat,
      args: [context]
    });

    return { success: true };
  } catch (err) {
    console.error('Cortex injection failed:', err);
    await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: showNotification,
      args: [`🧠 Cortex not running. Start it with: cortex serve`]
    });
    return { success: false, error: err.message };
  }
}

function insertTextIntoChat(text) {
  // Try Claude's ProseMirror editor first
  const proseMirror = document.querySelector('[contenteditable="true"].ProseMirror');
  if (proseMirror) {
    const p = proseMirror.querySelector('p');
    if (p) {
      p.textContent = text + '\n\n' + (p.textContent || '');
      proseMirror.dispatchEvent(new Event('input', { bubbles: true }));
      return;
    }
    proseMirror.textContent = text + '\n\n' + (proseMirror.textContent || '');
    proseMirror.dispatchEvent(new Event('input', { bubbles: true }));
    return;
  }

  // Try ChatGPT textarea / contenteditable div
  const textarea = document.querySelector('textarea[placeholder*="Message"]')
    || document.querySelector('textarea')
    || document.querySelector('#prompt-textarea');

  if (textarea) {
    const existing = textarea.value || '';
    textarea.value = text + '\n\n' + existing;
    textarea.dispatchEvent(new Event('input', { bubbles: true }));
    textarea.focus();
    return;
  }

  // Fallback: any contenteditable
  const editable = document.querySelector('[contenteditable="true"]');
  if (editable) {
    editable.textContent = text + '\n\n' + (editable.textContent || '');
    editable.dispatchEvent(new Event('input', { bubbles: true }));
    return;
  }

  console.error('Cortex: Could not find chat input element');
}

function showNotification(message) {
  const div = document.createElement('div');
  div.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background: #1a1a2e;
    color: #eee;
    padding: 16px 24px;
    border-radius: 12px;
    font-family: system-ui, sans-serif;
    font-size: 14px;
    z-index: 999999;
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
    border: 1px solid #333;
    max-width: 320px;
    line-height: 1.5;
  `;
  div.textContent = message;
  document.body.appendChild(div);
  setTimeout(() => div.remove(), 5000);
}
