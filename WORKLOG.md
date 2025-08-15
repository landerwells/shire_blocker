# Shire Blocker Development Log

## Next Priority Tasks
- Due to current architectural constraints, starting a block does not cause blacklisted sites to actually get blocked. This is because of the one-way message sending of the current bridge design. If this were changed to be a multi-directional, it would potentially eliminate this issue. It is worth looking.
- Maybe locking persistence first
- Scheduling?
- Add a way to pass a config file into the daemon to start it
- Write unit and integration tests for almost every use case for more efficient workflow

## Future Features
- Landing page improvements: display which block is preventing the current website, or multiple blocks, and if the block is due to a schedule. Possibly dark/light theme
- Listing blocks should display if they are schedule-based or not, but won't display the full schedule (this will be done with `schedule list`)

---

## Daily Work Log

### August 15, 2025
**Accomplished:**
- Basic implementation of scheduling blocks

### August 14, 2025
**Accomplished:**
- Fixed bug where switching tabs, or following links wouldn't block a page.
- Finished implementing all commands for CLI

### August 12, 2025
**Accomplished:**
- Combined daemon and cli into one for better commands 
- Refactoring and updating README.md 
- Working uninstall for Linux and MacOS

### August 11, 2025
**Accomplished:**
- Worked on refactoring and getting Linux installation working
- Separated WORKLOG and README
- Updated README file

### August 10, 2025
**Accomplished:**
- Refactored commands 

### August 8, 2025
**Accomplished:**
- Daemon implementation 
- File reorganization 
- Working on thread messaging 

### August 7, 2025
**Accomplished:**
- Working on CLI block start functionality 

### August 6, 2025
**Accomplished:**
- Manifest updates 

### August 4, 2025
**Accomplished:**
- Reached working milestone 
- Got whitelists working 

### August 3, 2025
**Accomplished:**
- Enhanced daemon logic 
- Switched to Unix sockets architecture 

### August 1, 2025
**Accomplished:**
- JavaScript integration work 
- Messages being sent to JavaScript 
- Working browser message communication 

### July 25, 2025
**Accomplished:**
- Working on bridge functionality 

### July 23, 2025
**Accomplished:**
- Better default settings and handling 
- General updates 

### July 21, 2025
**Accomplished:**
- Clap parsing working 
- Binary approach implementation 

### July 20, 2025
**Accomplished:**
- URL sending functionality 
- Code cleanup 
- Working prototype 
- Initial commit and project setup 
