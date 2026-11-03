//! SVG rendering of QR matrices.
//!
//! [`render_svg`] turns a [`QrMatrix`] into a standalone SVG document with
//! configurable module shapes, finder patterns, gradients, backgrounds, centre
//! logos, and an optional call-to-action banner.

use std::f32::consts;
use std::fmt::Write as _;

use serde::Deserialize;
use serde::Serialize;

use crate::matrix::QrMatrix;

/// Shape of a single data module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ModuleShape {
    /// Axis-aligned square modules.
    Square,
    /// Modules with corners rounded where they do not touch a neighbour.
    Smooth,
    /// Circular modules.
    Dots,
    /// Modules with one rounded corner, producing a diagonal "classy" look.
    Classy,
}

/// Shape of the outer finder pattern frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum EyeFrameShape {
    /// Axis-aligned square frame.
    Square,
    /// Frame with rounded corners.
    Rounded,
    /// Circular frame.
    Circle,
}

/// Shape of the centre dot inside a finder pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum EyeDotShape {
    /// Axis-aligned square dot.
    Square,
    /// Dot with rounded corners.
    Rounded,
    /// Circular dot.
    Circle,
}

/// Fill applied to the data modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum GradientConfig {
    /// Linear gradient at a fixed angle in degrees.
    Linear {
        /// Gradient direction in degrees.
        angle: f32,
        /// Colour at offset 0%.
        color_start: String,
        /// Colour at offset 100%.
        color_end: String,
    },
    /// Radial gradient centred on the code.
    Radial {
        /// Colour at offset 0%.
        color_start: String,
        /// Colour at offset 100%.
        color_end: String,
    },
}

/// Mask shape applied to a centre logo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LogoMaskShape {
    /// Square mask.
    Square,
    /// Mask with rounded corners.
    Rounded,
    /// Circular mask.
    Circle,
}

/// Centre logo configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogoConfig {
    /// Data URL containing the image.
    pub data_url: String,
    /// Shape of the mask and the background cut-out.
    pub shape: LogoMaskShape,
    /// Logo width as a percentage of the code width, clamped to 15-30%.
    pub scale_percent: f32,
}

/// Call-to-action banner rendered below the code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CtaBannerConfig {
    /// Banner text. Rendered in upper case.
    pub text: String,
    /// Font size at the reference width of 340 px.
    pub font_size: f32,
    /// Text colour.
    pub color: String,
}

/// Configuration for [`render_svg`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorRenderConfig {
    /// Size of one module in SVG user units.
    pub scale: f32,
    /// Quiet-zone width in modules.
    pub margin_modules: usize,
    /// Shape of data modules.
    pub module_shape: ModuleShape,
    /// Shape of finder pattern frames.
    pub eye_frame_shape: EyeFrameShape,
    /// Shape of finder pattern dots.
    pub eye_dot_shape: EyeDotShape,
    /// Fill colour for data modules, used when `gradient` is `None`.
    pub code_color: String,
    /// Optional gradient that replaces `code_color`.
    pub gradient: Option<GradientConfig>,
    /// Optional finder frame colour. Defaults to the module fill.
    pub eye_color: Option<String>,
    /// Optional finder dot colour. Defaults to the frame colour.
    #[serde(default)]
    pub eye_dot_color: Option<String>,
    /// Optional background colour. `None` produces a transparent background.
    pub bg_color: Option<String>,
    /// Optional centre logo.
    pub logo: Option<LogoConfig>,
    /// Optional call-to-action banner.
    pub cta: Option<CtaBannerConfig>,
    /// Accessible title for the generated SVG.
    pub alt_text: Option<String>,
}

