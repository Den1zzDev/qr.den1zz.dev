use axum::{
    extract::Json,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use base64::Engine;
use leptos::prelude::*;
use qr_core::{
    matrix::{EccLevel, QrMatrix},
    payload::Payload,
    vector::{VectorRenderConfig, VectorRenderer},
    verifier::{ScanVerifier, VerificationReport},
};
use qr_frontend::App;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const TAILWIND_CONFIG: &str = r##"
tailwind.config = {
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        mono: ['"Maple Mono NF"', '"Maple Mono"', 'monospace'],
        sans: ['"Maple Mono NF"', '"Maple Mono"', 'monospace'],
      },
      colors: {
        bg: '#000000',
        'bg-elevated': '#080808',
        panel: '#0a0a0a',
        'panel-border': 'rgba(255, 255, 255, 0.12)',
        accent: '#00f0ff',
        'accent-dim': '#008b99',
        'accent-glow': 'rgba(0, 240, 255, 0.25)',
        text: '#ffffff',
        'text-dim': '#a0a0a0',
        'text-faint': '#555555',
      }
    }
  }
}
"##;

const CSS_STYLES: &str = r##"
    :root {
      --font-mono: "Maple Mono NF", "Maple Mono", monospace;
      --bg: #000000;
      --bg-elevated: #080808;
      --panel: #0a0a0a;
      --panel-border: rgba(255, 255, 255, 0.12);
      --accent: #00f0ff;
      --accent-dim: #008b99;
      --accent-glow: rgba(0, 240, 255, 0.25);
      --text: #ffffff;
      --text-dim: #a0a0a0;
      --text-faint: #555555;
    }

    *, *::before, *::after {
      box-sizing: border-box;
      font-family: "Maple Mono NF", "Maple Mono", monospace !important;
    }

    html {
      background: #000000;
      color-scheme: dark;
      scroll-behavior: smooth;
    }

    body {
      min-width: 320px;
      margin: 0;
      background: #000000;
      color: #ffffff;
      font-family: "Maple Mono NF", "Maple Mono", monospace !important;
      -webkit-font-smoothing: antialiased;
      line-height: 1.6;
    }

    ::selection {
      background: rgba(0, 240, 255, 0.25);
      color: #00f0ff;
    }

    /* AMOLED Glass Panels */
    .glass-panel {
      position: relative;
      overflow: hidden;
      border: 1px solid var(--panel-border);
      border-radius: 1rem;
      background: #0a0a0a;
      box-shadow: 0 8px 30px rgba(0, 0, 0, 0.8);
      transition: border-color 150ms ease;
    }

    .glass-panel:hover {
      border-color: rgba(255, 255, 255, 0.2);
    }

    /* Buttons */
    .button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      min-height: 2.25rem;
      padding: 0.5rem 1.25rem;
      border-radius: 999px;
      font-size: 0.8125rem;
      font-weight: 600;
      text-decoration: none;
      transition: transform 120ms ease, background 120ms ease, border-color 120ms ease, color 120ms ease;
      cursor: pointer;
    }

    .button:hover {
      transform: translateY(-1px);
    }

    .button:active {
      transform: translateY(0);
    }

    .button-primary {
      border: 1px solid var(--accent);
      background: var(--accent);
      color: #000000;
      box-shadow: 0 0 16px var(--accent-glow);
    }

    .button-primary:hover {
      background: #4df4ff;
      border-color: #4df4ff;
    }

    .button-white {
      border: 1px solid #ffffff;
      background: #ffffff;
      color: #000000;
    }

    .button-white:hover {
      background: #e0e0e0;
      border-color: #e0e0e0;
    }

    .button-secondary {
      border: 1px solid var(--panel-border);
      background: #111111;
      color: #ffffff;
    }

    .button-secondary:hover {
      border-color: #ffffff;
      color: #ffffff;
    }

    /* Viewfinder QR Stage Frame */
    .viewfinder-frame {
      position: relative;
      width: 100%;
      aspect-ratio: 1 / 1;
      max-width: 360px;
      max-height: 360px;
      margin: 0 auto;
      border-radius: 1rem;
      background-color: #000000;
      border: 1px solid rgba(255, 255, 255, 0.12);
      box-shadow: inset 0 0 20px rgba(0, 0, 0, 0.9);
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 1.25rem;
      overflow: hidden;
    }

    /* Guaranteed SVG scaling: fills frame without overflowing */
    .qr-svg-wrapper {
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .qr-svg-wrapper svg {
      width: 100% !important;
      height: 100% !important;
      max-width: 100% !important;
      max-height: 100% !important;
      object-fit: contain;
      display: block !important;
    }

    /* Custom Form Inputs */
    input[type="text"],
    input[type="password"],
    input[type="url"],
    textarea,
    select {
      background: #050505;
      border: 1px solid var(--panel-border);
      border-radius: 0.5rem;
      color: #ffffff;
      font-size: 0.875rem;
      padding: 0.625rem 0.875rem;
      transition: border-color 150ms ease;
    }

    input[type="text"]:focus,
    input[type="password"]:focus,
    input[type="url"]:focus,
    textarea:focus,
    select:focus {
      outline: none;
      border-color: var(--accent);
    }

    input[type="range"] {
      accent-color: var(--accent);
    }

    /* Scrollbars */
    ::-webkit-scrollbar {
      width: 6px;
      height: 6px;
    }
    ::-webkit-scrollbar-track {
      background: #000000;
    }
    ::-webkit-scrollbar-thumb {
      background: #222222;
      border-radius: 3px;
    }
    ::-webkit-scrollbar-thumb:hover {
      background: #444444;
    }
"##;

#[component]
fn Shell() -> impl IntoView {
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
                <link rel="icon" type="image/svg+xml" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><rect width=%22100%22 height=%22100%22 fill=%22%23000000%22/><rect x=%2220%22 y=%2220%22 width=%2260%22 height=%2260%22 rx=%2212%22 fill=%22none%22 stroke=%22%2300f0ff%22 stroke-width=%2210%22/><circle cx=%2250%22 cy=%2250%22 r=%2214%22 fill=%22%2300f0ff%22/></svg>" />
                <script src="https://cdn.tailwindcss.com"></script>
                <script inner_html=TAILWIND_CONFIG></script>
                <style inner_html=CSS_STYLES></style>
                <script type="module">
                    "import init from '/pkg/qr_frontend.js'; init();"
                </script>
            </head>
            <body class="min-h-screen bg-[#000000] text-[#ffffff]">
                <App />
            </body>
        </html>
    }
}

