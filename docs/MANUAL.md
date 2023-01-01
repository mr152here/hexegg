# Manual

The program open and read all input files specified by their names as command-line arguments.

```
hexegg <file1> [file2] [file3] ...
```

Each file is fully read into the memory, and takes there approximately its own size. File size (and memory consumption) is not limited, so please think twice before you open a large files or streams with undefined size like /dev/random.

## Basic controls

Basic controls are pretty intuitive. The arrow keys move the file view offset or move the cursor if is visible (with SHIFT it moves 8x faster):
- UP - one line up
- DOWN - one line down
- LEFT - one byte up
- RIGHT - one byte down
- PGUP - one page up
- PGDOWN - one page down
- HOME - beginning of the file
- END - one page before the end of the file
- ENTER - cycle through the screens
- TAB - cycle through the opened files

#### Program modes

The program can be in one of three modes. The default mode is "view mode". In this mode, it is not possible to directly modify file content. And is intended to be used for preview, inspection, search, ...
The second mode is "byte mode". The cursor is shown, and every 0-9, a-f, A-F key modifies the byte at the cursor position, and this modification is stored as a patch (which you can save to file later). All other keys work the same as in view mode. The third mode is "text mode". In this mode, every printable character modifies a byte (creates a patch) under the cursor location. Intended for string modification. Because many keyboard shortcuts are letters, most of them are not available in this mode.

Switching program modes:
- 'b' - toggles the view mode to byte mode
- 't' - toggles the view or byte mode to text mode
- ESC - return to view mode from byte or text mode

If the program is in byte mode, you may also select a block of bytes. To select some bytes, start by pressing the 's' key. Selection starts from the current cursor position, adjust it to the required size with standard movement keys, and press the 's' again to finish the selection (or you can cancel it by ESC). The mouse is not supported yet. 

#### Patches

Almost every file modification is stored in the "patch map", which contains the original byte from the file and its new value. You can restore any byte to its original state in B/T mode by pressing BACKSPACE or DEL. Just like in every standard text editor. When you extend a file or insert a new blocks, their original bytes are undefined and are arbitrary set to 0. When you delete part of the file, it can't be restored this way. There is no other "undo" command.

#### More controls

- 'q' - in the view mode will quit the program. If the file is modified, you will be asked whether to save it
- 'h' - toggles between highlighting different bytes
- 'p' - show / hide non-printable characters (nice when looking for strings in binary blobs)
- 'k' - toggle the file buffer offset lock. When you change the offset in one file, it will be set in all other files (if lock is enabled)
- '.' - locate the next diff.
- ',' - locate the next patch.
- '+' - increase row size (nice when looking for some symetry or tables)
- '-' - decrease row size (nice when looking for some symetry or tables)
- 'i' - toggles the visibility of the info bar
- 'o' - toggles the visibility of the offset bar

#### Location bar

