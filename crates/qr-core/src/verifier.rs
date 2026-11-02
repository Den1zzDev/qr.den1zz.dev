//! Scanability verification for rendered QR codes.

use image::DynamicImage;
use image::GenericImageView;
use rqrr::PreparedImage;
use serde::Deserialize;
use serde::Serialize;

use crate::contrast::contrast_ratio;
use crate::matrix::QrMatrix;

/// Minimum WCAG contrast ratio considered acceptable for scanning.
const MIN_CONTRAST_RATIO: f32 = 3.5;

/// Pixels per module used by [`verify_matrix`].
const MODULE_PIXELS: usize = 3;

/// Quiet-zone width in modules used by [`verify_matrix`].
const QUIET_MODULES: usize = 4;

/// Outcome of verifying a rendered code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ScanStatus {
    /// The code decoded successfully and meets contrast requirements.
    Verified,
    /// The code decoded but needs attention.
    Uncertain {
        /// Human-readable explanation of the concern.
        reason: String,
    },
    /// The code could not be decoded.
    Unscannable,
}

/// Result of a verification run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VerificationReport {
    /// Overall verification status.
    pub status: ScanStatus,
    /// Decoded payload, when decoding succeeded.
    pub decoded_content: Option<String>,
    /// WCAG contrast ratio between the dark and light colours.
    pub contrast_ratio: f32,
    /// Whether [`contrast_ratio`](Self::contrast_ratio) meets the minimum
    /// threshold.
    pub is_contrast_acceptable: bool,
}

/// Verifies a raster image against the expected payload.
#[must_use]
pub fn verify(
    image: &DynamicImage,
    expected_payload: &str,
    dark_hex: &str,
    light_hex: &str,
) -> VerificationReport {
    let contrast = contrast_ratio(dark_hex, light_hex);
    if expected_payload.is_empty() || image.width() == 0 || image.height() == 0 {
        return unscannable(contrast);
    }

    let (width, height) = image.dimensions();
    match image {
        DynamicImage::ImageLuma8(luma) => {
            let raw = luma.as_raw();
            decode_from_fill(
                width as usize,
                height as usize,
                |x, y| raw[y * width as usize + x],
                expected_payload,
                contrast,
            )
        }
        _ => {
            let gray = image.to_luma8();
            decode_from_fill(
                width as usize,
                height as usize,
                |x, y| gray.get_pixel(x as u32, y as u32)[0],
                expected_payload,
                contrast,
            )
        }
    }
}

/// Verifies a single-channel grayscale buffer of `width` by `height` pixels.
#[must_use]
pub fn verify_luma(
    width: usize,
    height: usize,
    luma_pixels: &[u8],
    expected_payload: &str,
    dark_hex: &str,
    light_hex: &str,
) -> VerificationReport {
    let contrast = contrast_ratio(dark_hex, light_hex);
    if expected_payload.is_empty()
        || width == 0
        || height == 0
        || luma_pixels.len() < width * height
    {
        return unscannable(contrast);
    }

    decode_from_fill(
        width,
        height,
        |x, y| luma_pixels[y * width + x],
        expected_payload,
        contrast,
    )
}

/// Verifies `matrix` by rasterising it into a minimal grayscale buffer.
#[must_use]
pub fn verify_matrix(
    matrix: &QrMatrix,
    expected_payload: &str,
    dark_hex: &str,
    light_hex: &str,
) -> VerificationReport {
    let contrast = contrast_ratio(dark_hex, light_hex);
    if expected_payload.is_empty() || matrix.size == 0 {
        return unscannable(contrast);
    }

    let total_modules = matrix.size + QUIET_MODULES * 2;
    let width = total_modules * MODULE_PIXELS;
    let height = width;
    let mut luma = vec![255u8; width * height];

    for module_y in 0..matrix.size {
        for module_x in 0..matrix.size {
            if !matrix.is_dark(module_x, module_y) {
                continue;
            }

            let start_x = (module_x + QUIET_MODULES) * MODULE_PIXELS;
            let start_y = (module_y + QUIET_MODULES) * MODULE_PIXELS;
            for pixel_y in start_y..start_y + MODULE_PIXELS {
                let row_start = pixel_y * width;
                luma[row_start + start_x..row_start + start_x + MODULE_PIXELS].fill(0);
            }
        }
    }

    verify_luma(width, height, &luma, expected_payload, dark_hex, light_hex)
}

/// Builds a report for input that cannot be decoded.
fn unscannable(contrast: f32) -> VerificationReport {
    VerificationReport {
        status: ScanStatus::Unscannable,
        decoded_content: None,
        contrast_ratio: contrast,
        is_contrast_acceptable: contrast >= MIN_CONTRAST_RATIO,
    }
}

/// Decodes a grayscale image produced by `fill` and compares the result.
fn decode_from_fill(
    width: usize,
    height: usize,
    fill: impl FnMut(usize, usize) -> u8,
    expected_payload: &str,
    contrast: f32,
) -> VerificationReport {
    let mut prepared = PreparedImage::prepare_from_greyscale(width, height, fill);
    let grids = prepared.detect_grids();
    let mut decoded_content = None;

    for grid in grids {
        let mut content_bytes = Vec::new();
        if grid.decode_to(&mut content_bytes).is_err() {
            continue;
        }
        if let Ok(text) = String::from_utf8(content_bytes) {
            decoded_content = Some(text);
            break;
        }
    }

    let is_contrast_acceptable = contrast >= MIN_CONTRAST_RATIO;
    let status = match decoded_content.as_deref() {
        Some(decoded) if decoded == expected_payload && is_contrast_acceptable => {
            ScanStatus::Verified
        }
        Some(decoded) if decoded == expected_payload => ScanStatus::Uncertain {
            reason: format!("Low contrast ratio ({contrast:.1}:1)"),
        },
        Some(_) => ScanStatus::Uncertain {
            reason: "Decoded text did not match expected payload".to_string(),
        },
        None => ScanStatus::Unscannable,
    };

    VerificationReport {
        status,
        decoded_content,
        contrast_ratio: contrast,
        is_contrast_acceptable,
    }
}

#[cfg(test)]
mod tests {
    use image::GrayImage;

    use super::*;
    use crate::matrix::EccLevel;
    use crate::matrix::Overlay;

    #[test]
    fn empty_payload_is_unscannable() {
        let image = DynamicImage::ImageLuma8(GrayImage::new(100, 100));
        let report = verify(&image, "", "#000000", "#FFFFFF");

        assert_eq!(report.status, ScanStatus::Unscannable);
        assert_eq!(report.decoded_content, None);
    }

    #[test]
    fn generated_matrices_are_scannable() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        let report = verify_matrix(&matrix, "https://den1zz.dev", "#000000", "#FFFFFF");

        assert_eq!(report.status, ScanStatus::Verified);
        assert_eq!(
            report.decoded_content.as_deref(),
            Some("https://den1zz.dev")
        );
    }

    #[test]
    fn low_contrast_is_reported_as_uncertain() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        let report = verify_matrix(&matrix, "https://den1zz.dev", "#999999", "#FFFFFF");

        assert!(matches!(report.status, ScanStatus::Uncertain { .. }));
        assert!(!report.is_contrast_acceptable);
    }
}
