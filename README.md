TODO
- Rename extension to shire_extension
- Possibly break things out into workspaces if it gets too cluttered


MVP
- Config file (no hotload yet)
- Commands for CLI
- Whitelist and Blacklist

Additional features
- Have a timer set to refresh occasionally the active tab so that there won't be blacklisted tabs open
- 

Goals of this project
- Only support Firefox (possibly Safari at some point)
- Cross-platform (Linux and MacOS)
- Text based configuration (would consider a UI later)
- CLI would also be able to control commands and start blocks
- Support for NixOS, Homebrew, and cargo installations

Features I would consider adding
- Do not disturb mode for mac
- Specifying an additional configuration file to read if there are blocks that you want to keep private






Commands I want to support


- Need to list all blocks
- Need to list what is blocked in a specific block
- start a block for a certain amount of time



Things I will not add
- Creation and deletion of block in the command line, this is literally what the config is for

how to resist pkill????
Need to put something like this in Launch agents, will need a windows and linux version of this
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
<key>AssociatedBundleIdentifiers</key>
<array><string>com.getcoldturkey.blocker</string></array>
<key>Label</key>
<string>launchkeep.cold-turkey</string>
<key>KeepAlive</key>
<true/>
<key>Program</key>
<string>/Applications/Cold Turkey Blocker.app/Contents/MacOS/Cold Turkey Blocker</string>
</dict>
</plist>



<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.example.shire</string>

  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/shire</string>
    <string>daemon</string>
    <string>start</string>
  </array>

  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
</dict>
</plist>
