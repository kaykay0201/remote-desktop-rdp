use crate::protocol::FrameData;

pub fn encode_frame(bgra_pixels: &[u8], width: u32, height: u32, quality: u8) -> Result<FrameData, String> {
    if width == 0 || height == 0 {
        return Err("width and height must be non-zero".to_string());
    }

    let expected_len = (width as usize) * (height as usize) * 4;
    if bgra_pixels.len() != expected_len {
        return Err(format!(
            "pixel buffer size mismatch: expected {} but got {}",
            expected_len,
            bgra_pixels.len()
        ));
    }

    let pixel_count = (width as usize) * (height as usize);
    let mut rgb_data = Vec::with_capacity(pixel_count * 3);
    for i in 0..pixel_count {
        let offset = i * 4;
        rgb_data.push(bgra_pixels[offset + 2]); // R (from BGRA position)
        rgb_data.push(bgra_pixels[offset + 1]); // G
        rgb_data.push(bgra_pixels[offset]);     // B (from BGRA position)
    }

    let mut jpeg_buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut jpeg_buf);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
    encoder
        .encode(&rgb_data, width, height, image::ExtendedColorType::Rgb8)
        .map_err(|e| format!("JPEG encode failed: {}", e))?;

    let compressed = lz4_flex::compress_prepend_size(&jpeg_buf);

    Ok(FrameData {
        width,
        height,
        jpeg_quality: quality,
        compressed_payload: compressed,
    })
}

pub fn decode_frame(frame_data: &FrameData) -> Result<Vec<u8>, String> {
    let jpeg_data = lz4_flex::decompress_size_prepended(&frame_data.compressed_payload)
        .map_err(|e| format!("LZ4 decompress failed: {}", e))?;

    let img = image::load_from_memory_with_format(&jpeg_data, image::ImageFormat::Jpeg)
        .map_err(|e| format!("JPEG decode failed: {}", e))?;

    Ok(img.to_rgba8().into_raw())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CaptureConfig;

    fn make_bgra_buffer(width: u32, height: u32) -> Vec<u8> {
        let mut buf = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                buf.push((x % 256) as u8);       // B
                buf.push((y % 256) as u8);       // G
                buf.push(((x + y) % 256) as u8); // R
                buf.push(255);                    // A
            }
        }
        buf
    }

    #[test]
    fn encode_synthetic_frame() {
        let buf = make_bgra_buffer(100, 100);
        let result = encode_frame(&buf, 100, 100, 75);
        assert!(result.is_ok());
        let frame = result.unwrap();
        assert_eq!(frame.width, 100);
        assert_eq!(frame.height, 100);
        assert_eq!(frame.jpeg_quality, 75);
        assert!(!frame.compressed_payload.is_empty());
    }

    #[test]
    fn encode_decode_roundtrip() {
        let width = 64;
        let height = 64;
        let buf = make_bgra_buffer(width, height);
        let frame = encode_frame(&buf, width, height, 90).unwrap();
        let rgba = decode_frame(&frame).unwrap();
        assert_eq!(rgba.len(), (width * height * 4) as usize);
    }

    #[test]
    fn quality_affects_size() {
        let buf = make_bgra_buffer(100, 100);
        let low = encode_frame(&buf, 100, 100, 10).unwrap();
        let high = encode_frame(&buf, 100, 100, 90).unwrap();
        assert!(high.compressed_payload.len() > low.compressed_payload.len());
    }

    #[test]
    fn config_defaults() {
        let config = CaptureConfig::default();
        assert_eq!(config.fps, 30);
        assert_eq!(config.jpeg_quality, 75);
    }

    #[test]
    fn encode_empty_fails() {
        let result = encode_frame(&[], 0, 0, 75);
        assert!(result.is_err());
    }
}
