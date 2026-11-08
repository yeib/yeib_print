use yeib_print::{compute_target_rect, parse_page_range, PrintScaleMode};

#[test]
fn test_parse_page_range_empty_or_all() {
    assert_eq!(parse_page_range("", 5).unwrap(), vec![0, 1, 2, 3, 4]);
    assert_eq!(parse_page_range("all", 3).unwrap(), vec![0, 1, 2]);
    assert_eq!(parse_page_range("*", 2).unwrap(), vec![0, 1]);
}

#[test]
fn test_parse_page_range_single_and_ranges() {
    assert_eq!(parse_page_range("1", 5).unwrap(), vec![0]);
    assert_eq!(parse_page_range("1-3", 5).unwrap(), vec![0, 1, 2]);
    assert_eq!(parse_page_range("1, 3, 5", 5).unwrap(), vec![0, 2, 4]);
    assert_eq!(parse_page_range(" 2 - 4 , 5 ", 5).unwrap(), vec![1, 2, 3, 4]);
}

#[test]
fn test_parse_page_range_deduplication() {
    assert_eq!(parse_page_range("1-3, 2, 3", 5).unwrap(), vec![0, 1, 2]);
}

#[test]
fn test_parse_page_range_errors() {
    assert!(parse_page_range("0", 5).is_err());
    assert!(parse_page_range("6", 5).is_err());
    assert!(parse_page_range("4-2", 5).is_err());
    assert!(parse_page_range("abc", 5).is_err());
    assert!(parse_page_range("all", 0).is_err());
}

#[test]
fn test_compute_target_rect_fit() {
    // 1000x2000 image inside 2000x2000 page -> scale 1.0 -> dst: 1000x2000, x: 500, y: 0
    let (x, y, w, h) = compute_target_rect(1000, 2000, 2000, 2000, PrintScaleMode::Fit);
    assert_eq!((x, y, w, h), (500, 0, 1000, 2000));
}

#[test]
fn test_compute_target_rect_fill() {
    // 1000x2000 image inside 2000x2000 page -> scale 2.0 -> dst: 2000x4000, x: 0, y: 0
    let (x, y, w, h) = compute_target_rect(1000, 2000, 2000, 2000, PrintScaleMode::Fill);
    assert_eq!(w, 2000);
    assert_eq!(h, 4000);
    assert_eq!(x, 0);
    assert_eq!(y, 0);
}
