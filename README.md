# hexegg

Interactive hex editor for linux terminal (and other platforms) with some advanced features.

![hexegg](docs/assets/hexegg.png)  

### Features

- cross-platform, based on crossterm library
- simple custom color schemes with 16, 256, and full RGB support
- multiple input files
- highlight diffs between files
- advanced block manipulation such as insert, delete, fill from generator, from other files, etc ...
- advanced search options
- search for embeded files and known signatures
- results highlighting
- parse some binary structures MZPE, ELF, ..
- simple command interface
- mouse support

Please read the [manual](docs/MANUAL.md) to see all commands and features or [cheat sheet](docs/CheatSheet.md) to see all keyboard shortcuts.

### Install and run

Download and extract the [compiled](https://github.com/mr152here/hexegg/releases) zip file for your platform.

```
cd hexegg
chmod +x hexegg
```

If you want to build latest version from the source, clone git repository (or download sources as a zip file) and compile it with the cargo. You must have installed [rust](https://www.rust-lang.org) programming language to be able to compile it.

```
git clone https://github.com/mr152here/hexegg.git
cd hexegg
cargo build --release
```

Then copy configuration [config.toml](config.toml) file to the same folder where compiled program is located. Or place it to the local config directory $HOME/.config/hexegg/ for linux and %APPDATA%\hexegg\ for windows.

To view/edit files just execute hexegg in terminal and pass file name(s) as arguments.

```
hexegg [-t <size_limit>] <file1> [file2] [file3] ...
```

for example:
```
hexegg notepad.exe
hexegg myfile mypatchedfile
hexegg -t 1000 /dev/random
hexegg -t 1000000 /dev/sda1
```
### Acknowledgment

Hexegg is written in the [rust](https://www.rust-lang.org) programming language using the following libraries:
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [toml](https://github.com/toml-rs/toml)
- [regex-lite](https://docs.rs/regex-lite/latest/regex_lite/)
- [serde](https://serde.rs/)
- [signal-hook](https://github.com/vorner/signal-hook)

### License

Hexegg is licensed under the Apache 2.0 license. See the [LICENSE](LICENSE) file for details.
