# QR Code Studio

QR Code Studio is a client-side QR code generator written in Rust and compiled to WebAssembly with Leptos. All generation, styling, and scan verification execute inside the browser sandbox. The generator sends zero data over the network.

Live instance: [qr.den1zz.dev](https://qr.den1zz.dev)

## Capabilities

- **Payload Types**:
  - **URL / Link**: Standard hyperlinks with optional tracking parameter stripping (`utm_*`, `fbclid`, `gclid`, `mc_eid`, etc.).
  - **Plain Text**: Arbitrary raw text, notes, and messages without escaping.
  - **Wi-Fi Credentials**: Standards-compliant ZXing format (`WIFI:S:...;T:WPA;P:...;;`) with support for WPA/WPA2/WPA3, WEP, unencrypted networks (`nopass`), hidden SSIDs, and automatic character escaping (`\`, `;`, `,`, `:`, `"`).
  - **vCard 3.0 / 4.0 Contact Cards**: Full contact cards with structured name, company, job title, phone numbers (mobile/work), email, website, address, notes, and standard semicolon/newline escaping.
  - **Email Drafts**: RFC 6068 `mailto:` links with recipient address, subject line, and multiline prefilled body with RFC 3986 percent-encoding.
  - **Phone Numbers**: RFC 3966 `tel:` links compatible with native mobile and desktop phone dialers, preserving phone extension parameters (`;ext=`).
  - **Geolocation Coordinates**: RFC 5870 `geo:` URI standard compatible with Apple Maps and Google Maps, with support for bounded latitude, longitude, and optional search/place labels (`?q=...`).
  - **Cryptocurrency Payment URIs**: BIP-21 standard URI schemes for Bitcoin (`bitcoin:`), Ethereum (`ethereum:`), Solana (`solana:`), and Litecoin (`litecoin:`) with address, safe float-free normalized amount strings, and optional label/memo.
- **Module and Eye Styling**:
  - Modules: Square (with contiguous horizontal run merging), Dots, Rounded, Smooth, and Ente Classy.
  - Eye frames: Square, Rounded, and Circle.
  - Eye dots: Square, Rounded, and Circle.
  - Colors: Independent hex colors for modules, eye frames, eye dots, and background, with transparent background support.
  - Center logo: Embeds custom PNG/SVG logos with automatic error correction boost to Level H.
- **Scannability Validation**:
  - Real-time WCAG contrast ratio calculations for module, eye frame, and eye dot against background.
  - In-browser scan verifier using the `rqrr` engine directly against generated matrices before export.
- **Export Options**:
  - Raster PNG: 512×512, 1024×1024, or 2048×2048 resolution with embedded tEXt metadata chunks.
  - Direct PNG Clipboard Copy: Clipboard copy via Async Clipboard API (`navigator.clipboard.write`).
  - Vector SVG: Standalone SVG download or raw SVG markup clipboard copy.
- **Offline PWA**:
  - Service Worker (`sw.js`) with 2500ms network-first fallback and Web Application Manifest (`manifest.webmanifest`) for offline functionality and desktop/mobile installation.

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
