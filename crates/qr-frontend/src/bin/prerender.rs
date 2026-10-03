use leptos::prelude::*;
use qr_frontend::App;
use std::fs;

#[component]
fn Shell(css: String) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"qr.den1zz.dev — Clean, Local QR Studio in Pure Rust"</title>
                <meta name="description" content="Local, client-side QR studio built in pure Rust & WebAssembly." />
                <meta name="theme-color" content="#000000" />
                <meta property="og:title" content="qr.den1zz.dev — Clean, Local QR Studio in Pure Rust" />
                <meta property="og:description" content="Local, client-side QR studio built in pure Rust & WebAssembly." />
                <meta property="og:url" content="https://qr.den1zz.dev" />
                <meta property="og:type" content="website" />
                <link rel="manifest" href="./manifest.webmanifest" />
                <link rel="icon" type="image/svg+xml" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><rect width=%22100%22 height=%22100%22 fill=%22%23000000%22/><rect x=%2220%22 y=%2220%22 width=%2260%22 height=%2260%22 rx=%2212%22 fill=%22none%22 stroke=%22%2300f0ff%22 stroke-width=%2210%22/><circle cx=%2250%22 cy=%2250%22 r=%2214%22 fill=%22%2300f0ff%22/></svg>" />
                <link rel="modulepreload" href="./qr_frontend.js" />
                <link rel="preload" href="./qr_frontend_bg.wasm" r#as="fetch" type="application/wasm" crossorigin="anonymous" />
                <style inner_html=css></style>
                <script type="module">
                    "import init from './qr_frontend.js'; init();"
                </script>
            </head>
            <body class="min-h-screen bg-[#000000] text-[#ffffff]">
                <App />
            </body>
        </html>
    }
}

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "crates/qr-frontend/dist".to_string());

    fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    let css_file = format!("{out_dir}/style.css");
    let css = fs::read_to_string(&css_file).unwrap_or_else(|_| {
        eprintln!("Warning: {css_file} not found, proceeding with empty style");
        String::new()
    });

    let html = view! { <Shell css=css /> }.to_html();
    let dest = format!("{out_dir}/index.html");
    fs::write(&dest, html).expect("Failed to write index.html");

    println!("Successfully pre-rendered to {dest}");
}
