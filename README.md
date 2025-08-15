# Shire Blocker

A simple, cross-platform, text-based configuration tool to block websites and applications. Shire Blocker helps you maintain focus by temporarily blocking distracting websites with configurable rules, scheduling, and lock-in periods.

## Features

- **Cross-platform support** - Works on Linux and macOS
- **Text-based configuration** - Simple TOML configuration files
- **Flexible blocking** - Support for whitelist/blacklist URL patterns
- **Time-based locks** - Lock blocks for specific durations to prevent easy bypassing
- **Scheduling** - Automatically activate blocks during specified times
- **Service integration** - Runs as a system service for persistent blocking

## Installation

> [!NOTE]
> Currently building from source is the only way there is to install Shire Blocker. Other ways of installation will not be considered until the Firefox add-on is published and available. As the add-on is not available yet on Firefox, the only way to load the add-on is manually through "about:debugging".

### Prerequisites
- Rust toolchain (for building from source)
- Firefox browser (primary supported browser)

### Building from Source
```bash
git clone https://github.com/landerwells/shire_blocker.git
cd shire_blocker
cargo build --release
```

### Setup
```bash
# Build and install the binary
cargo install --path .

# Start the service
shire service start
```

## Uninstallation

Shire Blocker provides a uniform way to uninstall across operating systems. Simply run this command and then delete the directory.
```
shire service uninstall
```

## Configuration

Shire Blocker uses a TOML configuration file located at `~/.config/shire/shire.toml`.

### Configuration Structure

The configuration file consists of two main sections:
- `[[blocks]]` - Define blocking rules with names, blacklists, and whitelists
- `[[schedule]]` - Define automatic scheduling for blocks

### Example Configuration

```toml
# Block distracting social media feeds while allowing specific functionality
[[blocks]]
name = "algorithmic_feeds"
active_by_default = true
# Whitelist: Allow specific pages that are useful
whitelist = [
  "instagram.com/direct/inbox",     # Direct messages
  "instagram.com/p/*",             # Specific posts
  "reddit.com/r/*",                # Specific subreddits
  "reddit.com/search*",            # Search functionality
  "youtube.com/watch?*",           # Specific videos
  "youtube.com/results?search_query=*", # Search results
]
# Blacklist: Block main feeds and time-wasting pages
blacklist = [
  "facebook.com",
  "instagram.com",
  "linkedin.com/feed",
  "reddit.com",
  "tiktok.com",
  "twitter.com",
  "youtube.com",
]

# Block stock trading and financial sites during work hours
[[blocks]]
name = "finance"
active_by_default = false
blacklist = [
  "robinhood.com",
  "tradingview.com",
  "finance.yahoo.com",
  "marketwatch.com"
]

# Block major shopping and search sites for deep focus
[[blocks]]
name = "google, amazon"
active_by_default = false
blacklist = [
  "amazon.com",
  "google.com",
  "shopping.google.com"
]

# Schedule automatic blocking during work hours
[[schedule]]
block = "algorithmic_feeds"
days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
start = "08:00"
end = "18:00"

# Block trading sites during market hours
[[schedule]]
block = "finance"
days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
start = "09:30"
end = "16:00"
```

### Block Configuration Options

- `name` - Unique identifier for the block (used in CLI commands)
- `active_by_default` - Whether this block is active when the service starts
- `blacklist` - Array of URLs/domains to block
- `whitelist` - Array of URLs/domains to allow (overrides blacklist)
- URL patterns support wildcards (`*`) for flexible matching

### Schedule Configuration Options

- `block` - Name of the block to schedule
- `days` - Array of days when the schedule is active
- `start` - Time when blocking starts (24-hour format)
- `end` - Time when blocking ends (24-hour format)

## Usage

### Basic Commands

```bash
# List available blocks
shire block list

# Start a specific block
shire block start <block_name>

# Start a block with a time lock (in minutes)
shire block start <block_name> --lock 60

# Stop a specific block
shire block stop <block_name>

# Check service status
shire service status

# Start the service
shire service start

# Stop the service
shire service stop
```

### Deep Work Script

