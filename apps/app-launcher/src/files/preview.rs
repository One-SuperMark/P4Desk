use super::{file_kind, FileError, FileKind, Limits, Preview};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::Arc;

pub(crate) fn read_preview(path: &Path, limits: Limits) -> Result<Preview, FileError> {
    let metadata = std::fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(FileError::Unsupported);
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(FileError::InvalidPath)?;
    match file_kind(name, false) {
        FileKind::Text => text(path, limits.text_bytes),
        FileKind::Image => {
            if metadata.len() > limits.image_file_bytes {
                return Err(FileError::TooLarge);
            }
            let mut bytes = Vec::new();
            File::open(path)?
                .take(limits.image_file_bytes + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > limits.image_file_bytes {
                return Err(FileError::TooLarge);
            }
            let extension = name
                .rsplit_once('.')
                .map(|(_, extension)| extension.to_ascii_lowercase())
                .unwrap_or_default();
            match extension.as_str() {
                "png" => png(&bytes, limits),
                "jpg" | "jpeg" => jpeg(&bytes, limits),
                "bmp" => bmp(&bytes, limits),
                _ => Ok(Preview::Unsupported),
            }
        }
        _ => Ok(Preview::Unsupported),
    }
}

fn text(path: &Path, maximum: usize) -> Result<Preview, FileError> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(maximum.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    let truncated = bytes.len() > maximum;
    bytes.truncate(maximum);
    let valid = match std::str::from_utf8(&bytes) {
        Ok(value) => value,
        Err(error) if truncated && error.error_len().is_none() => {
            std::str::from_utf8(&bytes[..error.valid_up_to()])
                .map_err(|_| FileError::InvalidText)?
        }
        Err(_) => return Err(FileError::InvalidText),
    };
    if valid.contains('\0') {
        return Err(FileError::InvalidText);
    }
    let valid = valid.strip_prefix('\u{feff}').unwrap_or(valid);
    Ok(Preview::Text {
        text: Arc::from(valid),
        truncated,
    })
}

fn dimensions(width: u32, height: u32, limits: Limits) -> Result<usize, FileError> {
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err(FileError::TooLarge);
    }
    let pixels = (width as usize)
        .checked_mul(height as usize)
        .ok_or(FileError::TooLarge)?;
    if pixels > limits.image_pixels {
        return Err(FileError::TooLarge);
    }
    Ok(pixels)
}

fn png(bytes: &[u8], limits: Limits) -> Result<Preview, FileError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits {
        bytes: limits
            .image_pixels
            .saturating_mul(4)
            .saturating_add(512 * 1024),
    });
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|_| FileError::InvalidImage)?;
    let info = reader.info();
    let original_width = info.width;
    let original_height = info.height;
    dimensions(original_width, original_height, limits)?;
    let length = reader.output_buffer_size();
    if length > limits.image_pixels.saturating_mul(4) {
        return Err(FileError::TooLarge);
    }
    let mut decoded = Vec::new();
    decoded
        .try_reserve_exact(length)
        .map_err(|_| FileError::TooLarge)?;
    decoded.resize(length, 0);
    let output = reader
        .next_frame(&mut decoded)
        .map_err(|_| FileError::InvalidImage)?;
    let color = output.color_type;
    let channels = match color {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => return Err(FileError::InvalidImage),
    };
    if output.bit_depth != png::BitDepth::Eight
        || output.buffer_size() != output.width as usize * output.height as usize * channels
    {
        return Err(FileError::InvalidImage);
    }
    to_preview(
        output.width,
        output.height,
        original_width,
        original_height,
        limits,
        |x, y| {
            let offset = (y as usize * output.width as usize + x as usize) * channels;
            let p = &decoded[offset..offset + channels];
            match channels {
                1 => [p[0], p[0], p[0], 255],
                2 => [p[0], p[0], p[0], p[1]],
                3 => [p[0], p[1], p[2], 255],
                _ => [p[0], p[1], p[2], p[3]],
            }
        },
    )
}

fn jpeg(bytes: &[u8], limits: Limits) -> Result<Preview, FileError> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    decoder.set_max_decoding_buffer_size(limits.image_pixels.saturating_mul(4));
    decoder.read_info().map_err(|_| FileError::InvalidImage)?;
    let original = decoder.info().ok_or(FileError::InvalidImage)?;
    let original_pixels = usize::from(original.width)
        .checked_mul(usize::from(original.height))
        .ok_or(FileError::TooLarge)?;
    if original.width == 0
        || original.height == 0
        || original.width > 8192
        || original.height > 8192
        || original_pixels > 16 * 1024 * 1024
    {
        return Err(FileError::TooLarge);
    }
    // Progressive coefficients retain their original resolution even with IDCT scaling.
    // The decoder's output-buffer limit does not protect that allocation.
    if original.coding_process != jpeg_decoder::CodingProcess::DctSequential
        && original_pixels > limits.image_pixels
    {
        return Err(FileError::TooLarge);
    }
    decoder
        .scale(
            limits.preview_width.min(u16::MAX as u32) as u16,
            limits.preview_height.min(u16::MAX as u32) as u16,
        )
        .map_err(|_| FileError::InvalidImage)?;
    let scaled = decoder.info().ok_or(FileError::InvalidImage)?;
    dimensions(u32::from(scaled.width), u32::from(scaled.height), limits)?;
    let decoded = decoder.decode().map_err(|_| FileError::InvalidImage)?;
    let info = decoder.info().ok_or(FileError::InvalidImage)?;
    let channels = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => 1,
        jpeg_decoder::PixelFormat::RGB24 => 3,
        jpeg_decoder::PixelFormat::CMYK32 => 4,
        jpeg_decoder::PixelFormat::L16 => return Err(FileError::Unsupported),
    };
    if decoded.len() != info.width as usize * info.height as usize * channels {
        return Err(FileError::InvalidImage);
    }
    to_preview(
        u32::from(info.width),
        u32::from(info.height),
        u32::from(original.width),
        u32::from(original.height),
        limits,
        |x, y| {
            let offset = (y as usize * info.width as usize + x as usize) * channels;
            let p = &decoded[offset..offset + channels];
            match channels {
                1 => [p[0], p[0], p[0], 255],
                3 => [p[0], p[1], p[2], 255],
                // jpeg-decoder emits non-inverted CMYK. Convert to RGB before packing.
                _ => [
                    ((255 - u16::from(p[0])) * (255 - u16::from(p[3])) / 255) as u8,
                    ((255 - u16::from(p[1])) * (255 - u16::from(p[3])) / 255) as u8,
                    ((255 - u16::from(p[2])) * (255 - u16::from(p[3])) / 255) as u8,
                    255,
                ],
            }
        },
    )
}

