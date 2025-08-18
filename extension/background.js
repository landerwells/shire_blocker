// Currently all of the processing gets done in the daemon, which needs 
// to receive messages from the bridge, which gets them from the browser.
//
// All of this would lead to decreased performance based of the fact that the 
// browser could simply handle all of the decision making, as long as it has
// the most up to date state.

// function sendUrlToNative(url) {
//   return browser.runtime.sendNativeMessage(
//     "com.shire_blocker",
//     { url: url }
//   ).then((response) => {
//     console.log("Native response:", response);
//
//     // Handle the response from the native host
//     if (response && response.status === "blocked") {
//       console.log("Site is blocked:", url);
//       return { blocked: true, url: url };
//     } else if (response && response.status === "allowed") {
//       console.log("Site is allowed:", url);
//       return { blocked: false, url: url };
//     } else if (response && response.status === "error") {
//       console.error("Error from native host:", response.message);
//       return { blocked: false, url: url, error: response.message };
//     }
//     return { blocked: false, url: url };
//   }).catch((error) => {
//     console.error("Failed to send native message:", error);
//     return { blocked: false, url: url, error: error.message };
//   });
// }
//
// function handleTabActivated(activeInfo) {
//   browser.tabs.get(activeInfo.tabId).then((tab) => {
//     if (tab.url && !tab.url.startsWith("about:")) {
//       sendUrlToNative(tab.url).then((result) => {
//         if (result.blocked) {
//           // Send message to content script to block the page
//           browser.tabs.sendMessage(activeInfo.tabId, {
//             action: "blockPage",
//             url: tab.url
//           }).catch((error) => {
//             console.log("Could not send block message to tab:", error);
//           });
//         }
//       });
//     }
//   });
// }
//
// function handleTabUpdated(tabId, changeInfo, tab) {
//   if (changeInfo.url && !changeInfo.url.startsWith("about:")) {
//     sendUrlToNative(changeInfo.url).then((result) => {
//       if (result.blocked) {
//         // Send message to content script to block the page
//         browser.tabs.sendMessage(tabId, {
//           action: "blockPage",
//           url: changeInfo.url
//         }).catch((error) => {
//           console.log("Could not send block message to tab:", error);
//         });
//       }
//     });
//   }
// }

// // Fires when the active tab changes
// browser.tabs.onActivated.addListener(handleTabActivated);
//
// // Fires when a tab's URL changes
// browser.tabs.onUpdated.addListener(handleTabUpdated);
//
// // Listen for messages from content scripts
// browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
//   if (message.action === "checkBlocked") {
//     // Check if the URL is blocked
//     sendUrlToNative(message.url).then((result) => {
//       sendResponse(result);
//     });
//     return true; // Keep the message channel open for async response
//   }
// });

let port = browser.runtime.connectNative("com.shire_blocker");
let applicationState = null;
console.log("The application has connected to com.shire_blocker")

port.onMessage.addListener((message) => {
  console.log(`Received from bridge:`, message);
  
  if (message.type === "state_update") {
    applicationState = message.state;
    console.log("Updated application state:", applicationState);
    checkAllTabsAgainstState();
  } else if (message.type === "block_update") {
    handleBlockUpdate(message);
  }
});

// port.onDisconnect.addListener(() => {
//   console.log("Bridge connection lost, attempting to reconnect...");
//   setTimeout(() => {
//     port = browser.runtime.connectNative("com.shire_blocker");
//     setupPortListeners();
//   }, 1000);
// });

function setupPortListeners() {
  port.onMessage.addListener(handleBridgeMessage);
  // port.onDisconnect.addListener(handleBridgeDisconnect);
}

function handleBridgeMessage(message) {
  if (message.type === "state_update") {
    applicationState = message.state;
    checkAllTabsAgainstState();
  }
}

// function handleBridgeDisconnect() {
//   console.log("Bridge disconnected, reconnecting...");
//   setTimeout(() => {
//     port = browser.runtime.connectNative("com.shire_blocker");
//     setupPortListeners();
//   }, 1000);
// }

function isUrlBlocked(url) {
  if (!applicationState || !applicationState.active_blocks) {
    return false;
  }
  
  for (const block of applicationState.active_blocks) {
    if (block.blacklist) {
      for (const pattern of block.blacklist) {
        if (urlMatches(url, pattern)) {
          if (block.whitelist) {
            for (const whitePattern of block.whitelist) {
              if (urlMatches(url, whitePattern)) {
                return false;
              }
            }
          }
          return true;
        }
      }
    }
  }
  return false;
}

function urlMatches(url, pattern) {
  const regex = new RegExp(pattern.replace(/\*/g, '.*'));
  return regex.test(url);
}

function checkAllTabsAgainstState() {
  if (!applicationState) return;
  
  browser.tabs.query({}).then(tabs => {
    tabs.forEach(tab => {
      if (tab.url && !tab.url.startsWith("about:") && !tab.url.startsWith("moz-extension:")) {
        if (isUrlBlocked(tab.url)) {
          browser.tabs.sendMessage(tab.id, {
            action: "blockPage",
            url: tab.url
          }).catch(() => {});
        }
      }
    });
  });
}

function handleTabActivated(activeInfo) {
  browser.tabs.get(activeInfo.tabId).then((tab) => {
    if (tab.url && !tab.url.startsWith("about:") && !tab.url.startsWith("moz-extension:")) {
      if (isUrlBlocked(tab.url)) {
        browser.tabs.sendMessage(activeInfo.tabId, {
          action: "blockPage",
          url: tab.url
        }).catch(() => {});
      }
    }
  });
}

function handleTabUpdated(tabId, changeInfo, tab) {
  if (changeInfo.url && !changeInfo.url.startsWith("about:") && !changeInfo.url.startsWith("moz-extension:")) {
    if (isUrlBlocked(changeInfo.url)) {
      browser.tabs.sendMessage(tabId, {
        action: "blockPage",
        url: changeInfo.url
      }).catch(() => {});
    }
  }
}

browser.tabs.onActivated.addListener(handleTabActivated);
browser.tabs.onUpdated.addListener(handleTabUpdated);

browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === "checkBlocked") {
    const blocked = isUrlBlocked(message.url);
    sendResponse({ blocked: blocked, url: message.url });
  }
  return true;
});
