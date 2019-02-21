# bender_config

bender_config is a rust library, that deals with reading, writing and creating \
the config for the bender renderfarm. It consists of two parts:
- the rust library
- a CLI tool for creating and managing the config

It can be loaded into a rust project via its git repository by putting this in your Cargo.toml:
```rust
[dependencies]
bender_config = { git = "ssh://git@code.hfbk.net:4242/bendercode/bender-config.git"}
```
To update this run
```rust
cargo clean
cargo update
```

### Testing
The libary is implemented with a extensive amount of tests to make
sure that repeated deserialization/serialization won't introduce
losses or glitches to the config file. The tests can be run with
```rust
cargo test
```

### Documentation
If you want to view the documentation run
```rust
cargo doc --no-deps --open
```

### Installation
To run cargo, make sure you have rust installed. Go to [rustup.rs](http://rustup.rs) and follow the instructions there
To install the CLI tool `bender-config` just execute `./install.sh` for a guided setup

License: MIT
