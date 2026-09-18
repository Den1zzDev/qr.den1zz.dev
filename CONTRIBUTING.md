# Contributing to QR Code Studio

Thank you for your interest in contributing to QR Code Studio.

## Workspace architecture

This repository is organized as a Cargo workspace with three crates:

- `crates/qr-core`: Core library containing matrix calculation, URL tracking sanitizer, vector SVG rendering, WCAG contrast verification, and PNG rasterization.
- `crates/qr-frontend`: Leptos client-side WebAssembly interface built with Trunk. Runs entirely in the browser sandbox.
- `crates/qr-server`: Optional Axum HTTP server to serve static assets locally.

## Prerequisites

You need the following tools installed:

- Rust stable toolchain: install via [rustup](https://rustup.rs/)
- WebAssembly compilation target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- Trunk build tool:
  ```bash
  cargo install trunk
  ```
  or prebuilt binary via `cargo binstall trunk`.

## Local development

### Running the WebAssembly frontend

Run Trunk directly inside the frontend directory:

```bash
cd crates/qr-frontend
trunk serve --port 3000
```

Open `http://localhost:3000` in your browser. Trunk will watch for changes and recompile the WebAssembly application automatically.

### Running the optional Axum server

To run the Axum server alongside static output:

```bash
cd crates/qr-frontend
trunk build --release
cd ../..
cargo run -p qr-server
```

The server listens on `http://127.0.0.1:3000`.

## Testing

Run the workspace test suite before submitting a pull request:

```bash
cargo test --workspace
```

Ensure all workspace crates compile without errors or warnings:

```bash
cargo check --workspace --all-targets
```

## Pull request guidelines

1. Fork the repository and create a branch from `main`.
2. Keep pull requests focused on a single feature or fix.
3. Add unit tests for any new logic in `crates/qr-core`.
4. Fill out the pull request template with details on your changes and manual testing.
