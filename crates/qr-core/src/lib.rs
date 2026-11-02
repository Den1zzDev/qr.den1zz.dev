//! Core library for the QR Studio application.
//!
//! The crate is deliberately free of DOM and network code so that the same
//! logic can run in a WebAssembly frontend and in an Axum server. It provides:
//!
//! - [`matrix`]: QR matrix construction and structural pattern queries.
//! - [`payload`]: payload builders that produce scannable payload strings.
//! - [`vector`]: SVG rendering with configurable shapes, colours, and logos.
//! - [`contrast`]: WCAG 2.1 relative luminance and contrast calculations.
//! - `export`: rasterisation and PNG export (feature `export`).
//! - `verifier`: scanability verification (feature `verifier`).

#![warn(missing_docs)]

pub mod contrast;
#[cfg(feature = "export")]
pub mod export;
pub mod matrix;
pub mod payload;
pub mod vector;
#[cfg(feature = "verifier")]
pub mod verifier;

pub use matrix::{EccLevel, EncodeError, Overlay, QrMatrix};
pub use payload::{CleanedUrl, CryptoCurrency, Payload, SanitizedPayload, UrlQueryInfo, WifiAuth};
pub use vector::{
    CtaBannerConfig, EyeDotShape, EyeFrameShape, GradientConfig, LogoConfig, LogoMaskShape,
    ModuleShape, VectorRenderConfig,
};
#[cfg(feature = "verifier")]
pub use verifier::{ScanStatus, VerificationReport};
