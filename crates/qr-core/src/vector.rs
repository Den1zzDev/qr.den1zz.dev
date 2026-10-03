use crate::matrix::QrMatrix;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleShape {
    Square,
    Smooth,
    Dots,
    Classy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EyeFrameShape {
    Square,
    Rounded,
    Circle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EyeDotShape {
    Square,
    Rounded,
    Circle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GradientConfig {
    Linear { angle: f32, color_start: String, color_end: String },
    Radial { color_start: String, color_end: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogoMaskShape {
    Square,
    Rounded,
    Circle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogoConfig {
    pub data_url: String,
    pub shape: LogoMaskShape,
    pub scale_percent: f32, // 15.0 to 30.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CtaBannerConfig {
    pub text: String,
    pub font_size: f32,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorRenderConfig {
    pub scale: f32,
    pub margin_modules: usize,
    pub module_shape: ModuleShape,
    pub eye_frame_shape: EyeFrameShape,
    pub eye_dot_shape: EyeDotShape,
    pub code_color: String,
    pub gradient: Option<GradientConfig>,
    pub eye_color: Option<String>,
    #[serde(default)]
    pub eye_dot_color: Option<String>,
    pub bg_color: Option<String>,
    pub logo: Option<LogoConfig>,
    pub cta: Option<CtaBannerConfig>,
    pub alt_text: Option<String>,
}

impl Default for VectorRenderConfig {
    fn default() -> Self {
        Self {
            scale: 20.0,
            margin_modules: 2,
            module_shape: ModuleShape::Classy,
            eye_frame_shape: EyeFrameShape::Rounded,
            eye_dot_shape: EyeDotShape::Circle,
            code_color: "#000000".to_string(),
            gradient: None,
            eye_color: Some("#000000".to_string()),
            eye_dot_color: Some("#008b99".to_string()),
            bg_color: Some("#ffffff".to_string()),
            logo: None,
            cta: None,
            alt_text: Some("QR Code".to_string()),
        }
    }
}

pub struct VectorRenderer;

impl VectorRenderer {
    pub fn render_svg(matrix: &QrMatrix, config: &VectorRenderConfig) -> String {
        let size = matrix.size;
        let scale = config.scale;
        let padding = config.margin_modules as f32 * scale;
        let qr_grid_size = size as f32 * scale;
        let qr_width = qr_grid_size + 2.0 * padding;

        let (cta_text, cta_height, target_font_size) = if let Some(cta) = &config.cta {
            let t = cta.text.trim().to_uppercase();
            if !t.is_empty() {
                let scale_factor = qr_width / 340.0;
                let fs = (cta.font_size * scale_factor).max(10.0);
                let h = (scale * 3.2).max(fs * 2.2);
                (Some(t), h, fs)
            } else {
                (None, 0.0, 0.0)
            }
        } else {
            (None, 0.0, 0.0)
        };

        let full_height = qr_width + cta_height;

        let mut defs = String::new();
        let body_fill = if let Some(grad) = &config.gradient {
            match grad {
                GradientConfig::Radial { color_start, color_end } => {
                    let cy_pct = ((qr_width / 2.0) / full_height) * 100.0;
                    let r_pct = 50.0;
                    defs.push_str(&format!(
                        r#"<radialGradient id="qr-grad" cx="50%" cy="{cy_pct:.2}%" r="{r_pct:.2}%"><stop offset="0%" stop-color="{color_start}"/><stop offset="100%" stop-color="{color_end}"/></radialGradient>"#
                    ));
                }
                GradientConfig::Linear { angle, color_start, color_end } => {
                    let rad = (*angle - 90.0) * (std::f32::consts::PI / 180.0);
                    let cx = qr_width / 2.0;
                    let cy = qr_width / 2.0;
                    let r = (qr_width / 2.0) * std::f32::consts::SQRT_2;
                    let x1 = (((cx - rad.cos() * r) / qr_width) * 100.0).clamp(0.0, 100.0);
                    let y1 = (((cy - rad.sin() * r) / full_height) * 100.0).clamp(0.0, 100.0);
                    let x2 = (((cx + rad.cos() * r) / qr_width) * 100.0).clamp(0.0, 100.0);
                    let y2 = (((cy + rad.sin() * r) / full_height) * 100.0).clamp(0.0, 100.0);
                    defs.push_str(&format!(
                        r#"<linearGradient id="qr-grad" x1="{x1:.2}%" y1="{y1:.2}%" x2="{x2:.2}%" y2="{y2:.2}%"><stop offset="0%" stop-color="{color_start}"/><stop offset="100%" stop-color="{color_end}"/></linearGradient>"#
                    ));
                }
            }
            "url(#qr-grad)".to_string()
        } else {
            config.code_color.clone()
        };

        let eye_fill = config.eye_color.clone().unwrap_or_else(|| body_fill.clone());
        let eye_dot_fill = config.eye_dot_color.clone().unwrap_or_else(|| eye_fill.clone());

        // Logo cutout geometry
        let logo_info = config.logo.as_ref().map(|l| {
            let scale_pct = l.scale_percent.clamp(15.0, 30.0) / 100.0;
            let pixel_size = (size as f32 * scale * scale_pct).round();
            let center_x = padding + (size as f32 * scale) / 2.0;
            let center_y = padding + (size as f32 * scale) / 2.0;
            let radius = pixel_size / 2.0;
            let x = center_x - radius;
            let y = center_y - radius;
            let pad = scale * 0.4;
            (l, pixel_size, center_x, center_y, radius, x, y, pad)
        });

        if let Some((logo, pixel_size, cx, cy, radius, x, y, pad)) = &logo_info {
            let inner_radius = (radius - pad).max(0.0);
            let inner_size = (pixel_size - 2.0 * pad).max(0.0);
            let inner_x = x + pad;
            let inner_y = y + pad;
            match logo.shape {
                LogoMaskShape::Circle => {
                    defs.push_str(&format!(
                        r#"<clipPath id="logo-clip"><circle cx="{cx}" cy="{cy}" r="{inner_radius}"/></clipPath>"#
                    ));
                }
                LogoMaskShape::Rounded => {
                    let rx = scale * 0.6;
                    defs.push_str(&format!(
                        r#"<clipPath id="logo-clip"><rect x="{inner_x}" y="{inner_y}" width="{inner_size}" height="{inner_size}" rx="{rx}" ry="{rx}"/></clipPath>"#
                    ));
                }
                LogoMaskShape::Square => {
                    defs.push_str(&format!(
                        r#"<clipPath id="logo-clip"><rect x="{inner_x}" y="{inner_y}" width="{inner_size}" height="{inner_size}"/></clipPath>"#
                    ));
                }
            }
        }

        let is_inside_logo = |mx: usize, my: usize| -> bool {
            if let Some((logo, _, cx, cy, radius, _, _, _)) = &logo_info {
                let module_cx = padding + mx as f32 * scale + scale / 2.0;
                let module_cy = padding + my as f32 * scale + scale / 2.0;
                let dx = module_cx - cx;
                let dy = module_cy - cy;
                match logo.shape {
                    LogoMaskShape::Circle => (dx * dx + dy * dy) <= (radius * radius),
                    LogoMaskShape::Rounded => {
                        let rx = scale * 0.8;
                        let abs_x = dx.abs();
                        let abs_y = dy.abs();
                        if abs_x > *radius || abs_y > *radius {
                            false
                        } else if abs_x <= radius - rx || abs_y <= radius - rx {
                            true
                        } else {
                            let cdx = abs_x - (radius - rx);
                            let cdy = abs_y - (radius - rx);
                            (cdx * cdx + cdy * cdy) <= (rx * rx)
                        }
                    }
                    LogoMaskShape::Square => dx.abs() <= *radius && dy.abs() <= *radius,
                }
            } else {
                false
            }
        };

        let is_active = |mx: usize, my: usize| -> bool {
            if matrix.is_in_finder(mx, my) {
                return false;
            }
            if !matrix.is_dark(mx, my) {
                return false;
            }
            if is_inside_logo(mx, my) {
                return false;
            }
            true
        };

        let mut elements = String::new();

        // 1. Background
        if let Some(bg) = &config.bg_color {
            elements.push_str(&format!(
                r#"<rect width="{qr_width}" height="{full_height}" fill="{bg}"/>"#
            ));
        }

        // 2. Body modules (combined into compound path)
        let mut body_path = String::new();
        for y in 0..size {
            for x in 0..size {
                if !is_active(x, y) {
                    continue;
                }

                let px = padding + x as f32 * scale;
                let py = padding + y as f32 * scale;

                match config.module_shape {
                    ModuleShape::Dots => {
                        let cx = px + scale / 2.0;
                        let cy = py + scale / 2.0;
                        let r = scale * 0.44;
                        let r2 = 2.0 * r;
                        body_path.push_str(&format!(
                            "M {x:.2} {cy:.2} a {r:.2} {r:.2} 0 1 0 {r2:.2} 0 a {r:.2} {r:.2} 0 1 0 -{r2:.2} 0 Z ",
                            x = cx - r
                        ));
                    }
                    ModuleShape::Smooth => {
                        let top = y > 0 && is_active(x, y - 1);
                        let right = x + 1 < size && is_active(x + 1, y);
                        let bottom = y + 1 < size && is_active(x, y + 1);
                        let left = x > 0 && is_active(x - 1, y);
                        let r = scale * 0.45;

                        let r_tl = if !top && !left { r } else { 0.0 };
                        let r_tr = if !top && !right { r } else { 0.0 };
                        let r_br = if !bottom && !right { r } else { 0.0 };
                        let r_bl = if !bottom && !left { r } else { 0.0 };

                        if r_tl == 0.0 && r_tr == 0.0 && r_br == 0.0 && r_bl == 0.0 {
                            body_path.push_str(&format!(
                                "M {px:.2} {py:.2} h {scale:.2} v {scale:.2} h -{scale:.2} Z "
                            ));
                        } else {
                            let mut d = format!("M {x:.2} {y:.2}", x = px + r_tl, y = py);
                            d.push_str(&format!(" h {:.2}", scale - r_tl - r_tr));
                            if r_tr > 0.0 {
                                d.push_str(&format!(" a {r_tr:.2} {r_tr:.2} 0 0 1 {r_tr:.2} {r_tr:.2}"));
                            }
                            d.push_str(&format!(" v {:.2}", scale - r_tr - r_br));
                            if r_br > 0.0 {
                                d.push_str(&format!(" a {r_br:.2} {r_br:.2} 0 0 1 -{r_br:.2} {r_br:.2}"));
                            }
                            d.push_str(&format!(" h -{:.2}", scale - r_br - r_bl));
                            if r_bl > 0.0 {
                                d.push_str(&format!(" a {r_bl:.2} {r_bl:.2} 0 0 1 -{r_bl:.2} -{r_bl:.2}"));
                            }
                            d.push_str(&format!(" v -{:.2}", scale - r_bl - r_tl));
                            if r_tl > 0.0 {
                                d.push_str(&format!(" a {r_tl:.2} {r_tl:.2} 0 0 1 {r_tl:.2} -{r_tl:.2}"));
                            }
                            d.push_str(" Z ");
                            body_path.push_str(&d);
                        }
                    }
                    ModuleShape::Classy => {
                        let top = y > 0 && is_active(x, y - 1);
                        let right = x + 1 < size && is_active(x + 1, y);
                        let bottom = y + 1 < size && is_active(x, y + 1);
                        let left = x > 0 && is_active(x - 1, y);
                        let r = scale / 2.0;

                        let (r_tl, r_tr, r_br, r_bl) = if left || right || top || bottom {
                            if left || top {
                                if right || bottom {
                                    (0.0, 0.0, 0.0, 0.0)
                                } else {
                                    (0.0, 0.0, r, 0.0)
                                }
                            } else {
                                (r, 0.0, 0.0, 0.0)
                            }
                        } else {
                            (r, 0.0, r, 0.0)
                        };

                        if r_tl == 0.0 && r_tr == 0.0 && r_br == 0.0 && r_bl == 0.0 {
                            body_path.push_str(&format!(
                                "M {px:.2} {py:.2} h {scale:.2} v {scale:.2} h -{scale:.2} Z "
                            ));
                        } else {
                            let mut d = format!("M {x:.2} {y:.2}", x = px + r_tl, y = py);
                            d.push_str(&format!(" h {:.2}", scale - r_tl - r_tr));
                            if r_tr > 0.0 {
                                d.push_str(&format!(" a {r_tr:.2} {r_tr:.2} 0 0 1 {r_tr:.2} {r_tr:.2}"));
                            }
                            d.push_str(&format!(" v {:.2}", scale - r_tr - r_br));
                            if r_br > 0.0 {
                                d.push_str(&format!(" a {r_br:.2} {r_br:.2} 0 0 1 -{r_br:.2} {r_br:.2}"));
                            }
                            d.push_str(&format!(" h -{:.2}", scale - r_br - r_bl));
                            if r_bl > 0.0 {
                                d.push_str(&format!(" a {r_bl:.2} {r_bl:.2} 0 0 1 -{r_bl:.2} -{r_bl:.2}"));
                            }
                            d.push_str(&format!(" v -{:.2}", scale - r_bl - r_tl));
                            if r_tl > 0.0 {
                                d.push_str(&format!(" a {r_tl:.2} {r_tl:.2} 0 0 1 {r_tl:.2} -{r_tl:.2}"));
                            }
                            d.push_str(" Z ");
                            body_path.push_str(&d);
                        }
                    }
                    ModuleShape::Square => {
                        body_path.push_str(&format!(
                            "M {px:.2} {py:.2} h {scale:.2} v {scale:.2} h -{scale:.2} Z "
                        ));
                    }
                }
            }
        }

        if !body_path.is_empty() {
            elements.push_str(&format!(r#"<path d="{}" fill="{body_fill}"/>"#, body_path.trim_end()));
        }

        // 3. Eye frames & dots (combined compound paths)
        let mut eye_frame_path = String::new();
        let mut eye_dot_path = String::new();
        let finders = [(0, 0), (size - 7, 0), (0, size - 7)];

        for &(fx, fy) in &finders {
            let px = padding + fx as f32 * scale;
            let py = padding + fy as f32 * scale;

            // Outer eye frame
            match config.eye_frame_shape {
                EyeFrameShape::Circle => {
                    let cx = px + 3.5 * scale;
                    let ro = 3.5 * scale;
                    let ri = 2.5 * scale;
                    eye_frame_path.push_str(&format!(
                        "M {cx} {py} a {ro} {ro} 0 1 0 0.001 0 Z M {cx} {py_inner} a {ri} {ri} 0 1 1 -0.001 0 Z ",
                        py_inner = py + scale
                    ));
                }
                EyeFrameShape::Rounded => {
                    let ro = 2.4 * scale;
                    let ri = 1.4 * scale;
                    let outer_d = format!(
                        "M {px_ro} {py} h {w_out} a {ro} {ro} 0 0 1 {ro} {ro} v {w_out} a {ro} {ro} 0 0 1 -{ro} {ro} h -{w_out} a {ro} {ro} 0 0 1 -{ro} -{ro} v -{w_out} a {ro} {ro} 0 0 1 {ro} -{ro} Z",
                        px_ro = px + ro,
                        w_out = 7.0 * scale - 2.0 * ro
                    );
                    let inner_d = format!(
                        "M {px_ri} {py_ri} h {w_in} a {ri} {ri} 0 0 1 {ri} {ri} v {w_in} a {ri} {ri} 0 0 1 -{ri} {ri} h -{w_in} a {ri} {ri} 0 0 1 -{ri} -{ri} v -{w_in} a {ri} {ri} 0 0 1 {ri} -{ri} Z",
                        px_ri = px + scale + ri,
                        py_ri = py + scale,
                        w_in = 5.0 * scale - 2.0 * ri
                    );
                    eye_frame_path.push_str(&format!("{outer_d} {inner_d} "));
                }
                EyeFrameShape::Square => {
                    let d = format!(
                        "M {px} {py} h {outer} v {outer} h -{outer} Z M {inner_x} {inner_y} v {inner} h {inner} v -{inner} Z ",
                        outer = 7.0 * scale,
                        inner_x = px + scale,
                        inner_y = py + scale,
                        inner = 5.0 * scale
                    );
                    eye_frame_path.push_str(&d);
                }
            }

            // Inner eye dot
            let dot_x = px + 2.0 * scale;
            let dot_y = py + 2.0 * scale;
            let dot_size = 3.0 * scale;

            match config.eye_dot_shape {
                EyeDotShape::Circle => {
                    let cx = dot_x + dot_size / 2.0;
                    let cy = dot_y + dot_size / 2.0;
                    let r = dot_size / 2.0;
                    let r2 = 2.0 * r;
                    eye_dot_path.push_str(&format!(
                        "M {x:.2} {cy:.2} a {r:.2} {r:.2} 0 1 0 {r2:.2} 0 a {r:.2} {r:.2} 0 1 0 -{r2:.2} 0 Z ",
                        x = cx - r
                    ));
                }
                EyeDotShape::Rounded => {
                    let r = scale * 0.9;
                    let w = dot_size - 2.0 * r;
                    eye_dot_path.push_str(&format!(
                        "M {x:.2} {dot_y:.2} h {w:.2} a {r:.2} {r:.2} 0 0 1 {r:.2} {r:.2} v {w:.2} a {r:.2} {r:.2} 0 0 1 -{r:.2} {r:.2} h -{w:.2} a {r:.2} {r:.2} 0 0 1 -{r:.2} -{r:.2} v -{w:.2} a {r:.2} {r:.2} 0 0 1 {r:.2} -{r:.2} Z ",
                        x = dot_x + r
                    ));
                }
                EyeDotShape::Square => {
                    eye_dot_path.push_str(&format!(
                        "M {dot_x:.2} {dot_y:.2} h {dot_size:.2} v {dot_size:.2} h -{dot_size:.2} Z "
                    ));
                }
            }
        }

        if !eye_frame_path.is_empty() {
            elements.push_str(&format!(
                r#"<path fill-rule="evenodd" d="{}" fill="{eye_fill}"/>"#,
                eye_frame_path.trim_end()
            ));
        }

        if !eye_dot_path.is_empty() {
            elements.push_str(&format!(
                r#"<path d="{}" fill="{eye_dot_fill}"/>"#,
                eye_dot_path.trim_end()
            ));
        }

        // 4. Center Logo
        if let Some((logo, pixel_size, cx, cy, radius, x, y, pad)) = &logo_info {
            let bg_color = config.bg_color.as_deref().unwrap_or("#04070d");
            match logo.shape {
                LogoMaskShape::Circle => {
                    elements.push_str(&format!(r#"<circle cx="{cx}" cy="{cy}" r="{radius}" fill="{bg_color}"/>"#));
                }
                LogoMaskShape::Rounded => {
                    let rx = scale * 0.8;
                    elements.push_str(&format!(
                        r#"<rect x="{x}" y="{y}" width="{pixel_size}" height="{pixel_size}" rx="{rx}" ry="{rx}" fill="{bg_color}"/>"#
                    ));
                }
                LogoMaskShape::Square => {
                    elements.push_str(&format!(
                        r#"<rect x="{x}" y="{y}" width="{pixel_size}" height="{pixel_size}" fill="{bg_color}"/>"#
                    ));
                }
            }
            let img_x = x + pad;
            let img_y = y + pad;
            let img_size = pixel_size - 2.0 * pad;
            let href = html_escape(&logo.data_url);
            elements.push_str(&format!(
                r#"<image href="{href}" x="{img_x}" y="{img_y}" width="{img_size}" height="{img_size}" clip-path="url(#logo-clip)"/>"#
            ));
        }

        // 5. CTA Text Banner
        if let Some(text) = cta_text {
            let text_y = qr_width + (cta_height / 2.0);
            let text_color = config.cta.as_ref().map(|c| c.color.as_str()).unwrap_or("#3fd9ff");
            let escaped = html_escape(&text);
            elements.push_str(&format!(
                r#"<text x="{}" y="{}" fill="{}" font-family="'Maple Mono NF', 'Maple Mono', 'JetBrains Mono', monospace" font-weight="bold" font-size="{}" text-anchor="middle" dominant-baseline="central" letter-spacing="1.5">{}</text>"#,
                qr_width / 2.0,
                text_y,
                text_color,
                target_font_size,
                escaped
            ));
        }

        let title_desc = if let Some(alt) = &config.alt_text {
            let esc = html_escape(alt);
            format!("<title>{esc}</title><desc>{esc}</desc>")
        } else {
            String::new()
        };

        let defs_tag = if !defs.is_empty() {
            format!("<defs>{defs}</defs>")
        } else {
            String::new()
        };

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {qr_width} {full_height}" width="{qr_width}" height="{full_height}" style="width:100%;height:auto;max-width:100%;display:block;">{title_desc}{defs_tag}{elements}</svg>"#
        )
    }
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::EccLevel;

    #[test]
    fn test_svg_generation() {
        let matrix = QrMatrix::new("https://qr.den1zz.dev", EccLevel::M, false).unwrap();
        let config = VectorRenderConfig::default();
        let svg = VectorRenderer::render_svg(&matrix, &config);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("fill=\"#000000\""));
        assert!(svg.contains("fill=\"#008b99\""));
    }

    #[test]
    fn test_classy_shape_generation() {
        let matrix = QrMatrix::new("TEST CLASSY", EccLevel::M, false).unwrap();
        let mut config = VectorRenderConfig::default();
        config.module_shape = ModuleShape::Classy;
        let svg = VectorRenderer::render_svg(&matrix, &config);
        assert!(svg.contains("<path d=\"M "));
    }
}
