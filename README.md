# Shire Blocker
A simple, cross-platform, text-based configuration tool to block websites and applications.

I wasn't able to get the application working on linux today. I believe that I need better testing for the bridge, or some way of testing that messages are getting passed correctly between it and the daemon.

These would be considered integration testing and should the system in general, specifically loading the configuration, and being able to send messages with the bridge. It would be nice if I had a combined interface to interact between the bridge and the daemon.

// For testing purposes, I think it would be beneficial to have a way to 
// pass a the configuration to the main function. This would allow us to 
// easily test different configurations without having to read from a file.

### Work on tomorrow
- Pulling all common message code into a separate file to standardize the message sending between processes.
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

