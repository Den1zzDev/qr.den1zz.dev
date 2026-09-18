use crate::matrix::QrMatrix;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DitherAlgorithm {
    Clustered,
    Floyd,
    Bayer4,
    Bayer8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HalftoneColorMode {
    Color,
    Sampled,
    Bw,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HalftoneConfig {
    pub mode: HalftoneColorMode,
    pub dither: DitherAlgorithm,
    pub strength_bias: i32, // 0 to 120
    pub cell_core_size: u32, // 1 to 5 (for Color mode)
    pub dark_color: String,  // hex e.g. "#1A237E"
    pub clahe: bool,         // adaptive histogram equalization
    pub preserve_structure: bool,
    pub crisp_cores: bool,
}

impl Default for HalftoneConfig {
    fn default() -> Self {
        Self {
            mode: HalftoneColorMode::Color,
            dither: DitherAlgorithm::Clustered,
            strength_bias: 50,
            cell_core_size: 3,
            dark_color: "#080d16".to_string(),
            clahe: true,
            preserve_structure: true,
            crisp_cores: false,
        }
    }
}

pub const SUB_CELL_SIZE: usize = 5;
pub const QUIET_ZONE_MODULES: usize = 4;

const BAYER_4X4: [u8; 16] = [
    0, 8, 2, 10,
    12, 4, 14, 6,
    3, 11, 1, 9,
    15, 7, 13, 5,
];

const BAYER_8X8: [u8; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42,
    48, 16, 56, 24, 50, 18, 58, 26,
    12, 44, 4, 36, 14, 46, 6, 38,
    60, 28, 52, 20, 62, 30, 54, 22,
    3, 35, 11, 43, 1, 33, 9, 41,
    51, 19, 59, 27, 49, 17, 57, 25,
    15, 47, 7, 39, 13, 45, 5, 37,
    63, 31, 55, 23, 61, 29, 53, 21,
];

pub struct HalftoneEngine;

impl HalftoneEngine {
    pub fn render(
        matrix: &QrMatrix,
        source_image: &DynamicImage,
        config: &HalftoneConfig,
    ) -> (RgbaImage, String) {
        let qr_size = matrix.size;
        let sub = SUB_CELL_SIZE;
        let inner_size = qr_size * sub;
        let qz = QUIET_ZONE_MODULES * sub;
        let output_size = inner_size + 2 * qz;

        // Crop center-square from source image and resize to inner_size x inner_size
        let (iw, ih) = source_image.dimensions();
        let (sx, sy, s_dim) = if iw > ih {
            ((iw - ih) / 2, 0, ih)
        } else {
            (0, (ih - iw) / 2, iw)
        };

        let cropped = source_image.crop_imm(sx, sy, s_dim, s_dim);
        let resized = cropped.resize_exact(
            inner_size as u32,
            inner_size as u32,
            image::imageops::FilterType::Triangle,
        );
        let rgb_resized = resized.to_rgba8();

        // Convert to grayscale Float32
        let mut gray = vec![0.0f32; inner_size * inner_size];
        for y in 0..inner_size {
            for x in 0..inner_size {
                let p = rgb_resized.get_pixel(x as u32, y as u32);
                let g = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
                gray[y * inner_size + x] = g;
            }
        }

        if config.clahe {
            gray = Self::apply_histogram_equalization(&gray);
        }

        let dithered = match config.dither {
            DitherAlgorithm::Floyd => Some(Self::floyd_steinberg(&gray, inner_size, inner_size)),
            DitherAlgorithm::Bayer4 => Some(Self::bayer_dither(&gray, inner_size, inner_size, 4)),
            DitherAlgorithm::Bayer8 => Some(Self::bayer_dither(&gray, inner_size, inner_size, 8)),
            DitherAlgorithm::Clustered => None,
        };

        // Binary halftone map (1 = dark, 0 = light)
        let mut pixels = vec![0u8; output_size * output_size];
        let bias = config.strength_bias;

        for my in 0..qr_size {
            for mx in 0..qr_size {
                let qr_bit = matrix.is_dark(mx, my);
                let protect = config.preserve_structure && matrix.is_structural(mx, my);

                for sy_sub in 0..sub {
                    for sx_sub in 0..sub {
                        let ox = mx * sub + sx_sub;
                        let oy = my * sub + sy_sub;
                        let px = ox + qz;
                        let py = oy + qz;
                        let idx = py * output_size + px;

                        if protect {
                            pixels[idx] = if qr_bit { 1 } else { 0 };
                        } else if let Some(ref dither_map) = dithered {
                            let in_center = sx_sub >= 1 && sx_sub <= 3 && sy_sub >= 1 && sy_sub <= 3;
                            if in_center && bias >= 30 {
                                pixels[idx] = if qr_bit { 1 } else { 0 };
                            } else {
                                pixels[idx] = if dither_map[oy * inner_size + ox] == 0 { 1 } else { 0 };
                            }
                        } else if sx_sub >= 1 && sx_sub <= 3 && sy_sub >= 1 && sy_sub <= 3 {
                            pixels[idx] = if qr_bit { 1 } else { 0 };
                        } else {
                            let threshold = if qr_bit { 128.0 + bias as f32 } else { 128.0 - bias as f32 };
                            pixels[idx] = if gray[oy * inner_size + ox] < threshold { 1 } else { 0 };
                        }
                    }
                }
            }
        }

        // Generate output bitmap image
        let mut out_image = RgbaImage::new(output_size as u32, output_size as u32);
        let dark_rgba = parse_hex_color(&config.dark_color).unwrap_or(Rgba([8, 13, 22, 255]));
        let white_rgba = Rgba([255, 255, 255, 255]);

        match config.mode {
            HalftoneColorMode::Color => {
                // Background is white
                for pixel in out_image.pixels_mut() {
                    *pixel = white_rgba;
                }
                // Paste resized image at quiet zone offset
                for y in 0..inner_size {
                    for x in 0..inner_size {
                        let p = rgb_resized.get_pixel(x as u32, y as u32);
                        out_image.put_pixel((qz + x) as u32, (qz + y) as u32, *p);
                    }
                }

                let core_fill = config.cell_core_size.clamp(1, 5) as usize;
                let offset = (sub - core_fill) / 2;

                for my in 0..qr_size {
                    for mx in 0..qr_size {
                        let qr_bit = matrix.is_dark(mx, my);
                        let x_base = qz + mx * sub;
                        let y_base = qz + my * sub;

                        if config.preserve_structure && matrix.is_structural(mx, my) {
                            let fill_col = if qr_bit { dark_rgba } else { white_rgba };
                            for dy in 0..sub {
                                for dx in 0..sub {
                                    out_image.put_pixel((x_base + dx) as u32, (y_base + dy) as u32, fill_col);
                                }
                            }
                        } else {
                            let fill_col = if qr_bit { dark_rgba } else { white_rgba };
                            for dy in 0..core_fill {
                                for dx in 0..core_fill {
                                    out_image.put_pixel(
                                        (x_base + offset + dx) as u32,
                                        (y_base + offset + dy) as u32,
                                        fill_col,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            HalftoneColorMode::Sampled => {
                // Base white
                for pixel in out_image.pixels_mut() {
                    *pixel = white_rgba;
                }

                for my in 0..qr_size {
                    for mx in 0..qr_size {
                        let qr_bit = matrix.is_dark(mx, my);
                        let x_base = qz + mx * sub;
                        let y_base = qz + my * sub;

                        if config.preserve_structure && matrix.is_structural(mx, my) {
                            let fill_col = if qr_bit { Rgba([10, 10, 10, 255]) } else { white_rgba };
                            for dy in 0..sub {
                                for dx in 0..sub {
                                    out_image.put_pixel((x_base + dx) as u32, (y_base + dy) as u32, fill_col);
                                }
                            }
                        } else if qr_bit {
                            let cx = (mx * sub + 2).min(inner_size - 1);
                            let cy = (my * sub + 2).min(inner_size - 1);
                            let sample_color = rgb_resized.get_pixel(cx as u32, cy as u32);
                            for dy in 0..sub {
                                for dx in 0..sub {
                                    out_image.put_pixel((x_base + dx) as u32, (y_base + dy) as u32, *sample_color);
                                }
                            }
                        }
                    }
                }
            }
            HalftoneColorMode::Bw => {
                for y in 0..output_size {
                    for x in 0..output_size {
                        let is_dark = pixels[y * output_size + x] == 1;
                        let col = if is_dark { Rgba([0, 0, 0, 255]) } else { white_rgba };
                        out_image.put_pixel(x as u32, y as u32, col);
                    }
                }
            }
        }

        // Generate vector run-length SVG string
        let svg_path = Self::generate_svg_path(&pixels, output_size, output_size);
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{output_size}" height="{output_size}" viewBox="0 0 {output_size} {output_size}" shape-rendering="crispEdges" style="width:100%;height:auto;max-width:100%;display:block;"><rect width="100%" height="100%" fill="white"/><path d="{svg_path}" fill="black"/></svg>"#
        );

        (out_image, svg)
    }

    fn apply_histogram_equalization(gray: &[f32]) -> Vec<f32> {
        let mut hist = [0u32; 256];
        for &v in gray {
            let iv = (v.round() as usize).clamp(0, 255);
            hist[iv] += 1;
        }

        let mut cdf = [0f32; 256];
        let mut sum = 0;
        for i in 0..256 {
            sum += hist[i];
            cdf[i] = sum as f32;
        }

        let mut cdf_min = 0.0f32;
        for i in 0..256 {
            if cdf[i] > 0.0 {
                cdf_min = cdf[i];
                break;
            }
        }

        let total = gray.len() as f32;
        if total - cdf_min <= 0.0 {
            return gray.to_vec();
        }

        gray.iter()
            .map(|&v| {
                let iv = (v.round() as usize).clamp(0, 255);
                (((cdf[iv] - cdf_min) / (total - cdf_min)) * 255.0).round().clamp(0.0, 255.0)
            })
            .collect()
    }

    fn bayer_dither(gray: &[f32], w: usize, h: usize, matrix_size: usize) -> Vec<u8> {
        let mut out = vec![0u8; w * h];
        let scale = 255.0 / (matrix_size * matrix_size) as f32;

        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let m_val = if matrix_size == 4 {
                    BAYER_4X4[(y % 4) * 4 + (x % 4)] as f32
                } else {
                    BAYER_8X8[(y % 8) * 8 + (x % 8)] as f32
                };
                let threshold = (m_val + 0.5) * scale;
                out[i] = if gray[i] < threshold { 0 } else { 255 };
            }
        }
        out
    }

    fn floyd_steinberg(gray: &[f32], w: usize, h: usize) -> Vec<u8> {
        let mut work = gray.to_vec();
        let mut out = vec![0u8; w * h];

        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let nv = if work[i] < 128.0 { 0.0 } else { 255.0 };
                out[i] = nv as u8;
                let err = work[i] - nv;

                if x + 1 < w {
                    work[i + 1] += (err * 7.0) / 16.0;
                }
                if y + 1 < h {
                    let nr = (y + 1) * w;
                    if x > 0 {
                        work[nr + x - 1] += (err * 3.0) / 16.0;
                    }
                    work[nr + x] += (err * 5.0) / 16.0;
                    if x + 1 < w {
                        work[nr + x + 1] += (err * 1.0) / 16.0;
                    }
                }
            }
        }
        out
    }

    fn generate_svg_path(pixels: &[u8], width: usize, height: usize) -> String {
        let mut path = String::new();
        for y in 0..height {
            let mut x = 0;
            while x < width {
                if pixels[y * width + x] == 1 {
                    let start = x;
                    while x < width && pixels[y * width + x] == 1 {
                        x += 1;
                    }
                    let len = x - start;
                    path.push_str(&format!("M{start} {y}h{len}v1h-{len}Z"));
                } else {
                    x += 1;
                }
            }
        }
        path
    }

    /// Auto-detect dominant colors from image for palette swatch recommendations
    pub fn detect_palette(image: &DynamicImage) -> Vec<String> {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return vec!["#080d16".to_string()];
        }

        let sz = 64.min(w).min(h);
        let thumb = image.resize_exact(sz, sz, image::imageops::FilterType::Triangle).to_rgba8();

        let mut r_sum = 0u64;
        let mut g_sum = 0u64;
        let mut b_sum = 0u64;
        let mut count = 0u64;

        for p in thumb.pixels() {
            r_sum += p[0] as u64;
            g_sum += p[1] as u64;
            b_sum += p[2] as u64;
            count += 1;
        }

        let main_r = (((r_sum / count) as f64) * 0.45).round() as u8;
        let main_g = (((g_sum / count) as f64) * 0.45).round() as u8;
        let main_b = (((b_sum / count) as f64) * 0.45).round() as u8;
        let main_hex = format!("#{main_r:02X}{main_g:02X}{main_b:02X}");

        vec![main_hex, "#080d16".into(), "#1E3A8A".into(), "#065F46".into(), "#3fd9ff".into()]
    }
}

pub fn parse_hex_color(hex: &str) -> Option<Rgba<u8>> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some(Rgba([r, g, b, 255]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::EccLevel;
    use image::RgbaImage;

    #[test]
    fn test_halftone_render() {
        let matrix = QrMatrix::new("https://qr.den1zz.dev", EccLevel::H, true).unwrap();
        let dummy_img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let config = HalftoneConfig::default();
        let (img, svg) = HalftoneEngine::render(&matrix, &dummy_img, &config);
        assert!(img.width() > 0);
        assert!(svg.contains("<svg"));
    }
}
