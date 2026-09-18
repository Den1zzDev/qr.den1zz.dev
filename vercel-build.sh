#!/bin/bash
set -euo pipefail

echo "==> Setting up Rust and wasm32 target..."
if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    export PATH="$HOME/.cargo/bin:$PATH"
fi

rustup target add wasm32-unknown-unknown

echo "==> Ensuring Trunk is installed..."
if ! command -v trunk &> /dev/null; then
    TRUNK_VERSION="0.21.14"
    echo "Downloading prebuilt Trunk ${TRUNK_VERSION} binary..."
    curl -sSL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xz -C "$HOME/.cargo/bin"
    chmod +x "$HOME/.cargo/bin/trunk"
fi

echo "==> Building frontend with Trunk (release mode)..."
cd crates/qr-frontend
trunk build --release

echo "==> Build complete. Artifacts ready in crates/qr-frontend/dist"
