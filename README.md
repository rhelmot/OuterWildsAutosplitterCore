# Outer Wilds Autosplitter

# Build
1. Get Rust from https://rustup.rs.
2. Add wasm32 target with `rustup target add wasm32-unknown-unknown`.
3. Build with `cargo build --target wasm32-unknown-unknown --release`.

# Use

AFAIK the only way to use this for real is to use the LiveSplitOne OBS plugin.
Install that, then in its settings, check "use local autosplitter" and navigate to the .wasm file that is produced when you build.
Usually, this will be in `target/wasm32-unknown-unknown/release/outer_wilds_autosplitter.wasm`.
