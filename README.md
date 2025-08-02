# Shire Blocker
A simple, cross-platform, text-based configuration tool to block websites and applications.

Cold-Turkey-Blocker is able to resist pkill, and is able to recall all blocks 
again after. This is all because of SQLite. What kind of data do I need to keep
in the database? 
- When locks on specific blocks end 
- I guess settings don't have to be persisted as they will be in the config file
- Maybe I should think of the delay effect being put on, since I could parse the config file, and then store all of the settings in the database, and not make updates until the delay has happened?

- I will also have to store block data in the database for

There would be config delay updat

Another killer feature would be to sync across devices???? I understand the 
config would handle that, but syncing locks would be satisfying




### Work on tomorrow
- Formalize the message sending back to the browser
- Set up the display page for when the user tries to access a blocked site
- Installing the plist file for launchd
- Build a page for when the user tries to access a blocked site

### Features
Before release
- Need to put Firefox add-on in the store.
- Config file
- Commands for CLI
- Whitelist and Blacklist

After release
- Look into rusqlite for persisting application state
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
- I have seen that the delay of pluckeye is quite nice, perhaps I could look into that model and see to adding that feature as an additional setting?

Things I will not add
- Creation and deletion of block in the command line, this is literally what the config is for

