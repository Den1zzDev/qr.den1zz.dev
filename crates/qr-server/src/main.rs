use axum::{
    extract::Json,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use base64::Engine;
use qr_core::{
    matrix::{EccLevel, QrMatrix},
    payload::Payload,
    vector::{VectorRenderConfig, VectorRenderer},
    verifier::{ScanVerifier, VerificationReport},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let dist_dir = std::env::var("DIST_DIR").unwrap_or_else(|_| "crates/qr-frontend/dist".to_string());

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/generate/vector", post(generate_vector))
        .route("/api/verify", post(verify_image))
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new(dist_dir));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
