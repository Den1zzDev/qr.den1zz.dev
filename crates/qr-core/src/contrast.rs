//! WCAG 2.1 relative luminance and contrast ratio calculations.

/// Minimum contrast ratio a rendered code needs against its background.
///
/// WCAG 2.1 treats 3:1 as the floor for non-text graphics. The scan verifier
/// and the studio both use this value so a code that passes the preview check
/// is also accepted by verification.
pub const MIN_GRAPHIC_CONTRAST: f32 = 3.0;

/// Luminance value at or below which the linearisation curve is applied.
const LINEAR_THRESHOLD: f32 = 0.039_28;

/// Parses the two hexadecimal digits starting at `start`.
fn parse_channel(hex: &str, start: usize) -> Option<u8> {
    let channel = hex.get(start..start + 2)?;
    u8::from_str_radix(channel, 16).ok()
}

/// Converts an sRGB channel in the range `0.0..=1.0` to linear light.
fn linearize(channel: f32) -> f32 {
    if channel <= LINEAR_THRESHOLD {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// Returns the WCAG 2.1 relative luminance of a `#RRGGBB` colour.
///
/// Returns `0.0` when `hex` is not a six-digit hexadecimal colour. A leading
/// `#` is optional.
#[must_use]
pub fn relative_luminance(hex: &str) -> f32 {
    let colour = hex.trim().trim_start_matches('#');
    if colour.len() != 6 {
        return 0.0;
    }

    let (Some(red), Some(green), Some(blue)) = (
        parse_channel(colour, 0),
        parse_channel(colour, 2),
        parse_channel(colour, 4),
    ) else {
        return 0.0;
    };

    let red = linearize(f32::from(red) / 255.0);
    let green = linearize(f32::from(green) / 255.0);
    let blue = linearize(f32::from(blue) / 255.0);

    0.2126 * red + 0.7152 * green + 0.0722 * blue
}

/// Returns the WCAG 2.1 contrast ratio between two `#RRGGBB` colours.
///
/// The result is always at least `1.0`. Colours that cannot be parsed are
/// treated as black by [`relative_luminance`].
#[must_use]
pub fn contrast_ratio(first: &str, second: &str) -> f32 {
    let first = relative_luminance(first);
    let second = relative_luminance(second);
    let brightest = first.max(second);
    let darkest = first.min(second);
    (brightest + 0.05) / (darkest + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white_is_maximum_contrast() {
        let ratio = contrast_ratio("#000000", "#FFFFFF");
        assert!(
            (ratio - 21.0).abs() < 0.5,
            "expected about 21:1, got {ratio}:1"
        );
    }

    #[test]
    fn identical_colours_have_unit_contrast() {
        let ratio = contrast_ratio("#008b99", "#008b99");
        assert!((ratio - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn invalid_colours_are_treated_as_black() {
        assert_eq!(relative_luminance("not-a-colour"), 0.0);
        assert_eq!(relative_luminance("ééé"), 0.0);
        assert_eq!(relative_luminance("#fff"), 0.0);
    }

    #[test]
    fn leading_hash_is_optional() {
        assert_eq!(relative_luminance("000000"), relative_luminance("#000000"));
    }
}
