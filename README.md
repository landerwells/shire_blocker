# Shire Blocker
A simple, cross-platform, text-based configuration tool to block websites and applications.

### Work on tomorrow
- Integrating CLI tool commands with daemon
    - better parsing for listing blocks
- Installing on Linux (systemd service)

### Features
Before release
- [ ] Unit tests and integration tests??
- [ ] Need to put Firefox add-on in the store.
- [ ] Config file
- [ ] Commands for CLI
- [x] Whitelist and Blacklist

After release
- Look into rusqlite for persisting locks if the service is pkilled
- Have a timer set to refresh occasionally the active tab so that there won't be blacklisted tabs open
- Hotload config?
- Scheduling

### Goals
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

