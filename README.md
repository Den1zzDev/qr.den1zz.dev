# QR Code Studio (`qr.den1zz.dev`)

> Pure-Rust QR engine unifying advanced vector custom styling, binary image halftoning, and built-in scan verification.

## Highlights

- **Unified Suite**: Integrates both vector graphic design and photo halftoning into a single fullstack Rust codebase.
- **Client-Side Reactive Wasm**: Interactive preview powered by Leptos 0.7 compiled to WebAssembly. Zero latency, instant feedback.
- **Full Backend API**: Axum-based server with endpoints for vector generation, photo halftoning, and scan verification.
- **Built-in `rqrr` Verifier**: Every generated QR code is verified for machine-readability before export, with contrast advisory metrics.
- **Privacy First**: Strips tracking parameters (`utm_*`, `fbclid`, `gclid`, etc.) before encoding, shrinking matrix density.
- **Vector Styling**:
  - Module shapes: Square, Smooth, Dots, Classy.
  - Eye shapes: Square, Rounded, Circle frames and inner dots.
  - Linear and Radial gradients.
  - Center logo punchout with automatic module collision avoidance.
  - Call-to-action text badges (SCAN ME, WIFI, MENU, custom).
- **Photo Halftoning**:
  - Color, Sampled, and B/W dither modes.
  - Dithering kernels: Clustered dot, Floyd-Steinberg, Bayer 4×4, Bayer 8×8.
  - CLAHE (Contrast Limited Adaptive Histogram Equalization).
  - Protected structural cell masks (finders, timing tracks, alignment markers).
- **High-Resolution Exports**: SVG vector downloads, PNG exports up to 4096×4096, and PNG `tEXt` alt-text accessibility metadata.

---

## Workspace Structure

```
.
├── crates
│   ├── qr-core      # Shared library: QR matrix, vector renderer, halftoner, verifier, PNG exporter
│   ├── qr-server    # Axum HTTP service and static file server
│   └── qr-frontend  # Leptos CSR WebAssembly application built with Trunk
├── Cargo.toml       # Cargo workspace configuration
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

3. **Run Tests**:
   ```bash
   cargo test --workspace
   ```

---

## API Reference

### `POST /api/generate/vector`
Generates an SVG string for a given payload and vector styling configuration.

### `POST /api/generate/halftone`
Processes an uploaded image with error-corrected halftoning, returning an SVG, base64 PNG, detected palette, and `rqrr` verification status.

### `POST /api/verify`
Scans a rendered QR code image, reports scan status (Verified, Uncertain, Unscannable), decoded content, and contrast ratio.

---

## License

MIT
