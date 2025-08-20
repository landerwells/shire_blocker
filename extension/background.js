/**
 * Shire Blocker Extension Background Script
 * Handles communication with native bridge and manages website blocking logic
 */

let port = browser.runtime.connectNative("com.shire_blocker");
let blocks = new Map();
let blacklist = new Set();
let whitelist = new Set();

// Initialize bridge connection
port.postMessage("ping");
setupPortListeners();

// Handle bridge disconnection and reconnection
port.onDisconnect.addListener(() => {
  console.log("Bridge connection lost, attempting to reconnect...");
  setTimeout(() => {
    port = browser.runtime.connectNative("com.shire_blocker");
    setupPortListeners();
  }, 1000);
});

/**
 * Sets up listeners for the native bridge port
 */
function setupPortListeners() {
  port.onMessage.addListener(handleBridgeMessage);
  port.onDisconnect.addListener(() => {
    console.log("Bridge disconnected, reconnecting...");
    setTimeout(() => {
      port = browser.runtime.connectNative("com.shire_blocker");
      setupPortListeners();
    }, 1000);
  });
}

/**
 * Handles messages from the native bridge
 * @param {Object} message - Message from bridge
 */
function handleBridgeMessage(message) {
  try {
    console.log(`Received from bridge:`, message);

    if (message.type === "state_update") {
      if (message.state && message.state.blocks) {
        blocks = new Map(Object.entries(message.state.blocks));
        
        // Clear existing lists to prevent stale entries
        blacklist.clear();
        whitelist.clear();

        for (const [blockName, block] of blocks) {
          if (block.block_state === "Unblocked") {
            continue;
          }

          block.blacklist?.forEach(pattern => blacklist.add(pattern));
          block.whitelist?.forEach(pattern => whitelist.add(pattern));
          console.log(blacklist);
          console.log(whitelist);
        }

        checkAllTabsAgainstState();
      }
    } else {
      console.log("Getting unsupported message from bridge");
    }
  } catch (error) {
    console.error("Error handling bridge message:", error);
  }
}
/**
 * Checks if a URL should be blocked based on current blocking rules
 * @param {string} url - The URL to check
 * @returns {boolean} - True if URL should be blocked
 */
function isUrlBlocked(url) {
  // try {
  //   if (!url || typeof url !== 'string') {
  //     return false;
  //   }
  //
  //   // Normalize URL: remove protocol and www
  //   const normalizedUrl = url.replace(/^(https?:\/\/)?(www\.)?/, '');
  //
  //   if (!blocks || blocks.size === 0) {
  //     return false;
  //   }
  //
  //   // TODO: Change the url
  //
  //   // Check if URL matches any whitelist patterns first (whitelist overrides blacklist)
  //   for (const pattern of whitelist) {
  //     if (urlMatches(normalizedUrl, pattern)) {
  //       return false;
  //     }
  //   }
  //
  //   // Check if URL matches any blacklist patterns
  //   for (const pattern of blacklist) {
  //     if (urlMatches(normalizedUrl, pattern)) {
  //       return true;
  //     }
  //   }
  //
  //   return false;
  // } catch (error) {
  //   console.error(`Error checking if URL is blocked: ${url}`, error);
  //   return false;
  // }
  
  return isBlacklisted(url) && !isWhitelisted(url);
}

// Example removeHttpWww function
function removeHttpWww(url) {
    return url.replace(/^https?:\/\//, '').replace(/^www\./, '');
}

function isBlacklisted(url) {
    const cleanUrl = removeHttpWww(url);
    for (const entry of blacklist) {
        if (cleanUrl.startsWith(entry)) {
            return true;
        }
    }
    return false;
}

function isWhitelisted(url) {
    const cleanUrl = removeHttpWww(url);
    for (const pattern of whitelist) {
        const prefix = pattern.endsWith('*') ? pattern.slice(0, -1) : pattern;
        if (cleanUrl.startsWith(prefix)) {
            return true;
        }
    }
    return false;
}

/**
 * Checks if a URL starts with the given pattern
 * @param {string} url - The URL to test
 * @param {string} pattern - The pattern to match against
 * @returns {boolean} - True if URL starts with pattern
 */
function urlMatches(url, pattern) {
  try {
    if (!url || !pattern) {
      return false;
    }
    
    return url.startsWith(pattern);
  } catch (error) {
    console.error(`Error matching URL ${url} against pattern ${pattern}:`, error);
    return false;
  }
}

/**
 * Checks all open tabs against current blocking state
 */
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
      const blocked = isUrlBlocked(tab.url);
      console.log(`Should this tab be blocked? ${blocked} for URL: ${tab.url}`);
      if (blocked) {
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
    const blocked = isUrlBlocked(changeInfo.url);
    console.log(`Should this tab be blocked? ${blocked} for URL: ${changeInfo.url}`);
    if (blocked) {
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
