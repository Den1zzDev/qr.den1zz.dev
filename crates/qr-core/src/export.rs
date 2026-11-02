//! Rasterisation and PNG export.

use std::borrow::Cow;
use std::io::Cursor;

use crc32fast::Hasher;
use image::imageops::FilterType;
use image::DynamicImage;
use image::ImageFormat;
use image::RgbaImage;

use crate::matrix::QrMatrix;

/// Length of the PNG signature and the mandatory `IHDR` chunk.
const PNG_HEADER_LENGTH: usize = 33;

/// Renders `source` as a PNG at the requested resolution.
///
/// The image is resized with nearest-neighbour filtering when its aspect ratio
/// differs from the target. `alt_text` is stored in a `tEXt` chunk when present.
///
/// # Errors
///
/// Returns an [`image::ImageError`] when the image cannot be encoded as PNG.
pub fn export_png(
    source: &DynamicImage,
    resolution: u32,
    alt_text: Option<&str>,
) -> Result<Vec<u8>, image::ImageError> {
    let aspect = source.height() as f32 / source.width() as f32;
    let export_height = ((resolution as f32) * aspect).round() as u32;

    let resized = if source.width() != resolution || source.height() != export_height {
        Cow::Owned(source.resize_exact(resolution, export_height, FilterType::Nearest))
    } else {
        Cow::Borrowed(source)
    };

    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    resized.write_to(&mut cursor, ImageFormat::Png)?;

    match alt_text.filter(|text| !text.trim().is_empty()) {
        Some(text) => Ok(insert_text_chunk(&png_bytes, "Description", text)),
        None => Ok(png_bytes),
    }
}

/// Rasterises `matrix` into an RGBA image at `resolution` square pixels.
///
/// Dark modules are filled with `dark_rgba` and everything else with
/// `light_rgba`. `margin_modules` controls the quiet zone.
#[must_use]
pub fn rasterize_matrix(
    matrix: &QrMatrix,
    resolution: u32,
    dark_rgba: [u8; 4],
    light_rgba: [u8; 4],
    margin_modules: usize,
) -> RgbaImage {
    let total_modules = matrix.size + margin_modules * 2;
    let mut image = RgbaImage::new(resolution, resolution);
    let raw_pixels = image.as_mut();

    fill_pixels(raw_pixels, light_rgba);
    if matrix.size == 0 || resolution == 0 || total_modules == 0 {
        return image;
    }

    let scale = f64::from(resolution) / total_modules as f64;
    let row_stride = resolution as usize * 4;

    for module_y in 0..matrix.size {
        let pixel_y_start = (((module_y + margin_modules) as f64) * scale).round() as usize;
        let pixel_y_end = ((((module_y + margin_modules + 1) as f64) * scale).round() as usize)
            .min(resolution as usize);
        if pixel_y_start >= pixel_y_end {
            continue;
        }

        let mut module_x = 0;
        while module_x < matrix.size {
            if !matrix.is_dark(module_x, module_y) {
                module_x += 1;
                continue;
            }

            let run_start = module_x;
            while module_x < matrix.size && matrix.is_dark(module_x, module_y) {
                module_x += 1;
            }
            let run_end = module_x;

            let pixel_x_start = (((run_start + margin_modules) as f64) * scale).round() as usize;
            let pixel_x_end = ((((run_end + margin_modules) as f64) * scale).round() as usize)
                .min(resolution as usize);
            if pixel_x_start >= pixel_x_end {
                continue;
            }

            let byte_start = pixel_x_start * 4;
            let byte_end = pixel_x_end * 4;
            for pixel_y in pixel_y_start..pixel_y_end {
                let row = &mut raw_pixels
                    [pixel_y * row_stride + byte_start..pixel_y * row_stride + byte_end];
                fill_pixels(row, dark_rgba);
            }
        }
    }

    image
}

/// Inserts a `tEXt` chunk into an existing PNG.
///
/// The chunk is inserted directly after the `IHDR` chunk so that decoders see
/// the metadata without having to scan the entire image. Invalid input shorter
/// than a PNG header is returned unchanged.
#[must_use]
pub fn insert_text_chunk(png_bytes: &[u8], keyword: &str, text: &str) -> Vec<u8> {
    if png_bytes.len() < PNG_HEADER_LENGTH {
        return png_bytes.to_vec();
    }

    let keyword_bytes = keyword.as_bytes();
    let mut chunk_data = Vec::with_capacity(keyword_bytes.len() + 1 + text.len());
    chunk_data.extend_from_slice(keyword_bytes);
    chunk_data.push(0);
    chunk_data.extend_from_slice(text.as_bytes());

    let chunk_type = b"tEXt";
    let mut hasher = Hasher::new();
    hasher.update(chunk_type);
    hasher.update(&chunk_data);
    let checksum = hasher.finalize();

    let mut chunk = Vec::with_capacity(4 + 4 + chunk_data.len() + 4);
    chunk.extend_from_slice(&(chunk_data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(chunk_type);
    chunk.extend_from_slice(&chunk_data);
    chunk.extend_from_slice(&checksum.to_be_bytes());

    let mut result = Vec::with_capacity(png_bytes.len() + chunk.len());
    result.extend_from_slice(&png_bytes[..PNG_HEADER_LENGTH]);
    result.extend_from_slice(&chunk);
    result.extend_from_slice(&png_bytes[PNG_HEADER_LENGTH..]);
    result
}

/// Fills `pixels` with `colour`.
///
/// Opaque greyscale colours are filled a byte at a time; every other colour is
/// copied one pixel at a time.
fn fill_pixels(pixels: &mut [u8], colour: [u8; 4]) {
    if colour.iter().all(|channel| *channel == colour[0]) {
        pixels.fill(colour[0]);
    } else {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&colour);
        }
    }
}

#[cfg(test)]
mod tests {
    use image::RgbaImage;

    use super::*;
    use crate::matrix::EccLevel;
    use crate::matrix::Overlay;

    #[test]
    fn exports_a_png_with_metadata() {
        let source = DynamicImage::ImageRgba8(RgbaImage::new(64, 64));
        let png = export_png(&source, 128, Some("Sample QR metadata"))
            .expect("PNG encoding should succeed");

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "PNG signature");
        assert!(
            png.windows(4).any(|window| window == b"tEXt"),
            "metadata chunk is present"
        );
    }

    #[test]
    fn rasterizes_every_supported_resolution() {
        let matrix = QrMatrix::try_new("https://den1zz.dev", EccLevel::M, Overlay::None)
            .expect("URL should encode");

        for resolution in [512, 1024, 2048] {
            let image =
                rasterize_matrix(&matrix, resolution, [0, 0, 0, 255], [255, 255, 255, 255], 2);
            assert_eq!(image.width(), resolution);
            assert_eq!(image.height(), resolution);
        }
    }

    #[test]
    fn short_input_is_returned_unchanged() {
        let input = b"not a png";
        assert_eq!(insert_text_chunk(input, "Description", "x"), input);
    }
}
