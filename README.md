# Shire Blocker
A simple, cross-platform, text-based configuration tool to block websites and applications.

### Work on tomorrow
- Installing the plist file for launchd
- Figure out how to send messages back to the browser extension
- Build a page for when the user tries to access a blocked site




Before release
- Need to put Firefox add-on in the store.
- Config file
- Commands for CLI
- Whitelist and Blacklist

After release
- Have a timer set to refresh occasionally the active tab so that there won't be blacklisted tabs open
- Hotload config?
- Scheduling

Primary goals of this project
- Only support Firefox (possibly Safari at some point)
- Cross-platform (Linux and MacOS first)
- Text based configuration
- Support for NixOS, Homebrew, and cargo installations

Features I would consider adding
- Do not disturb mode for mac
- Specifying an additional configuration file to read if there are blocks that you want to keep private
- Possibly a UI?

Things I will not add
- Creation and deletion of block in the command line, this is literally what the config is for

