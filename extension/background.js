function sendUrlToNative(url) {
  browser.runtime.sendNativeMessage(
    "com.shire_blocker",
    { url: url }
  )
    // .then((response) => {
    //   console.log("Native response:", response);
    // }).catch((error) => {
    //   console.error("Failed to send native message:", error);
    // });
}

function handleTabActivated(activeInfo) {
  browser.tabs.get(activeInfo.tabId).then((tab) => {
    if (tab.url && !tab.url.startsWith("about:")) {
      sendUrlToNative(tab.url);
    }
  });
}

function handleTabUpdated(tabId, changeInfo, tab) {
  if (changeInfo.url && !changeInfo.url.startsWith("about:")) {
    sendUrlToNative(changeInfo.url);
  }
}

// Fires when the active tab changes
browser.tabs.onActivated.addListener(handleTabActivated);

// Fires when a tab's URL changes
browser.tabs.onUpdated.addListener(handleTabUpdated);

