function sendUrlToNative(url) {
  return browser.runtime.sendNativeMessage(
    "com.shire_blocker",
    { url: url }
  ).then((response) => {
    console.log("Native response:", response);

    // Handle the response from the native host
    if (response && response.status === "blocked") {
      console.log("Site is blocked:", url);
      return { blocked: true, url: url };
    } else if (response && response.status === "allowed") {
      console.log("Site is allowed:", url);
      return { blocked: false, url: url };
    } else if (response && response.status === "error") {
      console.error("Error from native host:", response.message);
      return { blocked: false, url: url, error: response.message };
    }
    return { blocked: false, url: url };
  }).catch((error) => {
    console.error("Failed to send native message:", error);
    return { blocked: false, url: url, error: error.message };
  });
}

function handleTabActivated(activeInfo) {
  browser.tabs.get(activeInfo.tabId).then((tab) => {
    if (tab.url && !tab.url.startsWith("about:")) {
      sendUrlToNative(tab.url).then((result) => {
        if (result.blocked) {
          // Send message to content script to block the page
          browser.tabs.sendMessage(activeInfo.tabId, {
            action: "blockPage",
            url: tab.url
          }).catch((error) => {
            console.log("Could not send block message to tab:", error);
          });
        }
      });
    }
  });
}

function handleTabUpdated(tabId, changeInfo, tab) {
  if (changeInfo.url && !changeInfo.url.startsWith("about:")) {
    sendUrlToNative(changeInfo.url).then((result) => {
      if (result.blocked) {
        // Send message to content script to block the page
        browser.tabs.sendMessage(tabId, {
          action: "blockPage",
          url: changeInfo.url
        }).catch((error) => {
          console.log("Could not send block message to tab:", error);
        });
      }
    });
  }
}

// Fires when the active tab changes
browser.tabs.onActivated.addListener(handleTabActivated);

// Fires when a tab's URL changes
browser.tabs.onUpdated.addListener(handleTabUpdated);

// Listen for messages from content scripts
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === "checkBlocked") {
    // Check if the URL is blocked
    sendUrlToNative(message.url).then((result) => {
      sendResponse(result);
    });
    return true; // Keep the message channel open for async response
  }
});
