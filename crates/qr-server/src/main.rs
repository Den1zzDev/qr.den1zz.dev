#![recursion_limit = "256"]

//! Axum server with streaming SSR and QR processing endpoints.

use std::net::SocketAddr;

use axum::extract::Json;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use base64::Engine;
use leptos::prelude::*;
use qr_core::matrix::EccLevel;
use qr_core::matrix::Overlay;
use qr_core::matrix::QrMatrix;
use qr_core::payload::Payload;
use qr_core::vector;
use qr_core::vector::VectorRenderConfig;
use qr_core::verifier;
use qr_core::verifier::VerificationReport;
use qr_frontend::App;
use serde::Deserialize;
use serde::Serialize;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Document shell used for streaming server rendering.
#[component]
fn Shell() -> impl IntoView {
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
                <link rel="stylesheet" href="/style.css" />
                <link
                    rel="preload"
                    href="/fonts/maple-mono.woff2"
                    r#as="font"
                    type="font/woff2"
                    crossorigin="anonymous"
                />
                <script type="module">
                    "import init, { start_hydrate } from '/pkg/qr_frontend.js'; await init({ module_or_path: '/pkg/qr_frontend_bg.wasm' }); if (typeof start_hydrate === 'function') { start_hydrate(); }"
                </script>
            </head>
            <body class="min-h-screen bg-[#000000] text-[#ffffff]">
                <App />
            </body>
        </html>
    }
}

/// Request body for `POST /api/generate/vector`.
#[derive(Debug, Deserialize)]
struct VectorGenerateRequest {
    payload: Payload,
    config: VectorRenderConfig,
    ecc: Option<EccLevel>,
}

/// Response body for `POST /api/generate/vector`.
#[derive(Debug, Serialize)]
struct VectorGenerateResponse {
    svg: String,
    content: String,
    raw_bytes: usize,
    cleaned_bytes: usize,
    stripped_count: usize,
}

/// Request body for `POST /api/verify`.
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    image_base64: String,
    expected_payload: String,
    dark_color: Option<String>,
    light_color: Option<String>,
}

/// Returns `OK` while the server is healthy.
async fn health_check() -> &'static str {
    "OK"
}

/// Generates an SVG for the requested payload and render configuration.
async fn generate_vector(
    Json(request): Json<VectorGenerateRequest>,
) -> Result<Json<VectorGenerateResponse>, (StatusCode, String)> {
    let sanitized = request.payload.sanitize();
    let overlay = if request.config.logo.is_some() {
        Overlay::Image
    } else {
        Overlay::None
    };
    let ecc = request.ecc.unwrap_or(EccLevel::Auto);
    let matrix = QrMatrix::try_new(&sanitized.content, ecc, overlay)
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
    let svg = vector::render_svg(&matrix, &request.config);

    Ok(Json(VectorGenerateResponse {
        svg,
        content: sanitized.content,
        raw_bytes: sanitized.raw_bytes,
        cleaned_bytes: sanitized.cleaned_bytes,
        stripped_count: sanitized.stripped_params_count,
    }))
}

/// Decodes an uploaded image and verifies that it is scannable.
async fn verify_image(
    Json(request): Json<VerifyRequest>,
) -> Result<Json<VerificationReport>, (StatusCode, String)> {
    let encoded = request
        .image_base64
        .trim_start_matches("data:image/png;base64,")
        .trim_start_matches("data:image/jpeg;base64,")
        .trim_start_matches("data:image/webp;base64,");
    let image_bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid base64 image: {error}"),
            )
        })?;
    let image = image::load_from_memory(&image_bytes).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to decode image: {error}"),
        )
    })?;

    let dark_color = request.dark_color.as_deref().unwrap_or("#000000");
    let light_color = request.light_color.as_deref().unwrap_or("#FFFFFF");
    Ok(Json(verifier::verify(
        &image,
        &request.expected_payload,
        dark_color,
        light_color,
    )))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pkg_dir = std::env::var("PKG_DIR").unwrap_or_else(|_| "crates/qr-server/pkg".to_string());
    let style_path = std::env::var("STYLE_PATH")
        .unwrap_or_else(|_| "crates/qr-frontend/dist/style.css".to_string());
    let font_dir =
        std::env::var("FONT_DIR").unwrap_or_else(|_| "crates/qr-frontend/dist/fonts".to_string());

    if !std::path::Path::new(&style_path).exists() {
        tracing::warn!(
            "Stylesheet not found at {style_path}. Build it with Tailwind or set STYLE_PATH."
        );
    }

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/generate/vector", post(generate_vector))
        .route("/api/verify", post(verify_image))
        .route_service("/style.css", ServeFile::new(style_path))
        .nest_service("/fonts", ServeDir::new(font_dir))
        .nest_service("/pkg", ServeDir::new(pkg_dir))
        .fallback(leptos_axum::render_app_to_stream(Shell))
        .layer(CorsLayer::permissive());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(4321);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on http://{address}");

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await
}
