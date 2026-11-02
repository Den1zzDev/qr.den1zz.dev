//! QR matrix construction and structural pattern queries.

use std::error::Error;
use std::fmt;

use qrcode::types::QrError;
use qrcode::Color;
use qrcode::EcLevel;
use qrcode::QrCode;
use qrcode::Version;
use serde::Deserialize;
use serde::Serialize;

/// Number of modules along one side of a finder pattern.
pub const FINDER_SIZE: usize = 7;

/// Error correction level used when encoding a QR code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "UPPERCASE")]
pub enum EccLevel {
    /// Chooses a level based on whether the rendered code has an overlay.
    Auto,
    /// Recovers approximately 7% of codewords.
    L,
    /// Recovers approximately 15% of codewords.
    M,
    /// Recovers approximately 25% of codewords.
    Q,
    /// Recovers approximately 30% of codewords. Required for centre logos.
    H,
}

impl EccLevel {
    /// Returns the label used by the QR specification (`L`, `M`, `Q`, `H`).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::L => "L",
            Self::M => "M",
            Self::Q => "Q",
            Self::H => "H",
        }
    }

    /// Resolves `self` against `overlay`, returning both the concrete level and
    /// the corresponding `qrcode` level.
    fn resolve(self, overlay: Overlay) -> (Self, EcLevel) {
        match self {
            Self::Auto => {
                if overlay == Overlay::Image {
                    (Self::H, EcLevel::H)
                } else {
                    (Self::M, EcLevel::M)
                }
            }
            Self::L => (Self::L, EcLevel::L),
            Self::M => (Self::M, EcLevel::M),
            Self::Q => (Self::Q, EcLevel::Q),
            Self::H => (Self::H, EcLevel::H),
        }
    }
}

/// Whether content is drawn over the centre of the rendered QR code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Overlay {
    /// No content covers the centre of the code.
    None,
    /// An image or logo covers the centre of the code.
    Image,
}

/// Failure returned when content cannot be encoded as a QR code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeError {
    source: QrError,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "QR encoding failed: {}", self.source)
    }
}

impl Error for EncodeError {}

/// Alignment pattern centre coordinates for QR versions 1 to 40.
///
/// The first entry is unused so that the table can be indexed by version.
pub const ALIGNMENT_TABLE: [&[usize]; 41] = [
    &[],                             // v0 (unused)
    &[],                             // v1
    &[6, 18],                        // v2
    &[6, 22],                        // v3
    &[6, 26],                        // v4
    &[6, 30],                        // v5
    &[6, 34],                        // v6
    &[6, 22, 38],                    // v7
    &[6, 24, 42],                    // v8
    &[6, 26, 46],                    // v9
    &[6, 28, 50],                    // v10
    &[6, 30, 54],                    // v11
    &[6, 32, 58],                    // v12
    &[6, 34, 62],                    // v13
    &[6, 26, 46, 66],                // v14
    &[6, 26, 48, 70],                // v15
    &[6, 26, 50, 74],                // v16
    &[6, 30, 54, 78],                // v17
    &[6, 30, 56, 82],                // v18
    &[6, 30, 58, 86],                // v19
    &[6, 34, 62, 90],                // v20
    &[6, 28, 50, 72, 94],            // v21
    &[6, 26, 50, 74, 98],            // v22
    &[6, 30, 54, 78, 102],           // v23
    &[6, 28, 54, 80, 106],           // v24
    &[6, 32, 58, 84, 110],           // v25
    &[6, 30, 58, 86, 114],           // v26
    &[6, 34, 62, 90, 118],           // v27
    &[6, 26, 50, 74, 98, 122],       // v28
    &[6, 30, 54, 78, 102, 126],      // v29
    &[6, 26, 52, 78, 104, 130],      // v30
    &[6, 30, 56, 82, 108, 134],      // v31
    &[6, 34, 60, 86, 112, 138],      // v32
    &[6, 30, 58, 86, 114, 142],      // v33
    &[6, 34, 62, 90, 118, 146],      // v34
    &[6, 30, 54, 78, 102, 126, 150], // v35
    &[6, 24, 50, 76, 102, 128, 154], // v36
    &[6, 28, 54, 80, 106, 132, 158], // v37
    &[6, 32, 58, 84, 110, 136, 162], // v38
    &[6, 26, 54, 82, 110, 138, 166], // v39
    &[6, 30, 54, 78, 102, 126, 150], // v40
];

/// A decoded QR matrix in row-major order.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct QrMatrix {
    /// QR version, from 1 to 40.
    pub version: usize,
    /// Number of modules along one side of the matrix.
    pub size: usize,
    /// Error correction level that was used. Never [`EccLevel::Auto`].
    pub ec_level: EccLevel,
    /// Module values in row-major order; `true` is dark.
    pub modules: Vec<bool>,
    alignment_centers: Vec<usize>,
}

