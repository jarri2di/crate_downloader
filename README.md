# crate_downloader
Download rust crates for offline development.

### Overview
This is a small app that downloads Rust crates for offline development.
- Uses [reqwest](https://github.com/request/request) for http client requests
- Uses [structopt](https://github.com/TeXitoi/structopt) to define and parse command line arguments
- Downloads are streamed via concurrent, asynchronous requests (via futures)

### Setup
```bash
# Clone this repo
git clone https://github.com/jarri2di/crate_downloader.git

# Clone the crates index repo
git clone https://github.com/rust-lang/crates.io-index.git

# Create a directory for downloaded crates
mkdir -p /path/to/local/crates
```

### Usage
```bash
cargo run -- --help
```

### Check and lint
Install [clippy](https://github.com/rust-lang/rust-clippy#usage).

```bash
cargo check

cargo clippy
```

### Run
```bash
RUST_LOG=info cargo run -- -i /path/to/local/crates.io-index -d /path/to/local/crates
```

### Build
```bash
cargo build --release
```

### Run executable
```bash
RUST_LOG=info ./target/release/crate_downloader -i /path/to/local/crates.io-index -d /path/to/local/crates
```