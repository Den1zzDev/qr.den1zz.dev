use crate::matrix::QrMatrix;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

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
    Linear {
        angle: f32,
        color_start: String,
        color_end: String,
    },
    Radial {
        color_start: String,
        color_end: String,
    },
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

        let mut defs = String::with_capacity(256);
        let body_fill = if let Some(grad) = &config.gradient {
            match grad {
                GradientConfig::Radial {
                    color_start,
                    color_end,
                } => {
                    let cy_pct = ((qr_width / 2.0) / full_height) * 100.0;
                    let r_pct = 50.0;
                    let _ = write!(
                        defs,
                        r#"<radialGradient id="qr-grad" cx="50%" cy="{cy_pct:.2}%" r="{r_pct:.2}%"><stop offset="0%" stop-color="{color_start}"/><stop offset="100%" stop-color="{color_end}"/></radialGradient>"#
                    );
                }
                GradientConfig::Linear {
                    angle,
                    color_start,
                    color_end,
                } => {
                    let rad = (*angle - 90.0) * (std::f32::consts::PI / 180.0);
                    let cx = qr_width / 2.0;
                    let cy = qr_width / 2.0;
                    let r = (qr_width / 2.0) * std::f32::consts::SQRT_2;
                    let x1 = (((cx - rad.cos() * r) / qr_width) * 100.0).clamp(0.0, 100.0);
                    let y1 = (((cy - rad.sin() * r) / full_height) * 100.0).clamp(0.0, 100.0);
                    let x2 = (((cx + rad.cos() * r) / qr_width) * 100.0).clamp(0.0, 100.0);
                    let y2 = (((cy + rad.sin() * r) / full_height) * 100.0).clamp(0.0, 100.0);
                    let _ = write!(
                        defs,
                        r#"<linearGradient id="qr-grad" x1="{x1:.2}%" y1="{y1:.2}%" x2="{x2:.2}%" y2="{y2:.2}%"><stop offset="0%" stop-color="{color_start}"/><stop offset="100%" stop-color="{color_end}"/></linearGradient>"#
                    );
                }
            }
            "url(#qr-grad)".to_string()
        } else {
            config.code_color.clone()
        };

        let eye_fill = config
            .eye_color
            .clone()
            .unwrap_or_else(|| body_fill.clone());
        let eye_dot_fill = config
            .eye_dot_color
            .clone()
            .unwrap_or_else(|| eye_fill.clone());

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
                    let _ = write!(
                        defs,
                        r#"<clipPath id="logo-clip"><circle cx="{cx}" cy="{cy}" r="{inner_radius}"/></clipPath>"#
                    );
                }
                LogoMaskShape::Rounded => {
                    let rx = scale * 0.6;
                    let _ = write!(
                        defs,
                        r#"<clipPath id="logo-clip"><rect x="{inner_x}" y="{inner_y}" width="{inner_size}" height="{inner_size}" rx="{rx}" ry="{rx}"/></clipPath>"#
                    );
                }
                LogoMaskShape::Square => {
                    let _ = write!(
                        defs,
                        r#"<clipPath id="logo-clip"><rect x="{inner_x}" y="{inner_y}" width="{inner_size}" height="{inner_size}"/></clipPath>"#
                    );
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

        let mut elements = String::with_capacity(size * size * 32);

        // 1. Background
        if let Some(bg) = &config.bg_color {
            let _ = write!(
                elements,
                r#"<rect width="{qr_width}" height="{full_height}" fill="{bg}"/>"#
            );
        }

        // 2. Body modules (combined into compound path)
        let mut body_path = String::with_capacity(size * size * 24);

        if config.module_shape == ModuleShape::Square {
            // Contiguous horizontal run merging for ModuleShape::Square
            for y in 0..size {
                let py = padding + y as f32 * scale;
                let mut x = 0;
                while x < size {
                    if is_active(x, y) {
                        let run_start = x;
                        while x < size && is_active(x, y) {
                            x += 1;
                        }
                        let run_len = x - run_start;
                        let px = padding + run_start as f32 * scale;
                        let width = run_len as f32 * scale;
                        let _ = write!(
                            body_path,
                            "M {:.2} {:.2} h {:.2} v {:.2} h -{:.2} Z ",
                            px, py, width, scale, width
                        );
                    } else {
                        x += 1;
                    }
                }
            }
        } else {
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
                            let _ = write!(
                                body_path,
                                "M {x:.2} {cy:.2} a {r:.2} {r:.2} 0 1 0 {r2:.2} 0 a {r:.2} {r:.2} 0 1 0 -{r2:.2} 0 Z ",
                                x = cx - r,
                                cy = cy,
                                r = r,
                                r2 = r2
                            );
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
                                let _ = write!(
                                    body_path,
                                    "M {:.2} {:.2} h {:.2} v {:.2} h -{:.2} Z ",
                                    px, py, scale, scale, scale
                                );
                            } else {
                                let _ = write!(
                                    body_path,
                                    "M {:.2} {:.2} h {:.2}",
                                    px + r_tl,
                                    py,
                                    scale - r_tl - r_tr
                                );
                                if r_tr > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 {:.2} {:.2}",
                                        r_tr, r_tr, r_tr, r_tr
                                    );
                                }
                                let _ = write!(body_path, " v {:.2}", scale - r_tr - r_br);
                                if r_br > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 -{:.2} {:.2}",
                                        r_br, r_br, r_br, r_br
                                    );
                                }
                                let _ = write!(body_path, " h -{:.2}", scale - r_br - r_bl);
                                if r_bl > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 -{:.2} -{:.2}",
                                        r_bl, r_bl, r_bl, r_bl
                                    );
                                }
                                let _ = write!(body_path, " v -{:.2}", scale - r_bl - r_tl);
                                if r_tl > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 {:.2} -{:.2}",
                                        r_tl, r_tl, r_tl, r_tl
                                    );
                                }
                                body_path.push_str(" Z ");
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
                                let _ = write!(
                                    body_path,
                                    "M {:.2} {:.2} h {:.2} v {:.2} h -{:.2} Z ",
                                    px, py, scale, scale, scale
                                );
                            } else {
                                let _ = write!(
                                    body_path,
                                    "M {:.2} {:.2} h {:.2}",
                                    px + r_tl,
                                    py,
                                    scale - r_tl - r_tr
                                );
                                if r_tr > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 {:.2} {:.2}",
                                        r_tr, r_tr, r_tr, r_tr
                                    );
                                }
                                let _ = write!(body_path, " v {:.2}", scale - r_tr - r_br);
                                if r_br > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 -{:.2} {:.2}",
                                        r_br, r_br, r_br, r_br
                                    );
                                }
                                let _ = write!(body_path, " h -{:.2}", scale - r_br - r_bl);
                                if r_bl > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 -{:.2} -{:.2}",
                                        r_bl, r_bl, r_bl, r_bl
                                    );
                                }
                                let _ = write!(body_path, " v -{:.2}", scale - r_bl - r_tl);
                                if r_tl > 0.0 {
                                    let _ = write!(
                                        body_path,
                                        " a {:.2} {:.2} 0 0 1 {:.2} -{:.2}",
                                        r_tl, r_tl, r_tl, r_tl
                                    );
                                }
                                body_path.push_str(" Z ");
                            }
                        }
                        ModuleShape::Square => unreachable!(),
                    }
                }
            }
        }

        if !body_path.is_empty() {
            let _ = write!(
                elements,
                r#"<path d="{}" fill="{body_fill}"/>"#,
                body_path.trim_end()
            );
        }

        // 3. Eye frames & dots (combined compound paths)
        let mut eye_frame_path = String::with_capacity(512);
        let mut eye_dot_path = String::with_capacity(256);
        let finders = [(0, 0), (size - 7, 0), (0, size - 7)];

        for &(fx, fy) in &finders {
            let ox = padding + fx as f32 * scale;
            let oy = padding + fy as f32 * scale;
            let frame_size = 7.0 * scale;

            // Eye Frame (Outer 7x7 with inner 5x5 cutout using evenodd)
            match config.eye_frame_shape {
                EyeFrameShape::Circle => {
                    let cx = ox + frame_size / 2.0;
                    let cy = oy + frame_size / 2.0;
                    let r_outer = frame_size / 2.0;
                    let r_inner = frame_size / 2.0 - scale;
                    let r2_out = 2.0 * r_outer;
                    let r2_in = 2.0 * r_inner;
                    let _ = write!(
                        eye_frame_path,
                        "M {ox_c:.2} {cy:.2} a {r_outer:.2} {r_outer:.2} 0 1 0 {r2_out:.2} 0 a {r_outer:.2} {r_outer:.2} 0 1 0 -{r2_out:.2} 0 Z M {ix_c:.2} {cy:.2} a {r_inner:.2} {r_inner:.2} 0 1 0 {r2_in:.2} 0 a {r_inner:.2} {r_inner:.2} 0 1 0 -{r2_in:.2} 0 Z ",
                        ox_c = cx - r_outer,
                        ix_c = cx - r_inner,
                        cy = cy,
                        r_outer = r_outer,
                        r_inner = r_inner,
                        r2_out = r2_out,
                        r2_in = r2_in
                    );
                }
                EyeFrameShape::Rounded => {
                    let r = scale * 1.8;
                    let r_in = (r - scale).max(0.0);
                    let w = frame_size - 2.0 * r;
                    let w_in = (frame_size - 2.0 * scale) - 2.0 * r_in;

                    let _ = write!(
                        eye_frame_path,
                        "M {:.2} {oy:.2} h {w:.2} a {r:.2} {r:.2} 0 0 1 {r:.2} {r:.2} v {w:.2} a {r:.2} {r:.2} 0 0 1 -{r:.2} {r:.2} h -{w:.2} a {r:.2} {r:.2} 0 0 1 -{r:.2} -{r:.2} v -{w:.2} a {r:.2} {r:.2} 0 0 1 {r:.2} -{r:.2} Z ",
                        ox + r
                    );
                    let inner_y = oy + scale;
                    let _ = write!(
                        eye_frame_path,
                        "M {:.2} {inner_y:.2} h {w_in:.2} a {r_in:.2} {r_in:.2} 0 0 1 {r_in:.2} {r_in:.2} v {w_in:.2} a {r_in:.2} {r_in:.2} 0 0 1 -{r_in:.2} {r_in:.2} h -{w_in:.2} a {r_in:.2} {r_in:.2} 0 0 1 -{r_in:.2} -{r_in:.2} v -{w_in:.2} a {r_in:.2} {r_in:.2} 0 0 1 {r_in:.2} -{r_in:.2} Z ",
                        ox + scale + r_in
                    );
                }
                EyeFrameShape::Square => {
                    let inner_w = frame_size - 2.0 * scale;
                    let _ = write!(
                        eye_frame_path,
                        "M {ox:.2} {oy:.2} h {frame_size:.2} v {frame_size:.2} h -{frame_size:.2} Z M {:.2} {:.2} h {inner_w:.2} v {inner_w:.2} h -{inner_w:.2} Z ",
                        ox + scale, oy + scale
                    );
                }
            }

            // Eye Dot (Center 3x3)
            let dot_x = ox + 2.0 * scale;
            let dot_y = oy + 2.0 * scale;
            let dot_size = 3.0 * scale;

            match config.eye_dot_shape {
                EyeDotShape::Circle => {
                    let cx = dot_x + dot_size / 2.0;
                    let cy = dot_y + dot_size / 2.0;
                    let r = dot_size / 2.0;
                    let r2 = 2.0 * r;
                    let _ = write!(
                        eye_dot_path,
                        "M {x:.2} {cy:.2} a {r:.2} {r:.2} 0 1 0 {r2:.2} 0 a {r:.2} {r:.2} 0 1 0 -{r2:.2} 0 Z ",
                        x = cx - r,
                        cy = cy,
                        r = r,
                        r2 = r2
                    );
                }
                EyeDotShape::Rounded => {
                    let r = scale * 0.9;
                    let w = dot_size - 2.0 * r;
                    let _ = write!(
                        eye_dot_path,
                        "M {:.2} {dot_y:.2} h {w:.2} a {r:.2} {r:.2} 0 0 1 {r:.2} {r:.2} v {w:.2} a {r:.2} {r:.2} 0 0 1 -{r:.2} {r:.2} h -{w:.2} a {r:.2} {r:.2} 0 0 1 -{r:.2} -{r:.2} v -{w:.2} a {r:.2} {r:.2} 0 0 1 {r:.2} -{r:.2} Z ",
                        dot_x + r
                    );
                }
                EyeDotShape::Square => {
                    let _ = write!(
                        eye_dot_path,
                        "M {dot_x:.2} {dot_y:.2} h {dot_size:.2} v {dot_size:.2} h -{dot_size:.2} Z "
                    );
                }
            }
        }

        if !eye_frame_path.is_empty() {
            let _ = write!(
                elements,
                r#"<path fill-rule="evenodd" d="{}" fill="{eye_fill}"/>"#,
                eye_frame_path.trim_end()
            );
        }

        if !eye_dot_path.is_empty() {
            let _ = write!(
                elements,
                r#"<path d="{}" fill="{eye_dot_fill}"/>"#,
                eye_dot_path.trim_end()
            );
        }

        // 4. Center Logo
        if let Some((logo, pixel_size, cx, cy, radius, x, y, pad)) = &logo_info {
            let bg_color = config.bg_color.as_deref().unwrap_or("#04070d");
            match logo.shape {
                LogoMaskShape::Circle => {
                    let _ = write!(
                        elements,
                        r#"<circle cx="{cx}" cy="{cy}" r="{radius}" fill="{bg_color}"/>"#
                    );
                }
                LogoMaskShape::Rounded => {
                    let rx = scale * 0.8;
                    let _ = write!(
                        elements,
                        r#"<rect x="{x}" y="{y}" width="{pixel_size}" height="{pixel_size}" rx="{rx}" ry="{rx}" fill="{bg_color}"/>"#
                    );
                }
                LogoMaskShape::Square => {
                    let _ = write!(
                        elements,
                        r#"<rect x="{x}" y="{y}" width="{pixel_size}" height="{pixel_size}" fill="{bg_color}"/>"#
                    );
                }
            }
            let inner_size = (pixel_size - 2.0 * pad).max(0.0);
            let inner_x = x + pad;
            let inner_y = y + pad;
            let _ = write!(
                elements,
                r#"<image href="{}" x="{inner_x}" y="{inner_y}" width="{inner_size}" height="{inner_size}" preserveAspectRatio="xMidYMid slice" clip-path="url(#logo-clip)"/>"#,
                logo.data_url
            );
        }

        // 5. CTA Banner
        if let Some(cta_text) = &cta_text {
            let text_y = qr_width + cta_height * 0.65;
            let cta_color = config
                .cta
                .as_ref()
                .map(|c| c.color.as_str())
                .unwrap_or("#ffffff");
            let _ = write!(
                elements,
                r#"<text x="50%" y="{text_y}" text-anchor="middle" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="{target_font_size}px" font-weight="bold" fill="{cta_color}" letter-spacing="1.5">{cta_text}</text>"#
            );
        }

        let defs_tag = if defs.is_empty() {
            String::new()
        } else {
            format!("<defs>{defs}</defs>")
        };

        let alt = config.alt_text.as_deref().unwrap_or("QR Code");

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {qr_width} {full_height}" width="{qr_width}" height="{full_height}" shape-rendering="geometricPrecision"><title>{alt}</title>{defs_tag}{elements}</svg>"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::EccLevel;

    #[test]
    fn test_svg_generation() {
        let matrix = QrMatrix::new("https://den1zz.dev", EccLevel::M, false).unwrap();
        let config = VectorRenderConfig::default();
        let svg = VectorRenderer::render_svg(&matrix, &config);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("path"));
    }

    #[test]
    fn test_square_shape_horizontal_merging() {
        let matrix = QrMatrix::new("https://den1zz.dev", EccLevel::M, false).unwrap();
        let config = VectorRenderConfig {
            module_shape: ModuleShape::Square,
            scale: 10.0,
            margin_modules: 0,
            ..Default::default()
        };
        let svg = VectorRenderer::render_svg(&matrix, &config);
        // Ensure merged runs exist (i.e. horizontal width is a multiple of scale > 10.0, e.g. 20.00, 30.00, 70.00)
        assert!(svg.contains("h 20.00") || svg.contains("h 30.00") || svg.contains("h 40.00"));
    }

    #[test]
    fn test_classy_shape_generation() {
        let matrix = QrMatrix::new("https://den1zz.dev", EccLevel::M, false).unwrap();
        let config = VectorRenderConfig {
            module_shape: ModuleShape::Classy,
            ..Default::default()
        };
        let svg = VectorRenderer::render_svg(&matrix, &config);
        assert!(svg.contains("<path"));
    }
}
