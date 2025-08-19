let port = browser.runtime.connectNative("com.shire_blocker");
// Validation code to make sure bridge is set up to send and receive messages

port.postMessage("ping");

let blocks = new Map();
let blacklist = new Set();
let whitelist = new Set();
port.onMessage.addListener((message) => {
  console.log(`Received from bridge:`, message);

  if (message.type === "state_update") {
    if (message.state && message.state.blocks) {
      blocks = new Map(Object.entries(message.state.blocks));

      for (const [blockName, block] of blocks) {
        if (block.block_state === "Unblocked") {
          continue;
        }

        block.blacklist?.forEach(url => blacklist.add(url));
        block.whitelist?.forEach(url => whitelist.add(url));
      }

      checkAllTabsAgainstState();
    }
  } else {
    console.log("Getting unsupported message from bridge");
  }
});

// port.onDisconnect.addListener(() => {
//   console.log("Bridge connection lost, attempting to reconnect...");
//   setTimeout(() => {
//     port = browser.runtime.connectNative("com.shire_blocker");
//     setupPortListeners();
//   }, 1000);
// });

// function setupPortListeners() {
//   port.onMessage.addListener(handleBridgeMessage);
//   // port.onDisconnect.addListener(handleBridgeDisconnect);
// }

// function handleBridgeMessage(message) {
//   if (message.type === "state_update") {
//     if (message.state && message.state.blocks) {
//       blocks = new Map(Object.entries(message.state.blocks));
//       checkAllTabsAgainstState();
//     }
//   }
// }

// function handleBridgeDisconnect() {
//   console.log("Bridge disconnected, reconnecting...");
//   setTimeout(() => {
//     port = browser.runtime.connectNative("com.shire_blocker");
//     setupPortListeners();
//   }, 1000);
// }

// Alright, I think I have the message handling set up correctly, now I need
// to clean up the code for the extension.
function isUrlBlocked(url) {
  url = url.replace(/^(https?:\/\/)?(www\.)?/, '');

  if (!blocks || blocks.size === 0) {
    return false;
  }

  // Now simply check if url is in blacklist and not whitelist

  if (blacklist.has(url)) {
    if (whitelist.has(url)) {
      return false;
    }
    return true;
  }

  // I think that I should change this to how I had it in the rust code,
  // essentially I would just keep a global list of blacklisted URLs
  // and then check against that. 
  // for (const [blockName, block] of blocks) {
  //   if (block.block_state === "Unblocked") {
  //     continue;
  //   }
  //
  //   if (block.blacklist) {
  //     for (const pattern of block.blacklist) {
  //       if (urlMatches(url, pattern)) {
  //         if (block.whitelist) {
  //           for (const whitePattern of block.whitelist) {
  //             if (urlMatches(url, whitePattern)) {
  //               return false;
  //             }
  //           }
  //         }
  //         return true;
  //       }
  //     }
  //   }
}

function urlMatches(url, pattern) {
  const regex = new RegExp(pattern.replace(/\*/g, '.*'));
  return regex.test(url);
}

function checkAllTabsAgainstState() {
  if (!blocks || blocks.size === 0) return;

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

// Switching tabs
function handleTabActivated(activeInfo) {
  browser.tabs.get(activeInfo.tabId).then((tab) => {
    if (tab.url && !tab.url.startsWith("about:") && !tab.url.startsWith("moz-extension:")) {
      console.log("Should this tab be blocked? {}", isUrlBlocked(tab.url));
      if (isUrlBlocked(tab.url)) {
        browser.tabs.sendMessage(activeInfo.tabId, {
          action: "blockPage",
          url: tab.url
        }).catch(() => {});
      }
    }
  });
}

// Loading new tab
function handleTabUpdated(tabId, changeInfo, tab) {
  if (changeInfo.url && !changeInfo.url.startsWith("about:") && !changeInfo.url.startsWith("moz-extension:")) {
    if (isUrlBlocked(changeInfo.url)) {
      console.log("Should this tab be blocked? {}", isUrlBlocked(tab.url));
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
