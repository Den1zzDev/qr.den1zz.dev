use crc32fast::Hasher;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use std::io::Cursor;

pub struct PngExporter;

impl PngExporter {
    pub fn export_png(
        image: &DynamicImage,
        target_resolution: u32,
        alt_text: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        let (w, h) = (image.width(), image.height());
        let aspect = h as f32 / w as f32;
        let exp_width = target_resolution;
        let exp_height = ((target_resolution as f32) * aspect).round() as u32;

        let resized = if w != exp_width || h != exp_height {
            image.resize_exact(exp_width, exp_height, FilterType::Nearest)
        } else {
            image.clone()
        };

        let mut png_bytes = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);
        resized
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| format!("PNG encoding failed: {e}"))?;

        if let Some(text) = alt_text {
            if !text.trim().is_empty() {
                return Ok(Self::insert_text_chunk(&png_bytes, "Description", text));
            }
        }

        Ok(png_bytes)
    }

    pub fn insert_text_chunk(png_bytes: &[u8], keyword: &str, text: &str) -> Vec<u8> {
        if png_bytes.len() < 33 {
            return png_bytes.to_vec();
        }

        let key_bytes = keyword.as_bytes();
        let val_bytes = text.as_bytes();
        let chunk_data_len = key_bytes.len() + 1 + val_bytes.len();

        let mut chunk_data = Vec::with_capacity(chunk_data_len);
        chunk_data.extend_from_slice(key_bytes);
        chunk_data.push(0); // null separator
        chunk_data.extend_from_slice(val_bytes);

        let type_bytes = b"tEXt";
        let mut crc_hasher = Hasher::new();
        crc_hasher.update(type_bytes);
        crc_hasher.update(&chunk_data);
        let crc = crc_hasher.finalize();

        let total_chunk_len = 4 + 4 + chunk_data.len() + 4;
        let mut chunk = Vec::with_capacity(total_chunk_len);
        chunk.extend_from_slice(&(chunk_data.len() as u32).to_be_bytes());
        chunk.extend_from_slice(type_bytes);
        chunk.extend_from_slice(&chunk_data);
        chunk.extend_from_slice(&crc.to_be_bytes());

        // Standard PNG IHDR chunk ends at index 33 (8 byte signature + 25 byte IHDR)
        let insert_pos = 33;
        let mut result = Vec::with_capacity(png_bytes.len() + chunk.len());
        result.extend_from_slice(&png_bytes[..insert_pos]);
        result.extend_from_slice(&chunk);
        result.extend_from_slice(&png_bytes[insert_pos..]);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    #[test]
    fn test_png_export_with_text_chunk() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(64, 64));
        let png = PngExporter::export_png(&img, 128, Some("Sample QR metadata")).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        // Check for 'tEXt'
        assert!(png.windows(4).any(|w| w == b"tEXt"));
    }
}
