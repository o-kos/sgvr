#[allow(unused_imports)]
use super::*;

#[test]
fn test_parse_image_size_valid() {
    let (w, h) = parse_image_size("1024x768");
    assert_eq!(w, 1024);
    assert_eq!(h, 768);
}

#[test]
fn test_parse_image_size_default() {
    let (w, h) = parse_image_size("invalid");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_empty() {
    let (w, h) = parse_image_size("");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_single_number() {
    let (w, h) = parse_image_size("1024");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_invalid_numbers() {
    let (w, h) = parse_image_size("abcxdef");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_zero() {
    let (w, h) = parse_image_size("0x0");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_negative() {
    let (w, h) = parse_image_size("-1x-1");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_with_whitespace() {
    let (w, h) = parse_image_size(" 1024x768 ");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_partial_valid() {
    let (w, h) = parse_image_size("1024xabc");
    assert_eq!(w, 1024);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_multiple_x() {
    let (w, h) = parse_image_size("1024x768x256");
    assert_eq!(w, DEFAULT_IMAGE_WIDTH);
    assert_eq!(h, DEFAULT_IMAGE_HEIGHT);
}

#[test]
fn test_parse_image_size_large_numbers() {
    let (w, h) = parse_image_size("4096x2048");
    assert_eq!(w, 4096);
    assert_eq!(h, 2048);
}

#[test]
fn test_cli_window_type_conversion() {
    assert_eq!(scalc::WindowType::Hann, CliWindowType::Hann.into());
    assert_eq!(scalc::WindowType::Hamming, CliWindowType::Hamming.into());
}

#[test]
fn test_cli_color_scheme_conversion() {
    assert_eq!(srend::ColorScheme::Oceanic, CliColorScheme::Oceanic.into());
    assert_eq!(srend::ColorScheme::Grayscale, CliColorScheme::Grayscale.into());
    assert_eq!(srend::ColorScheme::Inferno, CliColorScheme::Inferno.into());
    assert_eq!(srend::ColorScheme::Viridis, CliColorScheme::Viridis.into());
    assert_eq!(srend::ColorScheme::Synthwave, CliColorScheme::Synthwave.into());
    assert_eq!(srend::ColorScheme::Sunset, CliColorScheme::Sunset.into());
}

#[test]
fn test_cli_window_type_debug() {
    let window_type = CliWindowType::Hann;
    let debug_str = format!("{:?}", window_type);
    assert_eq!(debug_str, "Hann");
}

#[test]
fn test_cli_color_scheme_debug() {
    let color_scheme = CliColorScheme::Oceanic;
    let debug_str = format!("{:?}", color_scheme);
    assert_eq!(debug_str, "Oceanic");
}

#[test]
fn test_cli_enum_equality() {
    assert_eq!(CliWindowType::Hann, CliWindowType::Hann);
    assert_ne!(CliWindowType::Hann, CliWindowType::Hamming);
    
    assert_eq!(CliColorScheme::Oceanic, CliColorScheme::Oceanic);
    assert_ne!(CliColorScheme::Oceanic, CliColorScheme::Grayscale);
}