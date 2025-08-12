# Shire Blocker
A simple, cross-platform, text-based configuration tool to block websites and applications.

## Installation

MacOS
```
shire service start
```
linux (for now)
```
shire service start
systemctl --user daemon-reload
systemctl --user enable shire.service
systemctl --user start shire.service
```


## Usage

``` zsh
deepwork() {
  read "hours? > how long? (in hours): "
  read "google_amazon? > block google/amazon? (y/n): "
  read "stocks? > block stocks? (y/n): "
  read "messages? > block messages? (y/n): "

  minutes=$((hours * 60))

  to_block=()
  [[ "$stocks" == "y" ]] && to_block+=("finance")
  [[ "$google_amazon" == "y" ]] && to_block+=("google, amazon")
  [[ "$messages" == "y" ]] && to_block+=("silence")

  echo ""
  echo "blocking ${to_block[*]} for $hours hours."
  echo "press any key to cancel..."

  for i in {10..1}; do
    echo -n "$i... "
    read -t 1 -n 1 key && { echo "cancelled."; return; }
  done

  echo ""

  [[ "$stocks" == "y" ]] && shire start "finance" --lock "$minutes"
  [[ "$google_amazon" == "y" ]] && shire start "google, amazon" --lock "$minutes"
  [[ "$messages" == "y" ]] && shire start "silence" --lock "$minutes"

  ~/.local/bin/arttime --nolearn -a butterfly -t "deep work time – blocking distractions" -g "${hours}h"
}
```

## Configuration

Example config

```
[[blocks]]
name = "Algorithmic Feeds"
active_by_default = true
whitelist = [
  "instagram.com/direct/inbox",
  "instagram.com/p/*",
  "linkedin.com/in/*",
  "reddit.com/message/inbox",
  "reddit.com/r/*",
  "reddit.com/search*",
  "reddit.com/user/*",
  "twitter.com/i/timeline",
  "youtube.com/@*",
  "youtube.com/c/*",
  "youtube.com/channel/*",
  "youtube.com/feed/library/*",
  "youtube.com/feed/subscriptions",
  "youtube.com/playlist?list=*",
  "youtube.com/results?search_query=*",
  "youtube.com/user/*",
  "youtube.com/watch?*"
]
blacklist = [
  "facebook.com",
  "instagram.com",
  "linkedin.com",
  "reddit.com",
  "tiktok.com",
  "twitter.com/home",
  "youtube.com",
]

[[blocks]]
name = "Stocks"
active_by_default = false
blacklist = [
  "robinhood.com",
  "tradingview.com",
  "finance.yahoo.com"
]

[[blocks]]
name = "Internet"
active_by_default = true
blacklist = [
  "amazon.com",
  "google.com",
]

[[schedule]]
block = "algorithmic_feeds"
days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
start = "08:00"
end = "18:00"

[[schedule]]
block = "stock_sites"
days = ["Mon", "Fri"]
start = "09:30"
end = "16:00"

```
## Roadmap

Before release
- [ ] Rusqlite for lock persistence throughout reboot and otherwise
- [ ] Need to put Firefox add-on in the store.
- [ ] Scheduling
- [ ] Have a timer set to refresh occasionally the active tab so that there won't be blacklisted tabs open
- [ ] General polish and good error handling everywhere

After release
- Hotload config?

## Goals
Primary goals of this project
- Only support Firefox (possibly Safari at some point)
- Cross-platform (Linux and MacOS first)
- Text based configuration
- Support for NixOS, Homebrew, and cargo installations

Features I would consider adding
- Do not disturb mode for mac
- Specifying an additional configuration file to read if there are blocks that you want to keep private
- Possibly a UI?
- I have seen that the delay of pluckeye is quite nice, perhaps I could look into that model and see to adding that feature as an additional setting? Would need to move a lot of data into the database for this change, will likely prioritize another database change first to get a feel for rusqlite.

Things I will not add
- Creation and deletion of block in the command line, this is literally what the config is for