impl QrMatrix {
    /// Builds a QR matrix for `content`.
    ///
    /// `overlay` determines whether [`EccLevel::Auto`] selects a level that
    /// tolerates a centre image.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] when `content` cannot be encoded at the resolved
    /// error correction level.
    pub fn try_new(content: &str, ecc: EccLevel, overlay: Overlay) -> Result<Self, EncodeError> {
        let (ec_level, qrcode_level) = ecc.resolve(overlay);
        let code = QrCode::with_error_correction_level(content.as_bytes(), qrcode_level)
            .map_err(|source| EncodeError { source })?;

        let size = code.width();
        let version = match code.version() {
            Version::Normal(version) | Version::Micro(version) => version as usize,
        };
        let alignment_centers = ALIGNMENT_TABLE
            .get(version)
            .map_or_else(Vec::new, |centers| centers.to_vec());

        let mut modules = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                modules.push(matches!(code[(x, y)], Color::Dark));
            }
        }

        Ok(Self {
            version,
            size,
            ec_level,
            modules,
            alignment_centers,
        })
    }

    /// Returns whether the module at `(x, y)` is dark.
    ///
    /// Coordinates outside the matrix are treated as light.
    #[inline]
    #[must_use]
    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        if x < self.size && y < self.size {
            self.modules[y * self.size + x]
        } else {
            false
        }
    }

    /// Returns whether `(x, y)` lies inside one of the three finder patterns.
    #[inline]
    #[must_use]
    pub fn is_in_finder(&self, x: usize, y: usize) -> bool {
        let far_edge = self.size.saturating_sub(FINDER_SIZE);
        (x < FINDER_SIZE && y < FINDER_SIZE)
            || (x >= far_edge && y < FINDER_SIZE)
            || (x < FINDER_SIZE && y >= far_edge)
    }

    /// Returns whether `(x, y)` lies on a timing pattern.
    #[inline]
    #[must_use]
    pub fn is_timing_pattern(&self, x: usize, y: usize) -> bool {
        let far_edge = self.size.saturating_sub(FINDER_SIZE);
        (y == 6 && x >= FINDER_SIZE && x < far_edge) || (x == 6 && y >= FINDER_SIZE && y < far_edge)
    }

    /// Returns whether `(x, y)` lies inside an alignment pattern.
    #[must_use]
    pub fn is_alignment_pattern(&self, x: usize, y: usize) -> bool {
        let far_edge = self.size.saturating_sub(9);
        for &center_x in &self.alignment_centers {
            for &center_y in &self.alignment_centers {
                let overlaps_finder = (center_x < 9 && center_y < 9)
                    || (center_x >= far_edge && center_y < 9)
                    || (center_x < 9 && center_y >= far_edge);
                if overlaps_finder {
                    continue;
                }
                if x.abs_diff(center_x) <= 2 && y.abs_diff(center_y) <= 2 {
                    return true;
                }
            }
        }
        false
    }

    /// Returns whether `(x, y)` is a protected structural module.
    ///
    /// Structural modules belong to finder patterns, timing tracks, or
    /// alignment patterns and must keep their original shape.
    #[inline]
    #[must_use]
    pub fn is_structural(&self, x: usize, y: usize) -> bool {
        self.is_in_finder(x, y) || self.is_timing_pattern(x, y) || self.is_alignment_pattern(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_a_usable_matrix() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::Auto, Overlay::None)
            .expect("URL should encode");
        assert!(matrix.size >= 21, "QR version 1 is 21 modules wide");
        assert!(matrix.is_in_finder(0, 0), "top-left finder");
        assert!(
            matrix.is_in_finder(6, 6),
            "bottom-right corner of the finder"
        );
        assert!(!matrix.is_in_finder(10, 10), "centre is not a finder");
    }

    #[test]
    fn accepts_empty_content() {
        assert!(
            QrMatrix::try_new("", EccLevel::Auto, Overlay::None).is_ok(),
            "empty payloads are valid QR content"
        );
    }

    #[test]
    fn ecc_levels_have_specification_labels() {
        assert_eq!(EccLevel::L.label(), "L");
        assert_eq!(EccLevel::M.label(), "M");
        assert_eq!(EccLevel::Q.label(), "Q");
        assert_eq!(EccLevel::H.label(), "H");
        assert_eq!(EccLevel::Auto.label(), "Auto");
    }

    #[test]
    fn overlay_boosts_automatic_error_correction() {
        let plain = QrMatrix::try_new("data", EccLevel::Auto, Overlay::None)
            .expect("plain payload should encode");
        let with_logo = QrMatrix::try_new("data", EccLevel::Auto, Overlay::Image)
            .expect("payload with logo should encode");

        assert_eq!(plain.ec_level, EccLevel::M);
        assert_eq!(with_logo.ec_level, EccLevel::H);
    }

    #[test]
    fn structural_modules_are_identified() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");
        assert!(matrix.is_timing_pattern(10, 6), "horizontal timing track");
        assert!(matrix.is_structural(0, 0), "finder patterns are structural");
    }
}
