# QR Code Studio (`qr.den1zz.dev`)

> Pure-Rust, client-side QR studio with Ente classy modules, URL query tracking sanitizer, and high-resolution exports. Runs entirely locally in WebAssembly with zero network tracking.

## Highlights

- **Pure Rust & WebAssembly**: Fast, reactive client-side engine powered by Leptos 0.7 CSR and compiled with Trunk. Zero latency, instant preview updates.
- **Privacy First & Query Cleaner**: Automatic URL query inspection and stripping of marketing tracking parameters (`utm_*`, `fbclid`, `gclid`, `ref`, etc.) to keep matrix density low and protect recipient privacy.
- **Vector Styling**:
  - Module shapes: Ente Classy, Smooth, Dots, Square.
  - Eye frame shapes: Rounded, Circle, Square.
  - Eye dot shapes: Circle, Rounded, Square.
  - Granular color controls: Module color, Eye Frame color, Eye Dot color, and background color or transparent mode.
  - Optional center logo punchout with automatic error correction upgrade.
- **Scannability Assurance**: Live WCAG contrast ratio calculations for module, eye frame, and eye dot against the background.
- **High-Resolution Exports**: Download pure SVG, copy raw SVG to clipboard, or export PNG at 512px, 1024px, and 2048px resolutions.
- **Zero Network Calls**: All generation happens directly in the browser's WebAssembly sandbox.

---

## Workspace Structure

```
.
├── crates
│   ├── qr-core      # Shared library: QR matrix generation, vector SVG renderer, payload sanitizer, contrast verifier
│   ├── qr-frontend  # Leptos CSR WebAssembly application built with Trunk
│   └── qr-server    # Optional Axum HTTP service for serving static assets and optional API endpoints
├── Cargo.toml       # Cargo workspace configuration
├── vercel.json      # Vercel deployment configuration
├── vercel-build.sh  # Automated Vercel build script
└── README.md
```

---

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk` (or `cargo binstall trunk`)

### Development

1. **Build the Web Frontend**:
   ```bash
   cd crates/qr-frontend
   trunk build --release
   cd ../..
   ```

2. **Run the Axum Server**:
   ```bash
   cargo run -p qr-server
   ```
   The application will be live at `http://localhost:3000`.

3. **Or run with Trunk serve directly**:
   ```bash
   cd crates/qr-frontend
   trunk serve --port 3000
   ```

4. **Run Tests**:
   ```bash
   cargo test --workspace
   ```

---

## Deploy to Vercel

The project is configured for one-click static deployment on Vercel:

1. Import the repository in Vercel.
2. The root `vercel.json` and `vercel-build.sh` will automatically configure:
   - **Build Command**: `./vercel-build.sh`
   - **Output Directory**: `crates/qr-frontend/dist`
3. All assets are compiled to static HTML/Wasm and served via Vercel Edge CDN with caching and security headers.

---

## License

MIT
