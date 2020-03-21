# crate_downloader
Download rust crates for offline development.

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

### Run
```bash
RUST_LOG=info cargo run -- -i /path/to/local/crates.io-index -d /path/to/local/crates
```

### Build and Run
```bash
cargo build --release

RUST_LOG=info ./target/release/crate_downloader -i /path/to/local/crates.io-index -d /path/to/local/crates
```
