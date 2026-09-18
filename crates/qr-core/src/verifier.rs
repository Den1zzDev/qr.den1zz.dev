use image::{DynamicImage, GenericImageView};
use rqrr::PreparedImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScanStatus {
    Verified,
    Uncertain { reason: String },
    Unscannable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationReport {
    pub status: ScanStatus,
    pub decoded_content: Option<String>,
    pub contrast_ratio: f32,
    pub is_contrast_acceptable: bool,
}

pub struct ScanVerifier;

impl ScanVerifier {
    pub fn verify(image: &DynamicImage, expected_payload: &str, dark_hex: &str, light_hex: &str) -> VerificationReport {
        let (w, h) = image.dimensions();
        let gray_img = image.to_luma8();
        let mut prepared = PreparedImage::prepare_from_greyscale(w as usize, h as usize, |x, y| {
            gray_img.get_pixel(x as u32, y as u32)[0]
        });

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

        let contrast = calculate_contrast_ratio(dark_hex, light_hex);
        let is_contrast_acceptable = contrast >= 3.5;

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
}
