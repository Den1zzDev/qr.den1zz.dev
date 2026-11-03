# QR Code Studio

QR Code Studio is a QR code generator that runs in the browser. It is written in Rust and compiled to WebAssembly with Leptos. Generation, styling, contrast checks, and export all happen in the tab; the studio sends no payload data over the network.

Live site: [qr.den1zz.dev](https://qr.den1zz.dev)

## What it does

- **Payload types**: links, plain text, Wi-Fi credentials, vCard 3.0 contacts, email drafts, phone numbers, location coordinates, and crypto payment URIs.
- **Styling**: four module shapes, three eye frame shapes, three eye dot shapes, separate colours for modules, eye frame, eye dot, and background, a transparent background, and an optional centre logo. Each palette hex readout copies its value when clicked.
- **Contrast checks**: each coloured region is measured against the background. A region below 3:1 is marked amber in the palette and in the preview.
- **Exports**: PNG at 512, 1024, or 2048 pixels, PNG copied to the clipboard, standalone SVG, and raw SVG copied to the clipboard.
- **Offline use**: a service worker caches the site after the first load, and the page installs as a PWA.
- **Keyboard support**: native radio and file inputs, a keyboard-operable content-type menu, a skip link, and visible focus rings.
- **Mobile quick actions**: below 1024 pixels, a fixed bar shows the current code and the PNG export once the preview scrolls out of view.

The studio checks contrast. It does not decode the generated code in the browser. The Axum server crate exposes a separate `POST /api/verify` endpoint that rasterises and decodes an image with `rqrr`; the static deployment does not call it. Test-scan exported files before a print run.

## Payload details

| Type | Format | Notes |
|---|---|---|
| Link | URL | Adds `https://` when the scheme is missing. Finds and removes `utm_*`, `fbclid`, `gclid`, and `mc_eid` tracking parameters. |
| Text | UTF-8 | Preserves line breaks, emoji, and non-Latin scripts. |
| Wi-Fi | `WIFI:S:...;T:...;P:...;;` | WPA, WEP, and open networks. Supports hidden SSIDs and escapes `\`, `;`, `,`, `:`, and `"`. |
| Contact | vCard 3.0 | Structured name, company, title, phone, email, website, address, and note. |
| Email | `mailto:` | RFC 6068 percent-encoding for the recipient, subject, and multiline body. |
| Phone | `tel:` | RFC 3966, including `;ext=` parameters. |
| Location | `geo:` | RFC 5870 coordinates with an optional place label. |
| Crypto | BIP-21-style URI | Bitcoin, Ethereum, Solana, Monero, and Litecoin. Amounts are normalised without floating-point arithmetic. |

The preview footer summarises Wi-Fi, vCard, email, and crypto payloads instead of echoing them, so a Wi-Fi password or an email body does not appear in plain text. If a coordinate pair or crypto amount is incomplete or invalid, the preview stays empty and the field explains what is missing.

## Workspace layout

```
.
├── crates
│   ├── qr-core      # Payload encoders, QR matrix, SVG renderer, PNG export, contrast, verifier
│   ├── qr-frontend  # Leptos application, pages, and browser helpers
│   └── qr-server    # Axum server with streaming SSR and verification endpoints
├── crates/qr-frontend/input.css       # Tailwind source and the component layer
├── crates/qr-frontend/public          # Icons, manifest, service worker, font, social image
├── vercel-build.sh                    # Static build for Vercel
└── vercel.json                        # Header and routing configuration
```

The site is a single page. One centred column, 45rem at its widest, stacks the payload, preview, shapes, palette, and logo trays in the order the work happens. There is no two-column desktop layout. The code sits in a black matted viewfinder inside the preview tray, on a square stage with a four-module quiet zone, and the stage switches to a checkerboard when the background is transparent. Below 1024 pixels, a fixed quick-action bar keeps the PNG export reachable once the preview scrolls out of view.

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- WebAssembly target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- `wasm-bindgen-cli` 0.2.128. The CLI version must match the `wasm-bindgen` crate in `Cargo.lock`:
  ```bash
  cargo install wasm-bindgen-cli --version 0.2.128
  ```
- Node.js 20 or newer for the Tailwind build.

## Build the static site

1. Compile the stylesheet:
   ```bash
   npx --yes tailwindcss@3 \
     -i crates/qr-frontend/input.css \
     --content "./crates/qr-frontend/src/**/*.rs" \
     --minify \
     -o crates/qr-frontend/dist/style.css
   ```

2. Compile the WebAssembly client:
   ```bash
   cargo build -p qr-frontend --target wasm32-unknown-unknown --features hydrate
   mkdir -p crates/qr-frontend/dist
   wasm-bindgen \
     --target web \
     --out-dir crates/qr-frontend/dist \
     --no-typescript \
     target/wasm32-unknown-unknown/debug/qr_frontend.wasm
   ```

3. Copy the static assets and pre-render the HTML shell:
   ```bash
   cp -r crates/qr-frontend/public/* crates/qr-frontend/dist/
   cargo run -p qr-frontend --bin prerender --features ssr -- crates/qr-frontend/dist
   ```

4. Serve the output:
   ```bash
   python3 -m http.server 4321 --directory crates/qr-frontend/dist
   ```

Open `http://localhost:4321`. Use a local server rather than opening `index.html` directly, because WebAssembly and the clipboard need an HTTP origin.

`./vercel-build.sh` runs the same steps in release mode, then optimises the bundle with `wasm-opt` and minifies the JavaScript with `esbuild`.

## Run the Axum server

The server renders the same application and adds the QR API endpoints. Build the stylesheet and copy the static assets into `dist` first, because the server serves `/style.css` and `/fonts` from there.

```bash
npx --yes tailwindcss@3 \
  -i crates/qr-frontend/input.css \
  --content "./crates/qr-frontend/src/**/*.rs" \
  --minify \
  -o crates/qr-frontend/dist/style.css
cp -r crates/qr-frontend/public/* crates/qr-frontend/dist/

cargo build -p qr-frontend --target wasm32-unknown-unknown --features hydrate
mkdir -p crates/qr-server/pkg
wasm-bindgen \
  --target web \
  --out-dir crates/qr-server/pkg \
  --no-typescript \
  target/wasm32-unknown-unknown/debug/qr_frontend.wasm

cargo run -p qr-server
```

The server listens on `http://127.0.0.1:4321` by default. Set `PORT` to change it, `PKG_DIR` to point at a different client bundle, `STYLE_PATH` to override the stylesheet, and `FONT_DIR` to override the font directory.

Endpoints:

- `GET /health` returns `OK`.
- `POST /api/generate/vector` accepts a payload and render configuration and returns an SVG.
- `POST /api/verify` accepts a base64 image and the expected payload, then returns a verification report.

## Checks

Run the same checks as CI before opening a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --all-targets
```

The core crate has unit tests for URL sanitising, payload escaping, amount normalisation, contrast maths, SVG output, PNG metadata, and scan verification.

## Deployment

The project builds to static files and deploys on Vercel:

- Build command: `./vercel-build.sh`
- Output directory: `crates/qr-frontend/dist`

`vercel.json` sets caching for hashed assets, disables framing, sends `nosniff`, and applies a content security policy that allows the inline bootstrap, the self-hosted font, the WebAssembly module, and local image data only. The service worker uses a network-first strategy with a 2.5-second timeout and falls back to the cache when offline.

## Fonts and assets

The studio self-hosts Maple Mono as a subsetted variable WOFF2. The font is licensed under the SIL Open Font License 1.1, and the licence text ships at `crates/qr-frontend/public/fonts/OFL.txt`.

The icons are inline SVG drawn on a 24-unit grid with a 1.6-unit stroke. The social image and PWA icons are committed under `crates/qr-frontend/public`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the architecture, local workflow, and pull request checklist.

## License

MIT. See [LICENSE](LICENSE).
