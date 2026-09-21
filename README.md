# QR Code Studio

QR Code Studio is a client-side QR code generator written in Rust and compiled to WebAssembly with Leptos. All generation, styling, and scan verification execute inside the browser sandbox. The generator sends zero data over the network.

Live instance: [qr.den1zz.dev](https://qr.den1zz.dev)

## Capabilities

- Payload types: Plain text and URLs.
- Query tracking sanitizer: Inspects URL query strings and strips marketing trackers (`utm_*`, `fbclid`, `gclid`, `mc_eid`, and related tags). This reduces matrix density and prevents tracking parameters from reaching the final code.
- Module and eye styling:
  - Modules: Square, Dots, Rounded, Smooth, and Ente Classy.
  - Eye frames: Square, Rounded, and Circle.
  - Eye dots: Square, Rounded, and Circle.
  - Colors: Independent hex colors for modules, eye frames, eye dots, and background, with transparent background support.
  - Center logo: Embeds custom logos with an automatic upgrade to high error correction to preserve scan reliability.
- Scannability validation:
  - Calculates real-time WCAG contrast ratios between foreground shapes and the background.
  - Verifies matrix scannability using the `rqrr` QR decoder directly against generated output before export.
- Export options:
  - Vector SVG: Download SVG file or copy raw SVG markup to clipboard.
  - Raster PNG: Export rendered codes at 512x512, 1024x1024, or 2048x2048 resolution with embedded metadata.

## To-do

- [ ] Wi-Fi credentials (WPA, WEP, unencrypted networks, hidden SSID)
- [ ] vCard contact cards
- [ ] Email drafts (`mailto:` links with subject and body prefill)

## Workspace layout

The repository is structured as a Cargo workspace:

```
.
├── crates
│   ├── qr-core      # Matrix generation, query sanitizer, SVG renderer, WCAG verifier
│   ├── qr-frontend  # Leptos 0.7 hybrid frontend (WASM hydration + SSR pre-rendering)
│   └── qr-server    # Axum HTTP server with streaming SSR and API endpoints
├── vercel-build.sh  # Build script for static edge deployment
├── vercel.json      # Header and routing configuration for Vercel
├── Cargo.toml       # Workspace root manifest
└── README.md
```

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- WebAssembly target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [`wasm-bindgen-cli`](https://github.com/rustwasm/wasm-bindgen) (`0.2.128`):
  ```bash
  cargo install wasm-bindgen-cli --version 0.2.128
  ```

## Getting started

### Build and serve the static site locally

1. Compile the WebAssembly bundle and pre-render the initial HTML shell:
   ```bash
   # Build WASM client
   cargo build -p qr-frontend --target wasm32-unknown-unknown --features hydrate
   mkdir -p crates/qr-frontend/dist
   wasm-bindgen --target web --out-dir crates/qr-frontend/dist --no-typescript target/wasm32-unknown-unknown/debug/qr_frontend.wasm

   # Pre-render initial HTML shell via Rust SSR
   cargo run -p qr-frontend --bin prerender --features ssr -- crates/qr-frontend/dist
   ```

2. Serve the pre-rendered static output locally on port `4321`:
   ```bash
   python3 -m http.server 4321 --directory crates/qr-frontend/dist
   ```
   Open `http://localhost:4321` in your browser.

### Run with the Axum SSR server

To compile the client assets and run the Axum server locally:

```bash
# Build WASM client into the server's package directory
cargo build -p qr-frontend --target wasm32-unknown-unknown --features hydrate
mkdir -p crates/qr-server/pkg
wasm-bindgen --target web --out-dir crates/qr-server/pkg --no-typescript target/wasm32-unknown-unknown/debug/qr_frontend.wasm

# Run the Axum server (binds to http://127.0.0.1:4321)
cargo run -p qr-server
```

### Run tests

Run the workspace test suite:

```bash
cargo test --workspace
```

## Deployment

The project builds to static files for CDN hosting. The repository includes automated build configuration for Vercel:

- Build command: `./vercel-build.sh`
- Output directory: `crates/qr-frontend/dist`

`vercel.json` configures caching headers for immutable wasm and js assets and sets security headers (`nosniff`, `DENY` framing).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for code structure, guidelines, and pull request procedures.

## License

This project is licensed under the [MIT License](LICENSE).