Whenever you perform an operation that may returns multiple results, the results are displayed on the right side of the screen in a panel called the location bar. You can quickly jump through results and their associated offsets using:
- '[' - navigate to the previous position from location bar
- ']' - navigate to the next position from location bar
- '=' - navigate to the current position from location bar
- 'l' - toggles location bar visibility (doesn't affect shortcuts)

#### Info bar
![info_bar](info_bar.png)  

The info bar is at the top of the screen. There is information about the current file and program state. On the left is the current file index and the number of opened files. The current file name, where '+' indicates if the file has been modified. Next is the current position in the file in decimal, hexadecimal, and percentage representation. The following four letters indict:
> [V|B|T] - view, binary or text mode  
> [P|-] - only printable characters  
> [L|-] - file buffer offset lock  
> [D|-] - diff highlighting

Follows number of results in location bar. The 'i' key is used to toggle the visibility of the info bar.

## Commands

Hexegg has a build-in command interface with a simple history of last used commands. All non-basic functions are accessible through it.
- '/' - display the command prompt interface
- ESC - cancel command
- ENTER - execute command
- UP - select a previous command from history
- DOWN - select a next command from history  
- BACKSPACE - delete last character from the command

Next is a list of available commands, their parameters, and their descriptions. The parameters in [] are optional, the parameters in <> are mandatory, and the parameters in {} have a predefined default value. All parameters must be specified in order. If the {default} parameter is in front of another non-default parameter, it cannot be skipped.  Some commands also have shorthand notation.

#### Command list

*quit* | *q*
> closes all files; for each modified file, asks whether to save it and then exits.

*quit!* | *q!*
> abort all changes and quit. 

*goto \<position\>* | *g \<position\>*
> go to the file *position*. Specified as a decimal or hexadecimal value. If *position* starts with + or - sign, then is interpreted as relative value from the current position. 
> 
> 'g 10' - go to the file offset 10  
> 'g xBEEF' - go to the file offset 48879  
> 'g -x10' - go 16 bytes back from the current position  
> 'g +50' - go 50 bytes forward

*findallpatches* | *fap*
> find all existing patches and show their offsets in the location bar. 

*findalldiffs* | *fad*
> find all diffs (different bytes among all opened files) and show their offsets in the location bar.

*find \[pattern\]* | *f \[pattern\]*
> find and go to the next position of the *pattern* from the current position. *pattern* can mix of printable and escaped hexadecimal characters. If no pattern is specified, bytes from the currently selected block are used instead.
> 
> 'find hello' - find next position of 'hello'  
> 'f abc\\def' - find next position of 'abc\def'  
> 'f need\x20\xCA\xFE',  
> 'f \x0\x01\x2\x3\xF\x0F'

*findall \[pattern\]* | *f \[pattern\]*
> find and show all positions of the *pattern* (or selected block if not specified) in the location bar. See *find* command for *pattern* syntax.

*findhex \<bytes\>* | *fx \<bytes\>*
> wrapper to the *find* command. Works the same way but is faster to type when you are searching only for hexadecimal characters. *bytes* must be space separated.
> 
> 'findhex C0 01 5E ED'

*findallhex \<bytes\>* | *fax \<bytes\>*
> wrapper to the *findall* command with *findhex* syntax.

*findstring \{min_size = 5\} \[substring\]* | *fs \{min_size = 5\} \[substring\]*
> find and jump to the beginning of the next string that is at least *min_size* long and (if specified) contains a *substring*. If the size of *substring* is greater than *min_size*, that size is used instead. A string is a sequence of printable characters that ends with any non-printable character.
> 
> 'fs' - find the next string that is at least 5 bytes long.  
> 'fs 10' - find the next string that is at least 10 bytes long.  
> 'fs 20 hexedit' - find the next string that is at least 20 bytes in size and contains 'hexedit'  
> 'fs 4 hexedit' - find the next string that is at least 7 bytes in the size and contains 'hexedit' 

*findallstrings \{min_size = 5\} \[substring\]* | *fas \{min_size = 5\} \[substring\]*
> find all strings and show their first 8 bytes as a preview in the location bar. Works the same as *findstring* but locates all strings, not just the next one.

*deleteblock*
> delete the selected block from the file

*saveblock \<filename\>*
> save the selected block into the file with *filename*.

*fillblock \[bytes\]*
> fill the selected block with *bytes*. If the number of *bytes* is less than the block size, *bytes* will be repeated. *bytes* must be specified as hexadecimal values, and when not specified, a block will be filled with 0.
> 
> 'fillblock' - fill the selected block with 0.  
> 'fillblock FF' - fill the selected block with 0xFF.  
> 'fillblock 10 20 30' - fill the selected block with bytes 0x10, 0x20, 0x30, 0x10, 0x20, 0x30, 0x10, 0x20, 0x30, ...  

*insertfilledblock \<size\> \[bytes\]*
> create and insert a filled block at the cursor position. *size* is the requested size of the block and may be in decimal or hexadecimal notation. *bytes* is a list of bytes, as in the *fillblock* command. If not specified, it will create a block filled with 0.
> 
> 'insertfilledblock x10' - will create and insert a block of size 16 filled with 0x00  
> 'insertfilledblock 7 20 30 40' - will create and insert block of size 7 and filled with 0x20, 0x30, 0x40, 0x20, 0x30, 0x40, 0x20

*appendfilledblock \<size\> \[bytes\]*
> same as the *insertfilledblock* but will create and insert block after the cursor position.

*insertfile \<filename\>*
> load and insert *filename*'s content at the cursor position.

*appendfile \<filename\>*
> load and insert *filename*'s content after the cursor position.

*openfile \<filename\>*
> open *filename* and load its content into a new file buffer. You may cycle through all opened files with the TAB key.

*closefile*
> close the current file. If the file is modified, you will be asked whether to save the changes or not. If there are no more opened files, the program will quit.

*savefile \[filename\]*
> save the current file buffer with all its modifications as a *filename*. If no *filename* is specified, the file is saved under its own name.

*entropy \{block_size = 1024}\ \{margin = 1.1\}* | *ent \{block_size = 1024}\ \{margin = 1.1\}*
> split file into blocks of *block_size* size, calculate the entropy of each one, and if the difference between two following blocks is less then the *margin*, join them together. Entropy values of resulting blocks are in the location bar. The associated offset is starting offset of the block. For the wisely chosen parameters, command can be used to navigate quickly through the interesting areas.

*clearlocationbar*
> clear all results from the location bar, including all highlights.  

*set \<variable_name\> \<variable_value\>*
> set values of inner variables in the running program. *variable_name* must be one from 'colorscheme', 'higlightstyle', 'screenpagingsize'. *variable_value* is a variable name specific and can be found in the configuration file config.toml.


