# QR Code Studio

QR Code Studio is a client-side QR code generator written in Rust and compiled to WebAssembly. All generation, styling, and scan verification execute inside the browser sandbox. The generator sends zero data over the network.

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
│   ├── qr-frontend  # Leptos 0.7 CSR frontend compiled to WebAssembly
│   └── qr-server    # Axum HTTP server for static asset hosting
├── vercel-build.sh  # Build script for static deployment
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
- [Trunk](https://trunkrs.dev/):
  ```bash
  cargo install trunk
  ```
  or via `cargo binstall trunk`.

## Getting started

### Run the frontend with Trunk

To start the local development server with hot reload:

```bash
cd crates/qr-frontend
trunk serve --port 3000
```

Open `http://localhost:3000` in your browser.

### Run with the Axum server

To compile the frontend bundle and serve it through Axum:

```bash
cd crates/qr-frontend
trunk build --release
cd ../..
cargo run -p qr-server
```

The server binds to `http://127.0.0.1:3000`.

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
