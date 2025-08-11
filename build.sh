#!/bin/sh

# Resource Cache Build Script

set -e  # Exit on any error

echo "Building Resource Cache..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first:"
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

# Check if we're using nightly Rust (needed for --out-dir)
if ! rustup show | grep -q "nightly"; then
    echo "Warning: --out-dir flag requires nightly Rust"
    echo "   Installing nightly Rust..."
    rustup install nightly
    rustup default nightly
fi

# Build the project
echo "Building with Cargo..."
cargo build --release --out-dir ./target -Z unstable-options

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary location: ./target/resource-cache"
    echo "Completions: ./target/_resource-cache (zsh), ./target/resource-cache.bash (bash)"
    
    # Show binary info
    if [ -f "./target/resource-cache" ]; then
        echo "Binary size: $(ls -lh ./target/resource-cache | awk '{print $5}')"
        echo "Architecture: $(file ./target/resource-cache | grep -o 'arm64\|x86_64')"
    fi
else
    echo "Build failed!"
    exit 1
fi