impl Default for VectorRenderConfig {
    fn default() -> Self {
        Self {
            scale: 20.0,
            margin_modules: 4,
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

/// Resolved geometry of the optional call-to-action banner.
struct CtaLayout {
    text: Option<String>,
    height: f32,
    font_size: f32,
}

impl CtaLayout {
    /// Resolves the banner geometry for a QR code of `qr_width` user units.
    fn new(cta: &CtaBannerConfig, qr_width: f32, scale: f32) -> Self {
        let text = cta.text.trim().to_uppercase();
        if text.is_empty() {
            return Self::hidden();
        }

        let scale_factor = qr_width / 340.0;
        let font_size = (cta.font_size * scale_factor).max(10.0);
        let height = (scale * 3.2).max(font_size * 2.2);
        Self {
            text: Some(text),
            height,
            font_size,
        }
    }

    /// A layout with no visible banner.
    fn hidden() -> Self {
        Self {
            text: None,
            height: 0.0,
            font_size: 0.0,
        }
    }
}

/// Resolved geometry of the optional centre logo.
struct LogoGeometry<'config> {
    config: &'config LogoConfig,
    pixel_size: f32,
    center_x: f32,
    center_y: f32,
    radius: f32,
    x: f32,
    y: f32,
    inset: f32,
}

impl<'config> LogoGeometry<'config> {
    /// Resolves the logo geometry for a matrix of `size` modules.
    fn new(config: &'config LogoConfig, size: usize, scale: f32, padding: f32) -> Self {
        let scale_percent = config.scale_percent.clamp(15.0, 30.0) / 100.0;
        let pixel_size = (size as f32 * scale * scale_percent).round();
        let center_x = padding + (size as f32 * scale) / 2.0;
        let center_y = padding + (size as f32 * scale) / 2.0;
        let radius = pixel_size / 2.0;
        let x = center_x - radius;
        let y = center_y - radius;

        Self {
            config,
            pixel_size,
            center_x,
            center_y,
            radius,
            x,
            y,
            inset: scale * 0.4,
        }
    }
}

/// Appends one module with up to four rounded corners to `path`.
///
/// Corners are ordered top-left, top-right, bottom-right, bottom-left. A radius
/// of `0.0` keeps the corresponding corner square.
fn push_rounded_module(path: &mut String, x: f32, y: f32, scale: f32, corners: [f32; 4]) {
    let [top_left, top_right, bottom_right, bottom_left] = corners;
    if corners.iter().all(|radius| *radius == 0.0) {
        let _ = write!(
            path,
            "M {x:.2} {y:.2} h {scale:.2} v {scale:.2} h -{scale:.2} Z "
        );
        return;
    }

    let _ = write!(
        path,
        "M {:.2} {:.2} h {:.2}",
        x + top_left,
        y,
        scale - top_left - top_right
    );
    if top_right > 0.0 {
        let _ = write!(
            path,
            " a {:.2} {:.2} 0 0 1 {:.2} {:.2}",
            top_right, top_right, top_right, top_right
        );
    }
    let _ = write!(path, " v {:.2}", scale - top_right - bottom_right);
    if bottom_right > 0.0 {
        let _ = write!(
            path,
            " a {:.2} {:.2} 0 0 1 -{:.2} {:.2}",
            bottom_right, bottom_right, bottom_right, bottom_right
        );
    }
    let _ = write!(path, " h -{:.2}", scale - bottom_right - bottom_left);
    if bottom_left > 0.0 {
        let _ = write!(
            path,
            " a {:.2} {:.2} 0 0 1 -{:.2} -{:.2}",
            bottom_left, bottom_left, bottom_left, bottom_left
        );
    }
    let _ = write!(path, " v -{:.2}", scale - bottom_left - top_left);
    if top_left > 0.0 {
        let _ = write!(
            path,
            " a {:.2} {:.2} 0 0 1 {:.2} -{:.2}",
            top_left, top_left, top_left, top_left
        );
    }
    path.push_str(" Z ");
}

/// Appends a circular module to `path`.
fn push_circle(path: &mut String, center_x: f32, center_y: f32, radius: f32) {
    let diameter = 2.0 * radius;
    let _ = write!(
        path,
        "M {:.2} {:.2} a {:.2} {:.2} 0 1 0 {:.2} 0 a {:.2} {:.2} 0 1 0 -{:.2} 0 Z ",
        center_x - radius,
        center_y,
        radius,
        radius,
        diameter,
        radius,
        radius,
        diameter
    );
}

/// Renders `matrix` as a standalone SVG document.
///
/// The returned document has no external dependencies and can be embedded
/// directly in HTML or written to a file.
#[must_use]
pub fn render_svg(matrix: &QrMatrix, config: &VectorRenderConfig) -> String {
    let size = matrix.size;
    let scale = config.scale;
    let padding = config.margin_modules as f32 * scale;
    let qr_width = size as f32 * scale + 2.0 * padding;

    let cta = config.cta.as_ref().map_or_else(CtaLayout::hidden, |cta| {
        CtaLayout::new(cta, qr_width, scale)
    });
    let full_height = qr_width + cta.height;

    let mut defs = String::with_capacity(256);
    let body_fill = if let Some(gradient) = &config.gradient {
        match gradient {
            GradientConfig::Radial {
                color_start,
                color_end,
            } => {
                let center_y = ((qr_width / 2.0) / full_height) * 100.0;
                let radius = 50.0;
                let _ = write!(
                    defs,
                    r#"<radialGradient id="qr-grad" cx="50%" cy="{center_y:.2}%" r="{radius:.2}%"><stop offset="0%" stop-color="{color_start}"/><stop offset="100%" stop-color="{color_end}"/></radialGradient>"#
                );
            }
            GradientConfig::Linear {
                angle,
                color_start,
                color_end,
            } => {
                let radians = (angle - 90.0) * (consts::PI / 180.0);
                let center_x = qr_width / 2.0;
                let center_y = qr_width / 2.0;
                let radius = (qr_width / 2.0) * consts::SQRT_2;
                let x1 =
                    (((center_x - radians.cos() * radius) / qr_width) * 100.0).clamp(0.0, 100.0);
                let y1 =
                    (((center_y - radians.sin() * radius) / full_height) * 100.0).clamp(0.0, 100.0);
                let x2 =
                    (((center_x + radians.cos() * radius) / qr_width) * 100.0).clamp(0.0, 100.0);
                let y2 =
                    (((center_y + radians.sin() * radius) / full_height) * 100.0).clamp(0.0, 100.0);
                let _ = write!(
                    defs,
                    r#"<linearGradient id="qr-grad" x1="{x1:.2}%" y1="{y1:.2}%" x2="{x2:.2}%" y2="{y2:.2}%"><stop offset="0%" stop-color="{color_start}"/><stop offset="100%" stop-color="{color_end}"/></linearGradient>"#
                );
            }
        }
        "url(#qr-grad)"
    } else {
        config.code_color.as_str()
    };

    let eye_fill = config.eye_color.as_deref().unwrap_or(body_fill);
    let eye_dot_fill = config.eye_dot_color.as_deref().unwrap_or(eye_fill);

    let logo = config
        .logo
        .as_ref()
        .map(|logo| LogoGeometry::new(logo, size, scale, padding));

    if let Some(logo) = &logo {
        let inner_radius = (logo.radius - logo.inset).max(0.0);
        let inner_size = (logo.pixel_size - 2.0 * logo.inset).max(0.0);
        let inner_x = logo.x + logo.inset;
        let inner_y = logo.y + logo.inset;

        match logo.config.shape {
            LogoMaskShape::Circle => {
                let _ = write!(
                    defs,
                    r#"<clipPath id="logo-clip"><circle cx="{}" cy="{}" r="{inner_radius}"/></clipPath>"#,
                    logo.center_x, logo.center_y
                );
            }
            LogoMaskShape::Rounded => {
                let corner_radius = scale * 0.6;
                let _ = write!(
                    defs,
                    r#"<clipPath id="logo-clip"><rect x="{inner_x}" y="{inner_y}" width="{inner_size}" height="{inner_size}" rx="{corner_radius}" ry="{corner_radius}"/></clipPath>"#
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

    let is_inside_logo = |module_x: usize, module_y: usize| -> bool {
        let Some(logo) = &logo else {
            return false;
        };

        let module_center_x = padding + module_x as f32 * scale + scale / 2.0;
        let module_center_y = padding + module_y as f32 * scale + scale / 2.0;
        let delta_x = module_center_x - logo.center_x;
        let delta_y = module_center_y - logo.center_y;

        match logo.config.shape {
            LogoMaskShape::Circle => {
                delta_x * delta_x + delta_y * delta_y <= logo.radius * logo.radius
            }
            LogoMaskShape::Rounded => {
                let corner_radius = scale * 0.8;
                let abs_x = delta_x.abs();
                let abs_y = delta_y.abs();
                if abs_x > logo.radius || abs_y > logo.radius {
                    false
                } else if abs_x <= logo.radius - corner_radius
                    || abs_y <= logo.radius - corner_radius
                {
                    true
                } else {
                    let corner_x = abs_x - (logo.radius - corner_radius);
                    let corner_y = abs_y - (logo.radius - corner_radius);
                    corner_x * corner_x + corner_y * corner_y <= corner_radius * corner_radius
                }
            }
            LogoMaskShape::Square => delta_x.abs() <= logo.radius && delta_y.abs() <= logo.radius,
        }
    };

    let is_active = |module_x: usize, module_y: usize| -> bool {
        if matrix.is_in_finder(module_x, module_y) {
            return false;
        }
        matrix.is_dark(module_x, module_y) && !is_inside_logo(module_x, module_y)
    };

    let mut elements = String::with_capacity(size * size * 32);

    if let Some(background) = &config.bg_color {
        let _ = write!(
            elements,
            r#"<rect width="{qr_width}" height="{full_height}" fill="{background}"/>"#
        );
    }

    let mut body_path = String::with_capacity(size * size * 24);
    if config.module_shape == ModuleShape::Square {
        // Merge contiguous runs into one rectangle to keep the SVG compact.
        for y in 0..size {
            let y_position = padding + y as f32 * scale;
            let mut x = 0;
            while x < size {
                if !is_active(x, y) {
                    x += 1;
                    continue;
                }

                let run_start = x;
                while x < size && is_active(x, y) {
                    x += 1;
                }
                let run_length = x - run_start;
                let x_position = padding + run_start as f32 * scale;
                let width = run_length as f32 * scale;
                let _ = write!(
                    body_path,
                    "M {:.2} {:.2} h {:.2} v {:.2} h -{:.2} Z ",
                    x_position, y_position, width, scale, width
                );
            }
        }
    } else {
        for y in 0..size {
            for x in 0..size {
                if !is_active(x, y) {
                    continue;
                }

                let x_position = padding + x as f32 * scale;
                let y_position = padding + y as f32 * scale;

                match config.module_shape {
                    ModuleShape::Dots => {
                        let center_x = x_position + scale / 2.0;
                        let center_y = y_position + scale / 2.0;
                        // A half-module radius lets neighbouring dots touch,
                        // which keeps the grid readable to common decoders.
                        push_circle(&mut body_path, center_x, center_y, scale * 0.5);
                    }
                    ModuleShape::Smooth => {
                        let top = y > 0 && is_active(x, y - 1);
                        let right = x + 1 < size && is_active(x + 1, y);
                        let bottom = y + 1 < size && is_active(x, y + 1);
                        let left = x > 0 && is_active(x - 1, y);
                        let radius = scale * 0.45;
                        let corners = [
                            if top || left { 0.0 } else { radius },
                            if top || right { 0.0 } else { radius },
                            if bottom || right { 0.0 } else { radius },
                            if bottom || left { 0.0 } else { radius },
                        ];
                        push_rounded_module(&mut body_path, x_position, y_position, scale, corners);
                    }
                    ModuleShape::Classy => {
                        let top = y > 0 && is_active(x, y - 1);
                        let right = x + 1 < size && is_active(x + 1, y);
                        let bottom = y + 1 < size && is_active(x, y + 1);
                        let left = x > 0 && is_active(x - 1, y);
                        let radius = scale / 2.0;
                        let corners = if !(left || right || top || bottom) {
                            [radius, 0.0, radius, 0.0]
                        } else if !(left || top) {
                            [radius, 0.0, 0.0, 0.0]
                        } else if !(right || bottom) {
                            [0.0, 0.0, radius, 0.0]
                        } else {
                            [0.0, 0.0, 0.0, 0.0]
                        };
                        push_rounded_module(&mut body_path, x_position, y_position, scale, corners);
                    }
                    ModuleShape::Square => unreachable!("square modules are handled above"),
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

    let mut eye_frame_path = String::with_capacity(512);
    let mut eye_dot_path = String::with_capacity(256);
    let finders = [
        (0, 0),
        (size.saturating_sub(7), 0),
        (0, size.saturating_sub(7)),
    ];
    for &(finder_x, finder_y) in &finders {
        let origin_x = padding + finder_x as f32 * scale;
        let origin_y = padding + finder_y as f32 * scale;
        let frame_size = 7.0 * scale;

        match config.eye_frame_shape {
            EyeFrameShape::Circle => {
                let center_x = origin_x + frame_size / 2.0;
                let center_y = origin_y + frame_size / 2.0;
                let outer_radius = frame_size / 2.0;
                let inner_radius = frame_size / 2.0 - scale;
                let outer_diameter = 2.0 * outer_radius;
                let inner_diameter = 2.0 * inner_radius;
                let _ = write!(
                    eye_frame_path,
                    "M {:.2} {center_y:.2} a {outer_radius:.2} {outer_radius:.2} 0 1 0 {outer_diameter:.2} 0 a {outer_radius:.2} {outer_radius:.2} 0 1 0 -{outer_diameter:.2} 0 Z M {:.2} {center_y:.2} a {inner_radius:.2} {inner_radius:.2} 0 1 0 {inner_diameter:.2} 0 a {inner_radius:.2} {inner_radius:.2} 0 1 0 -{inner_diameter:.2} 0 Z ",
                    center_x - outer_radius,
                    center_x - inner_radius,
                );
            }
            EyeFrameShape::Rounded => {
                // 1.2 modules is the largest corner radius that still decodes
                // with the bundled verifier. The original 1.8 did not.
                let radius = scale * 1.2;
                let inner_radius = scale * 0.2;
                let straight = frame_size - 2.0 * radius;
                let inner_straight = (frame_size - 2.0 * scale) - 2.0 * inner_radius;
                let _ = write!(
                    eye_frame_path,
                    "M {:.2} {origin_y:.2} h {straight:.2} a {radius:.2} {radius:.2} 0 0 1 {radius:.2} {radius:.2} v {straight:.2} a {radius:.2} {radius:.2} 0 0 1 -{radius:.2} {radius:.2} h -{straight:.2} a {radius:.2} {radius:.2} 0 0 1 -{radius:.2} -{radius:.2} v -{straight:.2} a {radius:.2} {radius:.2} 0 0 1 {radius:.2} -{radius:.2} Z ",
                    origin_x + radius
                );
                let inner_y = origin_y + scale;
                let _ = write!(
                    eye_frame_path,
                    "M {:.2} {inner_y:.2} h {inner_straight:.2} a {inner_radius:.2} {inner_radius:.2} 0 0 1 {inner_radius:.2} {inner_radius:.2} v {inner_straight:.2} a {inner_radius:.2} {inner_radius:.2} 0 0 1 -{inner_radius:.2} {inner_radius:.2} h -{inner_straight:.2} a {inner_radius:.2} {inner_radius:.2} 0 0 1 -{inner_radius:.2} -{inner_radius:.2} v -{inner_straight:.2} a {inner_radius:.2} {inner_radius:.2} 0 0 1 {inner_radius:.2} -{inner_radius:.2} Z ",
                    origin_x + scale + inner_radius
                );
            }
            EyeFrameShape::Square => {
                let inner_size = frame_size - 2.0 * scale;
                let _ = write!(
                    eye_frame_path,
                    "M {origin_x:.2} {origin_y:.2} h {frame_size:.2} v {frame_size:.2} h -{frame_size:.2} Z M {:.2} {:.2} h {inner_size:.2} v {inner_size:.2} h -{inner_size:.2} Z ",
                    origin_x + scale,
                    origin_y + scale
                );
            }
        }

        let dot_x = origin_x + 2.0 * scale;
        let dot_y = origin_y + 2.0 * scale;
        let dot_size = 3.0 * scale;
        match config.eye_dot_shape {
            EyeDotShape::Circle => {
                let center_x = dot_x + dot_size / 2.0;
                let center_y = dot_y + dot_size / 2.0;
                push_circle(&mut eye_dot_path, center_x, center_y, dot_size / 2.0);
            }
            EyeDotShape::Rounded => {
                let radius = scale * 0.9;
                let straight = dot_size - 2.0 * radius;
                let _ = write!(
                    eye_dot_path,
                    "M {:.2} {dot_y:.2} h {straight:.2} a {radius:.2} {radius:.2} 0 0 1 {radius:.2} {radius:.2} v {straight:.2} a {radius:.2} {radius:.2} 0 0 1 -{radius:.2} {radius:.2} h -{straight:.2} a {radius:.2} {radius:.2} 0 0 1 -{radius:.2} -{radius:.2} v -{straight:.2} a {radius:.2} {radius:.2} 0 0 1 {radius:.2} -{radius:.2} Z ",
                    dot_x + radius
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

    if let Some(logo) = &logo {
        let background = config.bg_color.as_deref().unwrap_or("#04070d");
        match logo.config.shape {
            LogoMaskShape::Circle => {
                let _ = write!(
                    elements,
                    r#"<circle cx="{}" cy="{}" r="{}" fill="{background}"/>"#,
                    logo.center_x, logo.center_y, logo.radius
                );
            }
            LogoMaskShape::Rounded => {
                let corner_radius = scale * 0.8;
                let _ = write!(
                    elements,
                    r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{corner_radius}" ry="{corner_radius}" fill="{background}"/>"#,
                    logo.x, logo.y, logo.pixel_size, logo.pixel_size
                );
            }
            LogoMaskShape::Square => {
                let _ = write!(
                    elements,
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{background}"/>"#,
                    logo.x, logo.y, logo.pixel_size, logo.pixel_size
                );
            }
        }

        let inner_size = (logo.pixel_size - 2.0 * logo.inset).max(0.0);
        let inner_x = logo.x + logo.inset;
        let inner_y = logo.y + logo.inset;
        let _ = write!(
            elements,
            r#"<image href="{}" x="{inner_x}" y="{inner_y}" width="{inner_size}" height="{inner_size}" preserveAspectRatio="xMidYMid slice" clip-path="url(#logo-clip)"/>"#,
            logo.config.data_url
        );
    }

    if let Some(cta_text) = &cta.text {
        let text_y = qr_width + cta.height * 0.65;
        let cta_color = config
            .cta
            .as_ref()
            .map(|cta| cta.color.as_str())
            .unwrap_or("#ffffff");
        let _ = write!(
            elements,
            r#"<text x="50%" y="{text_y}" text-anchor="middle" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="{}px" font-weight="bold" fill="{cta_color}" letter-spacing="1.5">{cta_text}</text>"#,
            cta.font_size
        );
    }

    let defs = if defs.is_empty() {
        String::new()
    } else {
        format!("<defs>{defs}</defs>")
    };
    let alt_text = config.alt_text.as_deref().unwrap_or("QR Code");

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {qr_width} {full_height}" width="{qr_width}" height="{full_height}" shape-rendering="geometricPrecision"><title>{alt_text}</title>{defs}{elements}</svg>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::EccLevel;
    use crate::matrix::Overlay;

    #[test]
    fn produces_a_complete_svg_document() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        let config = VectorRenderConfig::default();
        let svg = render_svg(&matrix, &config);

        assert!(svg.starts_with("<svg"), "SVG root element");
        assert!(svg.ends_with("</svg>"), "SVG is closed");
        assert!(svg.contains("<path"), "modules are rendered as paths");
        assert!(svg.contains("<title>QR Code</title>"), "title is present");
    }

    #[test]
    fn default_quiet_zone_is_four_modules() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        let config = VectorRenderConfig::default();
        let expected_width = (matrix.size + 8) as f32 * config.scale;
        let svg = render_svg(&matrix, &config);

        assert_eq!(config.margin_modules, 4);
        assert!(
            svg.contains(&format!("width=\"{expected_width}\"")),
            "SVG reserves a four-module quiet zone on every side"
        );
    }

    #[test]
    fn square_modules_merge_horizontal_runs() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        let config = VectorRenderConfig {
            module_shape: ModuleShape::Square,
            scale: 10.0,
            margin_modules: 0,
            ..VectorRenderConfig::default()
        };
        let svg = render_svg(&matrix, &config);

        assert!(
            svg.contains("h 20.00") || svg.contains("h 30.00") || svg.contains("h 40.00"),
            "contiguous modules are merged into longer runs"
        );
    }

    #[test]
    fn every_module_shape_renders() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");

        for shape in [
            ModuleShape::Classy,
            ModuleShape::Smooth,
            ModuleShape::Dots,
            ModuleShape::Square,
        ] {
            let config = VectorRenderConfig {
                module_shape: shape,
                ..VectorRenderConfig::default()
            };
            let svg = render_svg(&matrix, &config);
            assert!(svg.contains("<path"), "{shape:?} renders a path");
        }
    }

    #[test]
    fn centre_logo_is_embedded_and_masked() {
        let matrix = QrMatrix::try_new("https://example.com", EccLevel::H, Overlay::Image)
            .expect("URL should encode");
        let config = VectorRenderConfig {
            logo: Some(LogoConfig {
                data_url: "data:image/png;base64,AAAA".to_string(),
                shape: LogoMaskShape::Rounded,
                scale_percent: 22.0,
            }),
            ..VectorRenderConfig::default()
        };
        let svg = render_svg(&matrix, &config);

        assert!(svg.contains(r#"<clipPath id="logo-clip">"#), "logo mask");
        assert!(
            svg.contains(r#"<image href="data:image/png;base64,AAAA""#),
            "logo data URL is embedded"
        );
    }

    #[test]
    fn gradient_definition_is_emitted() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        let config = VectorRenderConfig {
            gradient: Some(GradientConfig::Linear {
                angle: 45.0,
                color_start: "#000000".to_string(),
                color_end: "#00f0ff".to_string(),
            }),
            ..VectorRenderConfig::default()
        };
        let svg = render_svg(&matrix, &config);

        assert!(svg.contains("<linearGradient id=\"qr-grad\""), "gradient");
        assert!(svg.contains("url(#qr-grad)"), "fill references gradient");
    }

    #[test]
    fn call_to_action_adds_height() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        let config = VectorRenderConfig {
            cta: Some(CtaBannerConfig {
                text: "scan me".to_string(),
                font_size: 18.0,
                color: "#ffffff".to_string(),
            }),
            ..VectorRenderConfig::default()
        };
        let svg = render_svg(&matrix, &config);

        assert!(svg.contains(">SCAN ME</text>"), "banner text is uppercase");
    }
}
