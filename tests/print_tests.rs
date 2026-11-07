use image::{ImageBuffer, Rgba};
use yeib_print::{
    compute_target_rect, list_printers, parse_page_range, prepare_page_dib, PrintColorMode,
    PrintOrientation, PrintScaleMode,
};

fn sample_png(w: u32, h: u32) -> Vec<u8> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(w, h, |x, _| Rgba([x as u8, 128, 255, 255]));
    let mut bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    bytes
}

#[test]
fn test_parse_page_range_happy_and_edges() {
    assert_eq!(parse_page_range("", 5).unwrap(), vec![0, 1, 2, 3, 4]);
    assert_eq!(parse_page_range("all", 3).unwrap(), vec![0, 1, 2]);
    assert_eq!(parse_page_range("1-3, 5", 6).unwrap(), vec![0, 1, 2, 4]);
    assert_eq!(parse_page_range("2, 2, 1-2", 4).unwrap(), vec![1, 0]);
    assert!(parse_page_range("0", 5).is_err());
    assert!(parse_page_range("1-6", 5).is_err());
    assert!(parse_page_range("4-2", 5).is_err());
    assert!(parse_page_range("", 0).is_err());
}

#[test]
fn test_compute_target_rect_modes() {
    let (x, y, w, h) = compute_target_rect(1000, 500, 2000, 2000, PrintScaleMode::Fit);
    assert_eq!(w, 2000);
    assert_eq!(h, 1000);
    assert_eq!(x, 0);
    assert_eq!(y, 500);

    let (_, _, fw, fh) = compute_target_rect(1000, 500, 2000, 2000, PrintScaleMode::Fill);
    assert_eq!(fw, 4000);
    assert_eq!(fh, 2000);
}

#[test]
fn test_prepare_page_dib_color_and_grayscale() {
    let png = sample_png(40, 20);
    let (w, h, bgra) =
        prepare_page_dib(&png, PrintColorMode::Grayscale, PrintOrientation::Auto, true).unwrap();
    assert_eq!((w, h), (40, 20));
    assert_eq!(bgra.len(), 40 * 20 * 4);
    // En escala de grises B == G == R
    assert_eq!(bgra[0], bgra[1]);
    assert_eq!(bgra[1], bgra[2]);
    assert_eq!(bgra[3], 255);
}

#[test]
fn test_list_printers_does_not_panic() {
    let printers = list_printers().unwrap();
    // Si hay impresoras en el sistema, a lo sumo una es predeterminada y va primero
    if let Some(first) = printers.first() {
        assert!(!first.name.is_empty());
    }
}
