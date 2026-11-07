//! Parser de rangos de páginas, cálculo geométrico de ajuste y preparación DIB (BGRA).

use crate::types::{PrintColorMode, PrintError, PrintOrientation, PrintScaleMode, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use image::{DynamicImage, GenericImageView};

/// Convierte una cadena de rango 1-based ("", "all", "1-3, 5") en índices 0-based ordenados.
pub fn parse_page_range(spec: &str, total_pages: usize) -> Result<Vec<usize>> {
    if total_pages == 0 {
        return Err(PrintError::InvalidRange("El documento no tiene páginas".into()));
    }
    let trimmed = spec.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") || trimmed == "*" {
        return Ok((0..total_pages).collect());
    }

    let mut indices = Vec::new();
    for part in trimmed.split(',') {
        let token = part.trim();
        if token.is_empty() {
            continue;
        }
        if let Some((a, b)) = token.split_once('-') {
            let start: usize = a
                .trim()
                .parse()
                .map_err(|_| PrintError::InvalidRange(format!("Inicio inválido en '{token}'")))?;
            let end: usize = b
                .trim()
                .parse()
                .map_err(|_| PrintError::InvalidRange(format!("Fin inválido en '{token}'")))?;
            if start == 0 || end == 0 || start > end || end > total_pages {
                return Err(PrintError::InvalidRange(format!(
                    "Rango '{token}' fuera de límites (1..={total_pages})"
                )));
            }
            for p in start..=end {
                let idx = p - 1;
                if !indices.contains(&idx) {
                    indices.push(idx);
                }
            }
        } else {
            let page: usize = token
                .parse()
                .map_err(|_| PrintError::InvalidRange(format!("Número inválido '{token}'")))?;
            if page == 0 || page > total_pages {
                return Err(PrintError::InvalidRange(format!(
                    "Página {page} fuera de límites (1..={total_pages})"
                )));
            }
            let idx = page - 1;
            if !indices.contains(&idx) {
                indices.push(idx);
            }
        }
    }

    if indices.is_empty() {
        return Err(PrintError::InvalidRange("No se seleccionó ninguna página".into()));
    }
    Ok(indices)
}

/// Calcula el rectángulo destino `(x, y, dst_w, dst_h)` en píxeles físicos de la impresora.
pub fn compute_target_rect(
    img_w: u32,
    img_h: u32,
    page_w: i32,
    page_h: i32,
    mode: PrintScaleMode,
) -> (i32, i32, i32, i32) {
    if img_w == 0 || img_h == 0 || page_w <= 0 || page_h <= 0 {
        return (0, 0, page_w.max(1), page_h.max(1));
    }
    let iw = img_w as f64;
    let ih = img_h as f64;
    let pw = page_w as f64;
    let ph = page_h as f64;

    let scale = match mode {
        PrintScaleMode::Fit => (pw / iw).min(ph / ih),
        PrintScaleMode::Fill => (pw / iw).max(ph / ih),
        PrintScaleMode::Actual => (pw / iw).min(ph / ih) * 0.92,
    };

    let dst_w = (iw * scale).round() as i32;
    let dst_h = (ih * scale).round() as i32;
    let x = ((page_w - dst_w) / 2).max(0);
    let y = ((page_h - dst_h) / 2).max(0);
    (x, y, dst_w.max(1), dst_h.max(1))
}

/// Decodifica una imagen desde data-URI, base64 puro o ruta en disco.
pub fn decode_page_source(source: &str) -> Result<Vec<u8>> {
    let trimmed = source.trim();
    if let Some((_, b64_part)) = trimmed.split_once(";base64,") {
        return B64
            .decode(b64_part.trim())
            .map_err(|e| PrintError::ImageDecode(e.to_string()));
    }
    if std::path::Path::new(trimmed).exists() {
        return std::fs::read(trimmed).map_err(|e| PrintError::ImageDecode(e.to_string()));
    }
    B64.decode(trimmed)
        .map_err(|e| PrintError::ImageDecode(e.to_string()))
}

/// Prepara los píxeles en formato BGRA top-down 32-bit para `StretchDIBits`.
pub fn prepare_page_dib(
    raw_bytes: &[u8],
    color_mode: PrintColorMode,
    orientation: PrintOrientation,
    printer_is_landscape: bool,
) -> Result<(u32, u32, Vec<u8>)> {
    let mut img: DynamicImage =
        image::load_from_memory(raw_bytes).map_err(|e| PrintError::ImageDecode(e.to_string()))?;

    if color_mode == PrintColorMode::Grayscale {
        img = DynamicImage::ImageLuma8(img.to_luma8());
    }

    let (w, h) = img.dimensions();
    let img_is_landscape = w > h;
    let target_landscape = match orientation {
        PrintOrientation::Auto => img_is_landscape,
        PrintOrientation::Landscape => true,
        PrintOrientation::Portrait => false,
    };

    if target_landscape != printer_is_landscape && img_is_landscape != printer_is_landscape {
        img = img.rotate90();
    }

    let rgba = img.to_rgba8();
    let (final_w, final_h) = rgba.dimensions();
    let mut bgra = Vec::with_capacity((final_w * final_h * 4) as usize);
    for px in rgba.pixels() {
        let [r, g, b, a] = px.0;
        // Composición sobre fondo blanco para evitar transparencias negras en GDI
        let alpha = a as u16;
        let inv = 255 - alpha;
        let out_r = ((r as u16 * alpha + 255 * inv) / 255) as u8;
        let out_g = ((g as u16 * alpha + 255 * inv) / 255) as u8;
        let out_b = ((b as u16 * alpha + 255 * inv) / 255) as u8;
        bgra.extend_from_slice(&[out_b, out_g, out_r, 255]);
    }

    Ok((final_w, final_h, bgra))
}
