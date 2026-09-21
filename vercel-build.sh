#!/bin/bash
set -euo pipefail

mkdir -p "$HOME/.cargo/bin"
export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Setting up Rust and wasm32 target..."
if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
fi

rustup target add wasm32-unknown-unknown

echo "==> Ensuring wasm-bindgen is installed..."
WASM_BINDGEN_VERSION="0.2.128"
if ! command -v wasm-bindgen &> /dev/null || [ "$(wasm-bindgen --version | awk '{print $2}')" != "${WASM_BINDGEN_VERSION}" ]; then
    echo "Downloading prebuilt wasm-bindgen ${WASM_BINDGEN_VERSION}..."
    curl -sSL "https://github.com/rustwasm/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar -xz --strip-components=1 -C "$HOME/.cargo/bin"
    chmod +x "$HOME/.cargo/bin/wasm-bindgen"
fi

echo "==> Compiling client WASM bundle (release mode)..."
cargo build --release -p qr-frontend --target wasm32-unknown-unknown --features hydrate

echo "==> Generating JS/WASM bindings..."
mkdir -p crates/qr-frontend/dist
wasm-bindgen --target web --out-dir crates/qr-frontend/dist --no-typescript target/wasm32-unknown-unknown/release/qr_frontend.wasm

echo "==> Pre-rendering HTML with Leptos SSR..."
cargo run --release -p qr-frontend --bin prerender --features ssr -- crates/qr-frontend/dist

echo "==> Build complete. Artifacts ready in crates/qr-frontend/dist"
