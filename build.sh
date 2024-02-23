#!/bin/bash
BUILDS_DIR="builds"

# exit on failure
set -e

# create builds folder if missing
if [ ! -d "$BUILDS_DIR" ]; then
    mkdir "$BUILDS_DIR"
fi

export RUSTFLAGS='-C link-arg=-s'

# Build the linux binary
echo "Building Linux binary..."
cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/toolbelt "$BUILDS_DIR/toolbelt-linux_x86_64"

echo "Building Windows binary..."
# Build the windows binary
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/toolbelt.exe "$BUILDS_DIR/toolbelt-windows_x86_64.exe"