# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Developers, designers, and small teams who need a styled QR code for a link, Wi-Fi network, contact card, email draft, phone number, location, or crypto payment. They usually arrive with the payload already in hand and want a file they can print or paste into a document without creating an account.

## Product Purpose

qr.den1zz.dev is a QR code studio that runs entirely in the browser. It turns a payload into a styled code, checks the colour contrast of every code region, and exports a PNG or SVG at a chosen resolution. Success means the code looks deliberate, the exported file works on the first scan, and nothing the user typed leaves the tab.

## Positioning

The entire generator, styler, and contrast checker is Rust compiled to WebAssembly, with no server round trip and no analytics. A hosted QR service cannot truthfully make that claim while also seeing the payload. The studio also shows the real cost of each styling decision: contrast ratios are recomputed per region and the export includes a four-module quiet zone.

## Operating Context

- Used in a desktop or mobile browser, often alongside a document, print layout, or storefront.
- No account, no project files, no cloud sync. Opening the page again starts from the sample link.
- Works offline after the first load through the service worker.
- Exports are downloaded locally; the optional logo is read from the user's disk and embedded in the SVG.
- The repository is also the deployment source for the static site on Vercel.

## Capabilities and Constraints

- Payloads: URL with tracking-parameter cleanup, plain text, Wi-Fi credentials, vCard 3.0, `mailto:`, `tel:`, `geo:`, and BIP-21-style crypto payment URIs.
- Styling: four module shapes, three eye frame shapes, three eye dot shapes, independent colours for modules, eye frame, eye dot, and background, transparent background, and an optional centre logo.
- Encoding: automatic error correction selection, raised to level H when a logo is present.
- Export: PNG at 512, 1024, or 2048 pixels, PNG clipboard copy, SVG download, and SVG clipboard copy.
- Scannability: WCAG contrast ratio for each coloured region against the background, with a 3:1 minimum. The browser build does not decode the generated code; the server crate exposes a separate verification endpoint.
- Constraints: no `unsafe` code in the workspace, no server dependency for the deployed site, no third-party analytics or fonts loaded at runtime.
- Undecided: whether the browser build should run the full rasterised scan verifier before export. The current copy does not claim that it does.

## Brand Commitments

- Name and domain: qr.den1zz.dev, presented as `qr.` in the accent colour and `den1zz.dev` in white.
- The visual world is an AMOLED-black instrument panel with a single cyan accent and a monospace voice. This is a deliberate identity, not a default theme.
- The product speaks plainly about what it checks. It does not claim a scan will succeed just because contrast passes.
- The MIT licence and the public repository stay visible.

## Evidence on Hand

- `README.md` documents the payload formats, styling options, export formats, and deployment shape.
- `crates/qr-core/src/*` contains the tested payload encoders, matrix generator, SVG renderer, PNG exporter, contrast maths, and verifier.
- `crates/qr-frontend/src/*` contains the studio UI and the reactive state.
- The live site at https://qr.den1zz.dev is the reference rendering. No customer quotes, benchmarks, or usage metrics exist; future work must not invent them.

## Product Principles

1. **Nothing leaves the tab.** Generation, styling, and checking stay client-side. A feature that requires uploading the payload does not belong here.
2. **Show the trade.** Every styling control has a visible consequence: bytes, version, error level, or contrast. Do not hide a scannability cost behind a preset.
3. **One screen, no wizard.** The payload, the code, and the controls stay in one workspace. No multi-step flow for a task this small.
4. **Ship a file, not a preview.** The exported PNG and SVG are the product. The preview exists to predict them accurately, including the quiet zone and transparency.
5. **Say what is checked.** Contrast is measured; decoding is not. Copy must keep that distinction.

## Accessibility & Inclusion

- Target WCAG 2.2 AA for the studio: labelled fields, visible focus, keyboard-operable controls, 24 CSS pixel minimum targets, and text contrast of at least 4.5:1.
- Honour `prefers-reduced-motion`; keep the one authored motion (the initial scan line) decorative and removable.
- The page remains readable with scripting disabled, with a plain notice that live generation needs JavaScript.