Inspired by Eric "Reysu", from his [blog post](https://reysu.io/posts/automate-your-deepwork).

A convenient script for starting focused work sessions. Save this as `deepwork`, make sure its executable, and put it in your PATH. Rename the categories to coincide with the blocks in shire.toml.

```bash
#!/usr/bin/env bash

echo -n "How long? (in hours): "
read hours

echo -n "Block google/amazon? (y/n): "
read google_amazon

echo -n "Block stocks? (y/n): "
read stocks

echo -n "Block messages? (y/n): "
read messages

minutes=$((hours * 60))

to_block=()
[[ "$stocks" == "y" ]] && to_block+=("finance")
[[ "$google_amazon" == "y" ]] && to_block+=("google, amazon")
[[ "$messages" == "y" ]] && to_block+=("silence")

echo ""
echo "Blocking ${to_block[*]} for $hours hours."
echo "Press any key to cancel..."

for i in {10..1}; do
    echo -n "$i... "
    read -t 1 -n 1 key && { echo "cancelled."; exit 0; }
done

echo ""

[[ "$stocks" == "y" ]] && shire block start "finance" --lock "$minutes"
[[ "$google_amazon" == "y" ]] && shire block start "google, amazon" --lock "$minutes"
[[ "$messages" == "y" ]] && shire block start "silence" --lock "$minutes"

# Optional: Start a timer (requires arttime)
if command -v arttime &> /dev/null; then
    arttime --nolearn -a butterfly -t "deep work time – blocking distractions" -g "${hours}h"
fi
```

## Roadmap

### Version 1.0 - Core Release
**Target: Q3 2024**

**Critical Features:**
- [ ] Database persistence with Rusqlite for lock state across reboots
- [ ] Firefox add-on published to Mozilla Add-ons store
- [ ] Scheduling system implementation
- [ ] Automatic tab refresh to enforce active blocks
- [ ] Comprehensive error handling and user feedback
- [ ] Cross-platform installation packages (Linux, macOS)

**Quality & Polish:**
- [ ] Unit and integration test coverage
- [ ] Documentation improvements and examples
- [ ] Performance optimization for large blocklists
- [ ] Logging and debugging improvements

### Version 1.1 - Enhanced Functionality
**Target: Q4 2024**

**Feature Enhancements:**
- [ ] Configuration hot-reloading without service restart
- [ ] Advanced scheduling with recurring patterns
- [ ] Statistics and usage tracking
- [ ] Import/export configuration profiles
- [ ] Whitelist/blacklist pattern validation

**Platform Support:**
- [ ] NixOS package and configuration
- [ ] Homebrew formula for macOS
- [ ] Cargo installation improvements

### Future Versions

**Potential Features:**
- [ ] Safari browser support
- [ ] macOS Do Not Disturb integration  
- [ ] Private configuration file support
- [ ] Delay-based blocking (Pluckeye-style)
- [ ] Simple GUI for configuration management
- [ ] Mobile companion app (focus mode sync)
- [ ] Light and dark mode setting in configuration for block page

## Project Goals

### Core Philosophy

Shire Blocker is designed to be a **simple, reliable, and effective** website blocking tool that prioritizes:

- **Simplicity over feature bloat** - Text-based configuration, minimal UI complexity
- **Reliability over convenience** - Strong lock mechanisms that can't be easily bypassed
- **Focus over distraction** - Helping users maintain deep work sessions and healthy digital habits

### Primary Objectives

**🎯 Core Functionality:**
- Firefox browser support (primary target)
- Cross-platform compatibility (Linux and macOS)
- Text-based TOML configuration for transparency and version control
- Time-based locking mechanisms to prevent impulsive bypassing
- Persistent blocking across browser restarts and system reboots

**📦 Distribution & Installation:**
- Multiple installation methods: Cargo, NixOS packages, Homebrew
- Service-based architecture for reliable background operation
- Easy setup and configuration for non-technical users

### Future Considerations

**✅ Features Under Consideration:**
- Safari browser support for macOS users
- macOS Do Not Disturb mode integration
- Private configuration files for sensitive block lists
- Delay-based blocking (inspired by Pluckeye's approach)
- Optional simple GUI for configuration management
- Statistics and usage insights

**❌ Non-Goals:**

- **Command-line block management** - Configuration should be done via TOML files for version control and transparency
- **Chrome/Chromium support** - Focus on Firefox's robust extension ecosystem
- **Mobile apps** - Desktop focus session tool, not a comprehensive digital wellness platform
- **Complex scheduling** - Keep scheduling simple and predictable
- **Social features** - No sharing, leaderboards, or social comparison features

