/**
 * Shire Blocker Extension Background Script
 * Handles communication with native bridge and manages website blocking logic
 */

let port = null;

/**
 * Connects to the native bridge and sets up listeners
 */
function connectToBridge() {
  console.log("hello!");
  port = browser.runtime.connectNative("com.shire_blocker");
  port.postMessage({ type: "ping" });

  port.onMessage.addListener(handleBridgeMessage);
  port.onDisconnect.addListener(() => {
    console.log("Bridge disconnected, attempting to reconnect...");
    // Attempt to reconnect after a short delay
    setTimeout(connectToBridge, 1000);
  });
}

/**
 * Handles only connection state messages from the bridge
 * @param {Object} message
 */
function handleBridgeMessage(message) {
  try {
    if (!message || !message.type) return;

    switch (message.type) {
      case "connected":
        console.log("Bridge connected to daemon");
        // TODO: Add any logic needed when connected
        break;

      case "disconnected":
        console.log("Bridge disconnected from daemon");
        // TODO: Add any logic needed when disconnected
        break;

      default:
        console.log("Ignoring unsupported bridge message:", message);
    }
  } catch (error) {
    console.error("Error handling bridge message:", error);
  }
}

// Start the connection
connectToBridge();

// // Switching tabs
// function handleTabActivated(activeInfo) {
//   browser.tabs.get(activeInfo.tabId).then((tab) => {
//     if (tab.url && !tab.url.startsWith("about:") && !tab.url.startsWith("moz-extension:")) {
//       const blocked = isUrlBlocked(tab.url);
//       console.log(`Should this tab be blocked? ${blocked} for URL: ${tab.url}`);
//       if (blocked) {
//         browser.tabs.sendMessage(activeInfo.tabId, {
//           action: "blockPage",
//           url: tab.url
//         }).catch(() => {});
//       }
//     }
//   });
// }
//
// // Loading new tab
// function handleTabUpdated(tabId, changeInfo, tab) {
//   if (changeInfo.url && !changeInfo.url.startsWith("about:") && !changeInfo.url.startsWith("moz-extension:")) {
//     const blocked = isUrlBlocked(changeInfo.url);
//     console.log(`Should this tab be blocked? ${blocked} for URL: ${changeInfo.url}`);
//     if (blocked) {
//       browser.tabs.sendMessage(tabId, {
//         action: "blockPage",
//         url: changeInfo.url
//       }).catch(() => {});
//     }
//   }
// }
//
// browser.tabs.onActivated.addListener(handleTabActivated);
// browser.tabs.onUpdated.addListener(handleTabUpdated);
//
// browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
//   if (message.action === "checkBlocked") {
//     const blocked = isUrlBlocked(message.url);
//     sendResponse({ blocked: blocked, url: message.url });
//   }
//   return true;
// });