#[derive(Debug, Deserialize)]
pub struct VectorGenerateRequest {
    pub payload: Payload,
    pub config: VectorRenderConfig,
    pub ecc: Option<EccLevel>,
}

#[derive(Debug, Serialize)]
pub struct VectorGenerateResponse {
    pub svg: String,
    pub content: String,
    pub raw_bytes: usize,
    pub cleaned_bytes: usize,
    pub stripped_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub image_base64: String,
    pub expected_payload: String,
    pub dark_color: Option<String>,
    pub light_color: Option<String>,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn generate_vector(
    Json(req): Json<VectorGenerateRequest>,
) -> Result<Json<VectorGenerateResponse>, (StatusCode, String)> {
    let sanitized = req.payload.sanitize();
    let has_logo = req.config.logo.is_some();
    let ecc = req.ecc.unwrap_or(EccLevel::Auto);

    let matrix = QrMatrix::new(&sanitized.content, ecc, has_logo)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let svg = VectorRenderer::render_svg(&matrix, &req.config);

    Ok(Json(VectorGenerateResponse {
        svg,
        content: sanitized.content,
        raw_bytes: sanitized.raw_bytes,
        cleaned_bytes: sanitized.cleaned_bytes,
        stripped_count: sanitized.stripped_params_count,
    }))
}

async fn verify_image(
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerificationReport>, (StatusCode, String)> {
    let raw_b64 = req
        .image_base64
        .trim_start_matches("data:image/png;base64,")
        .trim_start_matches("data:image/jpeg;base64,")
        .trim_start_matches("data:image/webp;base64,");

    let img_bytes = base64::engine::general_purpose::STANDARD
        .decode(raw_b64)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64 image: {e}")))?;

    let dyn_img = image::load_from_memory(&img_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to decode image: {e}")))?;

    let dark_hex = req.dark_color.as_deref().unwrap_or("#000000");
    let light_hex = req.light_color.as_deref().unwrap_or("#FFFFFF");

    let report = ScanVerifier::verify(&dyn_img, &req.expected_payload, dark_hex, light_hex);
    Ok(Json(report))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pkg_dir = std::env::var("PKG_DIR").unwrap_or_else(|_| "crates/qr-server/pkg".to_string());

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/generate/vector", post(generate_vector))
        .route("/api/verify", post(verify_image))
        .nest_service("/pkg", ServeDir::new(pkg_dir))
        .fallback(leptos_axum::render_app_to_stream(Shell))
        .layer(CorsLayer::permissive());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4321);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
