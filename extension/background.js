let port = browser.runtime.connectNative("com.shire_blocker");
let applicationState = null;


console.log(applicationState);

port.onMessage.addListener((message) => {
  console.log(`Received from bridge:`, message);
  
  if (message.type === "state_update") {
    applicationState = message.state;
    console.log("Updated application state:", applicationState);
    checkAllTabsAgainstState();
  } else {
    console.log("Getting unsupported message from bridge");
  }
});

console.log(applicationState);

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
