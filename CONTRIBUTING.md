# Contributing to QR Code Studio

## Workspace architecture

The repository is a Cargo workspace with three crates:

- `crates/qr-core`: payload encoders, QR matrix construction, SVG rendering, PNG export, contrast maths, and scan verification. Public items carry doc comments and the crate denies missing docs.
- `crates/qr-frontend`: the Leptos application. `src/app.rs` owns the page shell and memo graph, `src/studio_state.rs` owns the reactive studio state, `src/components/` holds one file per panel plus the mobile quick-action bar, and `src/browser.rs` isolates clipboard, canvas, and download APIs.
- `crates/qr-server`: an Axum server with streaming SSR and JSON endpoints for vector generation and image verification. The deployed site is static and does not use it.

The frontend has three feature sets:

- `hydrate`: browser bundle that takes over the pre-rendered HTML.
- `ssr`: server and pre-renderer build.
- `csr`: client-only mount for local experiments.

## Prerequisites

- Rust stable toolchain from [rustup](https://rustup.rs/)
- The WebAssembly target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- `wasm-bindgen-cli` 0.2.128. The CLI and the `wasm-bindgen` crate in `Cargo.lock` must match exactly:
  ```bash
  cargo install wasm-bindgen-cli --version 0.2.128
  ```
- Node.js 20 or newer for Tailwind and the bundle minifier.

## Local development

### Build the static site

```bash
npx --yes tailwindcss@3 \
  -i crates/qr-frontend/input.css \
  --content "./crates/qr-frontend/src/**/*.rs" \
  --minify \
  -o crates/qr-frontend/dist/style.css

cargo build -p qr-frontend --target wasm32-unknown-unknown --features hydrate
mkdir -p crates/qr-frontend/dist
wasm-bindgen \
  --target web \
  --out-dir crates/qr-frontend/dist \
  --no-typescript \
  target/wasm32-unknown-unknown/debug/qr_frontend.wasm

cp -r crates/qr-frontend/public/* crates/qr-frontend/dist/
cargo run -p qr-frontend --bin prerender --features ssr -- crates/qr-frontend/dist
python3 -m http.server 4321 --directory crates/qr-frontend/dist
```

Open `http://localhost:4321`. The page must be served over HTTP, not opened as a file, because WebAssembly streaming and the clipboard require an origin.

### Run the Axum server

Build the stylesheet and copy the static assets first. The server serves `/style.css` and `/fonts` from `crates/qr-frontend/dist`.

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

The server listens on port 4321 by default. Set `PORT`, `PKG_DIR`, `STYLE_PATH`, or `FONT_DIR` to override the defaults. The server warns at startup when the stylesheet is missing.

## Checks

Run these before opening a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI also builds the WebAssembly bundle and pre-renders the page, so a change that only compiles for the host target still passes `cargo check` but fails CI. Build the wasm target locally when you touch the frontend.

## Code style

The workspace follows the [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html). Formatting is configured in [`rustfmt.toml`](rustfmt.toml) with a 100-column limit.

Conventions worth knowing:

- Public items in `qr-core` carry doc comments, and the crate enables the `missing_docs` lint.
- Import types directly and call free functions through their module, for example `vector::render_svg`.
- Name fallible constructors `try_new`.
- Mark public enums and metadata structs `#[non_exhaustive]`.
- `unsafe` code is forbidden across the workspace.
- Keep the browser APIs in `src/browser.rs`; do not scatter `web_sys` calls through components.
- Keep panel-level state in `StudioState` and derived values in memos in `app.rs`.

## Design system

The visual system is documented in [DESIGN.md](DESIGN.md), with the product record in [PRODUCT.md](PRODUCT.md). Read both before changing the interface.

The short version: AMOLED black, one cyan signal colour, a single monospace family, opaque trays with hairline borders, and amber for a failed measurement.

One centred column, 45rem at its widest, stacks the payload, preview, shapes, palette, and logo trays in that order. There is no desktop two-column split. Cyan marks the wordmark, the selected option, the selected export size, the primary export, links, and keyboard focus; a selected control is a cyan fill with black text. Below 1024 pixels, a fixed quick-action bar takes over once the preview scrolls out of view.

Controls are pill buttons, 10-pixel fields, 32-pixel segments and swatches, and 16-pixel trays. The code sits in a black matted viewfinder with a four-module quiet zone. Every styled control must show its cost, whether that is a contrast ratio, a byte count, or an error message.

When you add a control:

- Use a native input where one exists. Style it rather than rebuilding it.
- Give it a label, a focus state, and a disabled state.
- Use the `--line-control` border token, which clears the 3:1 non-text contrast floor.
- Keep touch targets at least 24 CSS pixels in both axes.
- Respect `prefers-reduced-motion`. The preview's scan pass is decorative and must disappear under that preference.
- Keep copy confirmations in the shared notice line rather than adding a second toast or banner.

## Assets

- The Maple Mono WOFF2 is subsetted to Latin, punctuation, arrows, and the symbols used by the UI. The licence ships beside it at `crates/qr-frontend/public/fonts/OFL.txt`.
- Icons live in `crates/qr-frontend/src/components/icons.rs` and use a 24-unit view box with a 1.6-unit stroke.
- PWA icons and the social image are committed under `crates/qr-frontend/public`. Regenerate them when the wordmark or the default code changes.

## Pull request guidelines

1. Branch from `main` and keep the change focused.
2. Add unit tests for new logic in `qr-core`.
3. Test the interface at 320, 390, 768, 1024, and 1440 pixels wide.
4. Check keyboard navigation and the generated PNG and SVG before submitting.
5. Fill out the pull request template with the payload types and controls you tested.
