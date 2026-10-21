use std::io::Cursor;

use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
use qrcode::types::Color;
use qrcode::{EcLevel, QrCode};

use crate::build_vcard;

pub struct QrImage {
    pub version: u32,
    pub modules: u32,
    pub svg: String,
    pub png: Vec<u8>,
}

pub fn encode_svg(text: &str, level: &str, fg: &str, bg: &str) -> Result<(u32, u32, String), String> {
    let (code, version, modules) = make_code(text, level)?;
    Ok((version, modules, svg_from(&code, fg, bg)?))
}

pub fn encode_png(
    text: &str,
    level: &str,
    fg: &str,
    bg: &str,
    size: u32,
) -> Result<QrImage, String> {
    let (code, version, modules) = make_code(text, level)?;
    let fg = parse_hex(fg)?;
    let bg = parse_hex(bg)?;
    let svg = svg_from(&code, &hex(fg), &hex(bg))?;
    let png = png_from(&code, fg, bg, size.max(21))?;
    Ok(QrImage {
        version,
        modules,
        svg,
        png,
    })
}

pub fn encode_contact_png(
    first: &str,
    last: &str,
    phone: &str,
    email: &str,
    org: &str,
    title: &str,
    url: &str,
    level: &str,
    fg: &str,
    bg: &str,
    size: u32,
) -> Result<QrImage, String> {
    let card = build_vcard(&crate::Contact {
        first: first.into(),
        last: last.into(),
        phone: phone.into(),
        email: email.into(),
        org: org.into(),
        title: title.into(),
        url: url.into(),
    });
    if card.is_empty() {
        return Err("Add a name, phone, or email.".into());
    }
    encode_png(&card, level, fg, bg, size)
}

fn make_code(text: &str, level: &str) -> Result<(QrCode, u32, u32), String> {
    if text.is_empty() {
        return Err("Add something to encode.".into());
    }
    let code = QrCode::with_error_correction_level(text.as_bytes(), ec(level))
        .map_err(|_| "That text is too long for a QR code at this error-correction level. Shorten it, or switch to Low.".to_string())?;
    let version = match code.version() {
        qrcode::Version::Normal(v) | qrcode::Version::Micro(v) => v as u32,
    };
    let modules = code.width() as u32;
    Ok((code, version, modules))
}

fn ec(level: &str) -> EcLevel {
    match level {
        "L" | "l" => EcLevel::L,
        "Q" | "q" => EcLevel::Q,
        "H" | "h" => EcLevel::H,
        _ => EcLevel::M,
    }
}

fn svg_from(code: &QrCode, fg: &str, bg: &str) -> Result<String, String> {
    let fg = hex(parse_hex(fg)?);
    let bg = hex(parse_hex(bg)?);
    let n = code.width();
    let margin = 4usize;
    let dim = n + margin * 2;
    let mut path = String::new();
    for y in 0..n {
        for x in 0..n {
            if code[(x, y)] == Color::Dark {
                let px = x + margin;
                let py = y + margin;
                path.push_str(&format!("M{px} {py}h1v1h-1z"));
            }
        }
    }
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {dim} {dim}\" width=\"100%\" height=\"100%\" shape-rendering=\"crispEdges\" aria-hidden=\"true\"><rect width=\"{dim}\" height=\"{dim}\" fill=\"{bg}\"/><path fill=\"{fg}\" d=\"{path}\"/></svg>"
    ))
}

fn png_from(code: &QrCode, fg: [u8; 3], bg: [u8; 3], size: u32) -> Result<Vec<u8>, String> {
    let n = code.width();
    let margin = 4usize;
    let dim = n + margin * 2;
    let size = size.max(dim as u32);
    let mut img: RgbaImage = ImageBuffer::from_pixel(size, size, Rgba([bg[0], bg[1], bg[2], 255]));
    for y in 0..size {
        for x in 0..size {
            let mx = (x as usize) * dim / size as usize;
            let my = (y as usize) * dim / size as usize;
            if mx < margin || my < margin || mx >= n + margin || my >= n + margin {
                continue;
            }
            if code[(mx - margin, my - margin)] == Color::Dark {
                img.put_pixel(x, y, Rgba([fg[0], fg[1], fg[2], 255]));
            }
        }
    }
    let mut cursor = Cursor::new(Vec::new());
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

fn parse_hex(value: &str) -> Result<[u8; 3], String> {
    let raw = value.trim().trim_start_matches('#');
    if raw.len() != 6 || !raw.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("Use a 6-digit hex color.".into());
    }
    let num = u32::from_str_radix(raw, 16).map_err(|_| "Use a 6-digit hex color.".to_string())?;
    Ok([
        ((num >> 16) & 255) as u8,
        ((num >> 8) & 255) as u8,
        (num & 255) as u8,
    ])
}

fn hex(rgb: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_and_svg_for_a_url() {
        let image = encode_png("https://example.com", "M", "#141814", "#f4f1e6", 320).unwrap();
        assert!(image.version >= 1);
        assert!(image.modules >= 21);
        assert!(image.svg.contains("<svg"));
        assert!(image.png.starts_with(b"\x89PNG"));
        assert_eq!(image.png.len() > 100, true);
    }
}
