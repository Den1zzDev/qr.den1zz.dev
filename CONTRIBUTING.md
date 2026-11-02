# Contributing to QR Code Studio

Thank you for your interest in contributing to QR Code Studio.

## Workspace architecture

This repository is organized as a Cargo workspace with three crates:

- `crates/qr-core`: Core library containing matrix calculation, URL tracking sanitizer, vector SVG rendering, WCAG contrast verification, and PNG rasterization.
- `crates/qr-frontend`: Leptos WebAssembly application supporting client-side hydration (`hydrate`) and compile-time pre-rendering (`ssr`).
- `crates/qr-server`: Axum HTTP server providing Leptos streaming SSR and QR processing endpoints.

## Prerequisites

You need the following tools installed:

- Rust stable toolchain: install via [rustup](https://rustup.rs/)
- WebAssembly compilation target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- `wasm-bindgen-cli` (v0.2.128):
  ```bash
  cargo install wasm-bindgen-cli --version 0.2.128
  ```

## Local development

### Building the WebAssembly frontend

To compile the frontend and generate the pre-rendered static site:

```bash
cargo build -p qr-frontend --target wasm32-unknown-unknown --features hydrate
mkdir -p crates/qr-frontend/dist
wasm-bindgen --target web --out-dir crates/qr-frontend/dist --no-typescript target/wasm32-unknown-unknown/debug/qr_frontend.wasm
cargo run -p qr-frontend --bin prerender --features ssr -- crates/qr-frontend/dist
```

You can serve `crates/qr-frontend/dist` with any static web server:

```bash
python3 -m http.server 4321 --directory crates/qr-frontend/dist
```

### Running the Axum server

To run the Axum server with streaming SSR and static client assets:

```bash
cargo build -p qr-frontend --target wasm32-unknown-unknown --features hydrate
mkdir -p crates/qr-server/pkg
wasm-bindgen --target web --out-dir crates/qr-server/pkg --no-typescript target/wasm32-unknown-unknown/debug/qr_frontend.wasm
cargo run -p qr-server
```

The server listens on `http://127.0.0.1:4321`.

## Testing

Run the workspace test suite before submitting a pull request:

```bash
cargo test --workspace
```

Ensure all workspace crates compile without errors or warnings:

```bash
cargo check --workspace --all-targets
```

## Code style

The workspace follows the [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html).
Formatting is configured in [`rustfmt.toml`](rustfmt.toml) with a 100-column limit.

Before submitting a pull request, verify formatting, lints, and tests:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Conventions worth knowing:

- Public items in `qr-core` carry doc comments; the crate enables the `missing_docs` lint.
- Types are imported directly, while free functions are called through their module
  (for example `vector::render_svg`).
- Fallible constructors are named `try_new`.
- Public enums and metadata structs are `#[non_exhaustive]`.
- `unsafe` code is forbidden across the workspace.

## Pull request guidelines

1. Fork the repository and create a branch from `main`.
2. Keep pull requests focused on a single feature or fix.
3. Add unit tests for any new logic in `crates/qr-core`.
4. Fill out the pull request template with details on your changes and manual testing.
