# Shire Blocker Development Log

## Next Priority Tasks
- Maybe locking persistence first
- Scheduling?
- Add a way to pass a config file into the daemon to start it
- Write unit and integration tests for almost every use case for more efficient workflow

## Future Features
- Landing page improvements: display which block is preventing the current website, or multiple blocks, and if the block is due to a schedule
- Listing blocks should display if they are schedule-based or not, but won't display the full schedule (that will be done with `schedule list`)

---

## Daily Work Log

### August 12, 2025
**Accomplished:**
- Combined daemon and cli into one for better commands (`1fe5cea`)
- Refactoring and updating README.md (`eca3ce1`)
- Fixed WORKLOG.md typo (`cb1527a`)
- Merged master branch updates (`81f1f07`)

**Focus:** Architecture consolidation and documentation updates

### August 11, 2025
**Accomplished:**
- Worked on refactoring and getting Linux installation working
- Separated WORKLOG and README
- Planning to combine daemon and cli (`17c9f6e`)
- Updated README files (`8af0de2`, `c63e6c6`)
- Added WORKLOG.md and prepared for testing effort (`d2a9f3b`)
- Started install testing (`7cbb799`)

**Focus:** Linux compatibility and project organization

### August 10, 2025
**Accomplished:**
- Refactored commands (`fe59ad6`)
- Pre-unit tests preparation (`c2ce170`)

**Focus:** Code organization and test setup

### August 8, 2025
**Accomplished:**
- Daemon implementation (`3588a02`)
- File reorganization (`71304f3`)
- Working on thread messaging (`5c15456`)

**Focus:** Daemon architecture and inter-process communication

### August 7, 2025
**Accomplished:**
- Working on CLI block start functionality (`230c7e7`)

**Focus:** CLI interface development

### August 6, 2025
**Accomplished:**
- Got core functionality working (`76113c3`)
- Manifest updates (`b70bcb4`, `a6d58d2`)
- Testing work (`15d1069`, `72e6d52`)
- Added Linux-specific configuration (`687818b`)
- Created basic integration test (`8ba7c88`)

**Focus:** Core functionality and Linux support

### August 5, 2025
**Accomplished:**
- Removed plist info (`07ad22e`)
- System service preparation (`3df3212`)

**Focus:** System integration cleanup

### August 4, 2025
**Accomplished:**
- Reached working milestone (`34ef796`)
- Got whitelists working (`cd6fccb`)

**Focus:** Whitelist functionality

### August 3, 2025
**Accomplished:**
- Enhanced daemon logic (`792b1e9`)
- Implemented working Unix sockets (`d3cfbc0`)
- Switched to Unix sockets architecture (`ddb31e6`)

**Focus:** IPC mechanism and daemon logic

### August 1, 2025
**Accomplished:**
- JavaScript integration work (`2f554a9`)
- Messages being sent to JavaScript (`a55dd66`)
- Working browser message communication (`4d867aa`)

**Focus:** Browser extension communication

### July 25, 2025
**Accomplished:**
- Working on bridge functionality (`0f219dc`)

**Focus:** Communication bridge development

### July 23, 2025
**Accomplished:**
- Better default settings and handling (`4d8d3bc`)
- General updates (`f6a7f32`)

**Focus:** Configuration and defaults

### July 21, 2025
**Accomplished:**
- Clap parsing working (`80d378d`)
- Binary approach implementation (`302179a`)

**Focus:** CLI argument parsing

### July 20, 2025
**Accomplished:**
- URL sending functionality (`7f28bf8`)
- Code cleanup (`e5afad5`)
- Working prototype (`eb60e82`)
- Initial commit and project setup (`705e6d2`)

**Focus:** Initial development and URL handling

