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

echo "==> Ensuring wasm-opt is installed..."
BINARYEN_VERSION="version_122"
if ! command -v wasm-opt &> /dev/null; then
    echo "Downloading prebuilt wasm-opt ${BINARYEN_VERSION}..."
    curl -sSL "https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz" | tar -xz --strip-components=2 -C "$HOME/.cargo/bin" "binaryen-${BINARYEN_VERSION}/bin/wasm-opt"
    chmod +x "$HOME/.cargo/bin/wasm-opt"
fi

echo "==> Preparing dist directory and static assets..."
mkdir -p crates/qr-frontend/dist
if [ -d crates/qr-frontend/public ]; then
    cp -r crates/qr-frontend/public/* crates/qr-frontend/dist/
fi

echo "==> Compiling Ahead-of-Time minified CSS..."
npx -y tailwindcss@3 -c crates/qr-frontend/tailwind.config.js -i crates/qr-frontend/input.css --minify -o crates/qr-frontend/dist/style.css

echo "==> Compiling client WASM bundle (release mode)..."
cargo build --release -p qr-frontend --target wasm32-unknown-unknown --features hydrate

echo "==> Generating JS/WASM bindings..."
wasm-bindgen --target web --out-dir crates/qr-frontend/dist --no-typescript target/wasm32-unknown-unknown/release/qr_frontend.wasm

echo "==> Optimizing WASM binary with wasm-opt -Oz..."
wasm-opt -Oz --all-features crates/qr-frontend/dist/qr_frontend_bg.wasm -o crates/qr-frontend/dist/qr_frontend_bg.wasm

echo "==> Minifying JavaScript bindings with esbuild..."
npx -y esbuild crates/qr-frontend/dist/qr_frontend.js --minify --allow-overwrite --outfile=crates/qr-frontend/dist/qr_frontend.js

echo "==> Pre-rendering HTML with Leptos SSR..."
cargo run --release -p qr-frontend --bin prerender --features ssr -- crates/qr-frontend/dist

echo "==> Build complete. Artifacts ready in crates/qr-frontend/dist"
echo "==> Final bundle summary:"
ls -lh crates/qr-frontend/dist
echo "==> Gzipped sizes:"
for f in crates/qr-frontend/dist/*.wasm crates/qr-frontend/dist/*.js crates/qr-frontend/dist/*.css crates/qr-frontend/dist/*.html; do
    if [ -f "$f" ]; then
        echo "  $(basename "$f"): $(gzip -c "$f" | wc -c) bytes gzipped ($(ls -lh "$f" | awk '{print $5}') uncompressed)"
    fi
done
