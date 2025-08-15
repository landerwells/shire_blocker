// Content script to handle blocking and display blank page
(function () {
  'use strict';

  // Check if this page is blocked
  function checkIfBlocked() {
    const currentUrl = window.location.href;

    // Send message to background script to check if URL is blocked
    browser.runtime.sendMessage({
      action: "checkBlocked",
      url: currentUrl
    }).then((response) => {
      console.log("Content script received response:", response);

      if (response && response.blocked) {
        console.log("Site is blocked, displaying blocked page");
        displayBlockedPage();
      } else {
        console.log("Site is allowed");
      }
    }).catch((error) => {
      console.error("Error checking if blocked:", error);
    });
  }

  // Display a blank page with blocking message
  function displayBlockedPage() {
    // Clear the entire page
    document.documentElement.innerHTML = '';

    // Create a simple blocked page
    const blockedContent = `
      <!DOCTYPE html>
      <html>
      <head>
        <title>Site Blocked</title>
        <style>
          body {
            font-family: Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background-color: #f5f5f5;
            color: #333;
          }
          .blocked-container {
            text-align: center;
            padding: 2rem;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            max-width: 500px;
            width: 90%;
          }
          .blocked-icon {
            font-size: 4rem;
            margin-bottom: 1rem;
          }
          h1 {
            color: #d32f2f;
            margin-bottom: 1rem;
          }
          p {
            color: #666;
            margin-bottom: 1rem;
          }
          .url {
            font-family: monospace;
            background: #f0f0f0;
            padding: 0.5rem;
            border-radius: 4px;
            word-break: break-all;
            margin: 1rem 0;
          }
        </style>
      </head>
      <body>
        <div class="blocked-container">
          <div class="blocked-icon">🚫</div>
          <h1>Site Blocked</h1>
          <p>This site has been blocked by Shire Blocker.</p>
          <div class="url">${window.location.href}</div>
          <p><small>To unblock this site, modify your configuration file.</small></p>
        </div>
      </body>
      </html>
    `;

    document.documentElement.innerHTML = blockedContent;
  }

  // Check immediately when script loads
  checkIfBlocked();

  // Also check when the page is fully loaded
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', checkIfBlocked);
  } else {
    checkIfBlocked();
  }

  // Listen for messages from background script
  browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (message.action === "blockPage") {
      console.log("Received block message from background script");
      displayBlockedPage();
    }
  });

  console.log("Content script loaded for:", window.location.href);
})();

