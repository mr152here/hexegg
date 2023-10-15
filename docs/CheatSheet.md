# hexegg keyboard shortcuts

##### navigation:
- UP, DOWN: one line up / down
- LEFT, RIGHT: one byte up / down
- SHIFT + UP, DOWN: 8 lines up / down
- SHIFT + LEFT, RIGHT: move 8 bytes up / down
- PGUP, PGDN: move screen size up/down
- HOME: start of the file
- END: end of the file
- TAB: cycle through opened file buffers
- ENTER: cycle through screens
- . : move to the next diff (if multiple files are opened)
- , : move to the next patch
- q : quit program
- 0 .. 9 : jump to the set bookmark

#### UI features:
- l : toggle location bar
- o : toggle offset bar
- i : toggle info bar
- h : toggle highlighting of diffs
- p : show / hide non-printable characters
- k : lock file/cursor position to all file buffers
- m : highlight selected block
- M : unhighlight selected block
- \ : repeat last command
- / : show command input line
- \+, - : increase / decrease size of the main window
- ESC: cancel selection, switch to the normal mode with hidden cursor or quit program (if specified in config file)

#### Command input interface:
- ESC: cancel and close command line
- TAB: complete / show possible commands
- ENTER: execute command
- UP / DOWN: cycle through commands history

#### Location Bar:
- [, ]: move to the next / previous item in the location bar
- {, }: move page up / down in the location bar
- \> : find highlighted block in the location bar
- < : go to the items offset from the location bar
- R : remove current item from the location bar
- r : rename current item in the location bar

#### Editing and modes:
- n : normal mode where cursor is visible
- t : text mode where you can edit file directly as a text
- b : binary mode to edit file as hex bytes only [0-9a-f] are allowed
- s : start / end selection of the block
- S : select entire ascii string at the cursor position
- U : select entire unicode string at the cursor position
- H : select entire highlighted block at the cursor position
- y : yank block to the internal register (and send it via STDIN to external application if specified in config file. E.g. to system clipboard)
- BACKSPACE: revert patch at the offset before the cursor and move 1 byte up
- DEL: revert patch at the cursor position

#### Signals:
- CTRL + c: quit program, abort all changes
- CTRL + z: suspend program to background (unix only)
