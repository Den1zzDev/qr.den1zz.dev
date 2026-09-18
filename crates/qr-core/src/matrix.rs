use qrcode::{EcLevel, QrCode, Version};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EccLevel {
    Auto,
    L,
    M,
    Q,
    H,
}

impl EccLevel {
    pub fn to_qrcode_ec_level(self, has_image_or_logo: bool) -> EcLevel {
        match self {
            EccLevel::Auto => {
                if has_image_or_logo {
                    EcLevel::H
                } else {
                    EcLevel::M
                }
            }
            EccLevel::L => EcLevel::L,
            EccLevel::M => EcLevel::M,
            EccLevel::Q => EcLevel::Q,
            EccLevel::H => EcLevel::H,
        }
    }
}

pub const FINDER_SIZE: usize = 7;

/// Alignment pattern center coordinates for QR code versions 1 to 40.
pub const ALIGNMENT_TABLE: &[&[usize]] = &[
    &[],                            // v0 (unused)
    &[],                            // v1
    &[6, 18],                       // v2
    &[6, 22],                       // v3
    &[6, 26],                       // v4
    &[6, 30],                       // v5
    &[6, 34],                       // v6
    &[6, 22, 38],                   // v7
    &[6, 24, 42],                   // v8
    &[6, 26, 46],                   // v9
    &[6, 28, 50],                   // v10
    &[6, 30, 54],                   // v11
    &[6, 32, 58],                   // v12
    &[6, 34, 62],                   // v13
    &[6, 26, 46, 66],               // v14
    &[6, 26, 48, 70],               // v15
    &[6, 26, 50, 74],               // v16
    &[6, 30, 54, 78],               // v17
    &[6, 30, 56, 82],               // v18
    &[6, 30, 58, 86],               // v19
    &[6, 34, 62, 90],               // v20
    &[6, 28, 50, 72, 94],           // v21
    &[6, 26, 50, 74, 98],           // v22
    &[6, 30, 54, 78, 102],          // v23
    &[6, 28, 54, 80, 106],          // v24
    &[6, 32, 58, 84, 110],          // v25
    &[6, 30, 58, 86, 114],          // v26
    &[6, 34, 62, 90, 118],          // v27
    &[6, 26, 50, 74, 98, 122],      // v28
    &[6, 30, 54, 78, 102, 126],     // v29
    &[6, 26, 52, 78, 104, 130],     // v30
    &[6, 30, 56, 82, 108, 134],     // v31
    &[6, 34, 60, 86, 112, 138],     // v32
    &[6, 30, 58, 86, 114, 142],     // v33
    &[6, 34, 62, 90, 118, 146],     // v34
    &[6, 30, 54, 78, 102, 126, 150],// v35
    &[6, 24, 50, 76, 102, 128, 154],// v36
    &[6, 28, 54, 80, 106, 132, 158],// v37
    &[6, 32, 58, 84, 110, 136, 162],// v38
    &[6, 26, 54, 82, 110, 138, 166],// v39
    &[6, 30, 54, 78, 102, 126, 150],// v40
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrMatrix {
    pub version: usize,
    pub size: usize,
    pub ec_level: EcLevel,
    pub modules: Vec<bool>,
    alignment_centers: Vec<usize>,
}

impl QrMatrix {
    pub fn new(content: &str, requested_ecc: EccLevel, has_image_or_logo: bool) -> Result<Self, String> {
        let ec_level = requested_ecc.to_qrcode_ec_level(has_image_or_logo);
        let code = QrCode::with_error_correction_level(content.as_bytes(), ec_level)
            .map_err(|e| format!("QR encoding failed: {e}"))?;

        let size = code.width();
        let version_num = match code.version() {
            Version::Normal(v) => v as usize,
            Version::Micro(v) => v as usize,
        };

        let alignment_centers = if version_num < ALIGNMENT_TABLE.len() {
            ALIGNMENT_TABLE[version_num].to_vec()
        } else {
            vec![]
        };

        let mut modules = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let dark = match code[(x, y)] {
                    qrcode::Color::Dark => true,
                    qrcode::Color::Light => false,
                };
                modules.push(dark);
            }
        }

        Ok(Self {
            version: version_num,
            size,
            ec_level,
            modules,
            alignment_centers,
        })
    }

    #[inline]
    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        if x < self.size && y < self.size {
            self.modules[y * self.size + x]
        } else {
            false
        }
    }

    #[inline]
    pub fn is_in_finder(&self, x: usize, y: usize) -> bool {
        (x < FINDER_SIZE && y < FINDER_SIZE)
            || (x >= self.size - FINDER_SIZE && y < FINDER_SIZE)
            || (x < FINDER_SIZE && y >= self.size - FINDER_SIZE)
    }

    #[inline]
    pub fn is_timing_pattern(&self, x: usize, y: usize) -> bool {
        if y == 6 && x >= 7 && x < self.size - 7 {
            return true;
        }
        if x == 6 && y >= 7 && y < self.size - 7 {
            return true;
        }
        false
    }

    pub fn is_alignment_pattern(&self, x: usize, y: usize) -> bool {
        for &cx in &self.alignment_centers {
            for &cy in &self.alignment_centers {
                let in_tl = cx < 9 && cy < 9;
                let in_tr = cx >= self.size - 9 && cy < 9;
                let in_bl = cx < 9 && cy >= self.size - 9;
                if in_tl || in_tr || in_bl {
                    continue;
                }
                if (x as isize - cx as isize).abs() <= 2 && (y as isize - cy as isize).abs() <= 2 {
                    return true;
                }
            }
        }
        false
    }

    /// Checks if a module is a protected structural pattern
    /// (finders, timing tracks, alignment patterns).
    #[inline]
    pub fn is_structural(&self, x: usize, y: usize) -> bool {
        self.is_in_finder(x, y) || self.is_timing_pattern(x, y) || self.is_alignment_pattern(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_matrix_generation() {
        let matrix = QrMatrix::new("https://qr.den1zz.dev", EccLevel::Auto, false).unwrap();
        assert!(matrix.size >= 21);
        assert!(matrix.is_in_finder(0, 0));
        assert!(matrix.is_in_finder(6, 6));
        assert!(!matrix.is_in_finder(10, 10));
    }
}
