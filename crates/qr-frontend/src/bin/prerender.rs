#![recursion_limit = "256"]

//! Static site pre-renderer for the WebAssembly frontend.

use std::fs;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use leptos::prelude::*;
use qr_frontend::App;

/// Document shell used for the pre-rendered site.
#[component]
fn Shell(css: String, timestamp: u64) -> impl IntoView {
    let js_path = format!("./qr_frontend.js?v={timestamp}");
    let wasm_path = format!("./qr_frontend_bg.wasm?v={timestamp}");
    let script = format!(
        "import init, {{ start_hydrate }} from '{js_path}'; await init({{ module_or_path: '{wasm_path}' }}); if (typeof start_hydrate === 'function') {{ start_hydrate(); }}"
    );
    let structured_data = r#"{
        "@context": "https://schema.org",
        "@type": "WebApplication",
        "name": "QR Code Studio",
        "url": "https://qr.den1zz.dev/",
        "description": "Design a QR code, tune the modules and colours, and export a print-ready PNG or SVG. Everything runs in the browser.",
        "applicationCategory": "UtilitiesApplication",
        "operatingSystem": "Any",
        "browserRequirements": "Requires WebAssembly and JavaScript",
        "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
        "isAccessibleForFree": true,
        "license": "https://github.com/Den1zzDev/qr.den1zz.dev/blob/main/LICENSE"
    }"#;

    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
                <title>"qr.den1zz.dev · Local QR code studio"</title>
                <meta
                    name="description"
                    content="Design a QR code, tune the modules and colours, and export a print-ready PNG or SVG. Everything runs in your browser."
                />
                <meta name="color-scheme" content="dark" />
                <meta name="theme-color" content="#000000" />
                <link rel="canonical" href="https://qr.den1zz.dev/" />

                <meta property="og:type" content="website" />
                <meta property="og:site_name" content="qr.den1zz.dev" />
                <meta property="og:title" content="qr.den1zz.dev · Local QR code studio" />
                <meta
                    property="og:description"
                    content="Design a QR code, tune the modules and colours, and export a print-ready PNG or SVG. Everything runs in your browser."
                />
                <meta property="og:url" content="https://qr.den1zz.dev/" />
                <meta property="og:image" content="https://qr.den1zz.dev/og.png" />
                <meta property="og:image:width" content="1200" />
                <meta property="og:image:height" content="630" />
                <meta property="og:image:alt" content="qr.den1zz.dev studio wordmark beside a generated QR code" />
                <meta name="twitter:card" content="summary_large_image" />
                <meta name="twitter:title" content="qr.den1zz.dev · Local QR code studio" />
                <meta
                    name="twitter:description"
                    content="Design a QR code, tune the modules and colours, and export a print-ready PNG or SVG. Everything runs in your browser."
                />
                <meta name="twitter:image" content="https://qr.den1zz.dev/og.png" />

                <link rel="manifest" href="./manifest.webmanifest" />
                <link rel="apple-touch-icon" href="./apple-touch-icon.png" />
                <link rel="icon" type="image/svg+xml" href="./favicon.svg" />
                <link rel="icon" type="image/png" sizes="192x192" href="./icon-192.png" />
                <link
                    rel="preload"
                    href="./fonts/maple-mono.woff2"
                    r#as="font"
                    type="font/woff2"
                    crossorigin="anonymous"
                />
                <link rel="modulepreload" href=js_path />
                <link
                    rel="preload"
                    href=wasm_path
                    r#as="fetch"
                    type="application/wasm"
                    crossorigin="anonymous"
                />
                <style inner_html=css></style>
                <script type="application/ld+json" inner_html=structured_data></script>
                <script>
                    "if ('serviceWorker' in navigator) { window.addEventListener('load', () => navigator.serviceWorker.register('./sw.js').catch(() => {})); }"
                </script>
                <script type="module" inner_html=script></script>
            </head>
            <body class="min-h-screen bg-[#000000] text-[#ffffff]">
                <App />
            </body>
        </html>
    }
}

fn main() {
    let output_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "crates/qr-frontend/dist".to_string());
    fs::create_dir_all(&output_dir).expect("Failed to create the output directory");

    let css_file = format!("{output_dir}/style.css");
    let css = fs::read_to_string(&css_file).unwrap_or_else(|_| {
        eprintln!("Warning: {css_file} not found, proceeding with empty styles");
        String::new()
    });

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let html = view! { <Shell css=css timestamp=timestamp /> }.to_html();
    let destination = format!("{output_dir}/index.html");
    fs::write(&destination, html).expect("Failed to write index.html");

    println!("Successfully pre-rendered to {destination}");
}
