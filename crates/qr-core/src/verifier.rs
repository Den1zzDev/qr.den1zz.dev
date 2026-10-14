#[cfg(feature = "verifier")]
use image::{DynamicImage, GenericImageView};
#[cfg(feature = "verifier")]
use rqrr::PreparedImage;
#[cfg(feature = "verifier")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "verifier")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanStatus {
    Verified,
    Uncertain { reason: String },
    Unscannable,
}

#[cfg(feature = "verifier")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationReport {
    pub status: ScanStatus,
    pub decoded_content: Option<String>,
    pub contrast_ratio: f32,
    pub is_contrast_acceptable: bool,
}

#[cfg(feature = "verifier")]
pub struct ScanVerifier;

#[cfg(feature = "verifier")]
impl ScanVerifier {
    pub fn verify(
        image: &DynamicImage,
        expected_payload: &str,
        dark_hex: &str,
        light_hex: &str,
    ) -> VerificationReport {
        let contrast = calculate_contrast_ratio(dark_hex, light_hex);
        let is_contrast_acceptable = contrast >= 3.5;

        // Zero-byte empty payload / zero dimension fast-path: short circuit immediately
        if expected_payload.is_empty() || image.width() == 0 || image.height() == 0 {
            return VerificationReport {
                status: ScanStatus::Unscannable,
                decoded_content: None,
                contrast_ratio: contrast,
                is_contrast_acceptable,
            };
        }

        let (w, h) = image.dimensions();
        match image {
            DynamicImage::ImageLuma8(luma) => {
                let raw = luma.as_raw();
                Self::decode_from_fill(
                    w as usize,
                    h as usize,
                    |x, y| raw[y * w as usize + x],
                    expected_payload,
                    contrast,
                    is_contrast_acceptable,
                )
            }
            _ => {
                let gray_img = image.to_luma8();
                Self::decode_from_fill(
                    w as usize,
                    h as usize,
                    |x, y| gray_img.get_pixel(x as u32, y as u32)[0],
                    expected_payload,
                    contrast,
                    is_contrast_acceptable,
                )
            }
        }
    }

    /// Verifies from a minimal single-channel grayscale (u8) buffer (e.g. 3-4px per module)
    pub fn verify_luma(
        w: usize,
        h: usize,
        luma_pixels: &[u8],
        expected_payload: &str,
        dark_hex: &str,
        light_hex: &str,
    ) -> VerificationReport {
        let contrast = calculate_contrast_ratio(dark_hex, light_hex);
        let is_contrast_acceptable = contrast >= 3.5;

        if expected_payload.is_empty() || w == 0 || h == 0 || luma_pixels.len() < w * h {
            return VerificationReport {
                status: ScanStatus::Unscannable,
                decoded_content: None,
                contrast_ratio: contrast,
                is_contrast_acceptable,
            };
        }

        Self::decode_from_fill(
            w,
            h,
            |x, y| luma_pixels[y * w + x],
            expected_payload,
            contrast,
            is_contrast_acceptable,
        )
    }

    /// Verifies directly from a QrMatrix using a minimal single-channel grayscale buffer
    pub fn verify_matrix(
        matrix: &crate::matrix::QrMatrix,
        expected_payload: &str,
        dark_hex: &str,
        light_hex: &str,
    ) -> VerificationReport {
        let contrast = calculate_contrast_ratio(dark_hex, light_hex);
        let is_contrast_acceptable = contrast >= 3.5;

        if expected_payload.is_empty() || matrix.size == 0 {
            return VerificationReport {
                status: ScanStatus::Unscannable,
                decoded_content: None,
                contrast_ratio: contrast,
                is_contrast_acceptable,
            };
        }

        let module_px = 3;
        let quiet_modules = 4;
        let total_modules = matrix.size + quiet_modules * 2;
        let w = total_modules * module_px;
        let h = w;

        let mut luma = vec![255u8; w * h];

        for my in 0..matrix.size {
            for mx in 0..matrix.size {
                if matrix.is_dark(mx, my) {
                    let start_x = (mx + quiet_modules) * module_px;
                    let start_y = (my + quiet_modules) * module_px;
                    for py in start_y..(start_y + module_px) {
                        let row_offset = py * w;
                        luma[row_offset + start_x..row_offset + start_x + module_px].fill(0);
                    }
                }
            }
        }

        Self::verify_luma(w, h, &luma, expected_payload, dark_hex, light_hex)
    }

    fn decode_from_fill<F: FnMut(usize, usize) -> u8>(
        w: usize,
        h: usize,
        fill: F,
        expected_payload: &str,
        contrast: f32,
        is_contrast_acceptable: bool,
    ) -> VerificationReport {
        let mut prepared = PreparedImage::prepare_from_greyscale(w, h, fill);
        let grids = prepared.detect_grids();
        let mut decoded_text = None;

        for grid in grids {
            let mut content_bytes = Vec::new();
            if grid.decode_to(&mut content_bytes).is_ok() {
                if let Ok(text) = String::from_utf8(content_bytes) {
                    decoded_text = Some(text);
                    break;
                }
            }
        }

        let status = match decoded_text.as_deref() {
            Some(decoded) => {
                if decoded == expected_payload {
                    if is_contrast_acceptable {
                        ScanStatus::Verified
                    } else {
                        ScanStatus::Uncertain {
                            reason: format!("Low contrast ratio ({contrast:.1}:1)"),
                        }
                    }
                } else {
                    ScanStatus::Uncertain {
                        reason: "Decoded text did not match expected payload".to_string(),
                    }
                }
            }
            None => ScanStatus::Unscannable,
        };

        VerificationReport {
            status,
            decoded_content: decoded_text,
            contrast_ratio: contrast,
            is_contrast_acceptable,
        }
    }
}

pub fn get_relative_luminance(hex: &str) -> f32 {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() != 6 {
        return 0.0;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0) as f32 / 255.0;

    let adjust = |v: f32| -> f32 {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * adjust(r) + 0.7152 * adjust(g) + 0.0722 * adjust(b)
}

pub fn calculate_contrast_ratio(hex1: &str, hex2: &str) -> f32 {
    let l1 = get_relative_luminance(hex1);
    let l2 = get_relative_luminance(hex2);
    let brightest = l1.max(l2);
    let darkest = l1.min(l2);
    (brightest + 0.05) / (darkest + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_ratio() {
        let black = "#000000";
        let white = "#FFFFFF";
        let ratio = calculate_contrast_ratio(black, white);
        assert!((ratio - 21.0).abs() < 0.5);
    }

    #[test]
    fn test_verify_empty_payload_fast_path() {
        let img = DynamicImage::ImageLuma8(image::GrayImage::new(100, 100));
        let report = ScanVerifier::verify(&img, "", "#000000", "#FFFFFF");
        assert_eq!(report.status, ScanStatus::Unscannable);
        assert_eq!(report.decoded_content, None);
    }

    #[test]
    fn test_verify_matrix_scannability() {
        let matrix =
            crate::matrix::QrMatrix::new("https://den1zz.dev", crate::matrix::EccLevel::M, false)
                .unwrap();
        let report =
            ScanVerifier::verify_matrix(&matrix, "https://den1zz.dev", "#000000", "#FFFFFF");
        assert_eq!(report.status, ScanStatus::Verified);
        assert_eq!(
            report.decoded_content.as_deref(),
            Some("https://den1zz.dev")
        );
    }
}
