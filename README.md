# hexegg

Cross-platform terminal hex viewer and editor with some advanced features.

### Features

- cross-platform, based on crossterm library
- simple custom color schemes with 16, 256, and full RGB support
- multiple input files
- highlight diffs between files
- toggle printable characters
- advanced block manipulation such as insert, delete, fill from other files, etc ...
- advanced search options
- results highlighting
- command interface
- minimal external dependencies

Please read the [manual](docs/MANUAL.md) to see all its commands and features.

### Install

Download and extract the [compiled](releases/) zip file for your platform. Or you can download the source code and build it with rust cargo:
```
cargo build --release
```

Then copy configuration [config.toml](config.toml) file to the same folder where compiled program is located.

### Acknowledgment

Hexegg is written in the [rust](https://www.rust-lang.org) programming language using the following libraries:
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [toml](https://github.com/toml-rs/toml)
- [serde](https://serde.rs/)

### License

Hexegg is licensed under the Apache 2.0 license. See the [LICENSE](LICENSE) file for details.