fn bmp(bytes: &[u8], limits: Limits) -> Result<Preview, FileError> {
    if bytes.len() < 54 || &bytes[..2] != b"BM" {
        return Err(FileError::InvalidImage);
    }
    let read_u16 = |at| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
    let read_u32 = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    let header_size = read_u32(14);
    let width = read_u32(18) as i32;
    let signed_height = read_u32(22) as i32;
    let bpp = read_u16(28);
    let offset = read_u32(10) as usize;
    if header_size < 40
        || width <= 0
        || signed_height == i32::MIN
        || signed_height == 0
        || read_u16(26) != 1
        || read_u32(30) != 0
        || !matches!(bpp, 24 | 32)
        || offset < 14usize.saturating_add(header_size as usize)
    {
        return Err(FileError::InvalidImage);
    }
    let width = width as u32;
    let height = signed_height.unsigned_abs();
    dimensions(width, height, limits)?;
    let channels = usize::from(bpp / 8);
    let stride = (width as usize * channels + 3) & !3;
    let end = stride
        .checked_mul(height as usize)
        .and_then(|length| offset.checked_add(length))
        .ok_or(FileError::TooLarge)?;
    if end > bytes.len() {
        return Err(FileError::InvalidImage);
    }
    to_preview(width, height, width, height, limits, |x, y| {
        let row = if signed_height < 0 { y } else { height - 1 - y };
        let at = offset + row as usize * stride + x as usize * channels;
        // BI_RGB 32-bit BMP treats its fourth byte as padding, not alpha.
        [bytes[at + 2], bytes[at + 1], bytes[at], 255]
    })
}

fn to_preview<F: Fn(u32, u32) -> [u8; 4]>(
    width: u32,
    height: u32,
    original_width: u32,
    original_height: u32,
    limits: Limits,
    sample: F,
) -> Result<Preview, FileError> {
    if limits.preview_width == 0 || limits.preview_height == 0 {
        return Err(FileError::TooLarge);
    }
    let scale = (limits.preview_width as f32 / width as f32)
        .min(limits.preview_height as f32 / height as f32)
        .min(1.0);
    let output_width = (width as f32 * scale).round().max(1.0) as u32;
    let output_height = (height as f32 * scale).round().max(1.0) as u32;
    let length = dimensions(output_width, output_height, limits)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(length)
        .map_err(|_| FileError::TooLarge)?;
    // P4 has a single-precision FPU. Calculate small coordinate tables once and use
    // integer interpolation in the pixel loop rather than software f64 arithmetic.
    let coordinates = |index: u32, original: u32, output: u32| {
        let source = if original == output {
            index as f32
        } else {
            ((index as f32 + 0.5) * original as f32 / output as f32 - 0.5).max(0.0)
        };
        let first = source.floor() as u32;
        (
            first,
            (first + 1).min(original - 1),
            ((source - first as f32) * 256.0).round() as u32,
        )
    };
    let x_coordinates: Vec<_> = (0..output_width)
        .map(|x| coordinates(x, width, output_width))
        .collect();
    for y in 0..output_height {
        let (y0, y1, fy) = coordinates(y, height, output_height);
        for x in 0..output_width {
            let (x0, x1, fx) = x_coordinates[x as usize];
            let a = sample(x0, y0);
            let b = sample(x1, y0);
            let c = sample(x0, y1);
            let d = sample(x1, y1);
            let mut rgba = [0u16; 4];
            for channel in 0..4 {
                let top = u32::from(a[channel]) * (256 - fx) + u32::from(b[channel]) * fx;
                let bottom = u32::from(c[channel]) * (256 - fx) + u32::from(d[channel]) * fx;
                rgba[channel] = ((top * (256 - fy) + bottom * fy + 32768) >> 16) as u16;
            }
            let checker = if (x / 12 + y / 12) % 2 == 0 {
                220u16
            } else {
                242u16
            };
            let alpha = rgba[3];
            let blend = |channel: usize| {
                ((rgba[channel] * alpha + checker * (255 - alpha) + 127) / 255) as u16
            };
            let red = blend(0);
            let green = blend(1);
            let blue = blend(2);
            pixels.push(((red >> 3) << 11) | ((green >> 2) << 5) | (blue >> 3));
        }
        if y % 16 == 0 {
            std::thread::yield_now();
        }
    }
    Ok(Preview::Image {
        width: output_width,
        height: output_height,
        original_width,
        original_height,
        pixels: Arc::from(pixels),
    })
}
