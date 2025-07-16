#[allow(unused_imports)]
use super::*;

#[test]
fn test_color_new() {
    let color = Color::new(255, 128, 64);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
}

#[test]
fn test_color_new_rgb() {
    let color = Color::new_rgb(0xFF8040);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
}

#[test]
fn test_color_equality() {
    let color1 = Color::new(255, 128, 64);
    let color2 = Color::new(255, 128, 64);
    let color3 = Color::new(255, 128, 65);
    
    assert_eq!(color1, color2);
    assert_ne!(color1, color3);
}

#[test]
fn test_color_scheme_equality() {
    assert_eq!(ColorScheme::Oceanic, ColorScheme::Oceanic);
    assert_ne!(ColorScheme::Oceanic, ColorScheme::Grayscale);
}

#[test]
fn test_get_color_stops() {
    let oceanic_stops = get_color_stops(ColorScheme::Oceanic);
    assert_eq!(oceanic_stops.len(), 4);
    assert_eq!(oceanic_stops[0], Color::new_rgb(0x01041B));
    
    let grayscale_stops = get_color_stops(ColorScheme::Grayscale);
    assert_eq!(grayscale_stops.len(), 3);
    assert_eq!(grayscale_stops[0], Color::new_rgb(0x000000));
    assert_eq!(grayscale_stops[2], Color::new_rgb(0xffffff));
}

#[test]
fn test_generate_gradient_hsl_single_color() {
    let stops = [Color::new(255, 0, 0)];
    let gradient = generate_gradient_hsl(&stops);
    
    assert_eq!(gradient.len(), GRADIENT_SIZE);
    assert_eq!(gradient[0], Color::new(255, 0, 0));
    assert_eq!(gradient[GRADIENT_SIZE - 1], Color::new(255, 0, 0));
}

#[test]
fn test_generate_gradient_hsl_two_colors() {
    let stops = [Color::new(0, 0, 0), Color::new(255, 255, 255)];
    let gradient = generate_gradient_hsl(&stops);
    
    assert_eq!(gradient.len(), GRADIENT_SIZE);
    assert_eq!(gradient[0], Color::new(0, 0, 0));
    assert_eq!(gradient[GRADIENT_SIZE - 1], Color::new(255, 255, 255));
}

#[test]
#[should_panic(expected = "List of reference colors cannot be empty")]
fn test_generate_gradient_hsl_empty_stops() {
    let stops: &[Color] = &[];
    generate_gradient_hsl(stops);
}

#[test]
fn test_create_spectrogram_image_empty_data() {
    let spec_data = SpectrogramData { data: vec![] };
    let image = create_spectrogram_image(&spec_data, 100, 100, ColorScheme::Grayscale, 50.0);
    
    assert_eq!(image.width(), 100);
    assert_eq!(image.height(), 100);
}

#[test]
fn test_create_spectrogram_image_with_data() {
    let spec_data = SpectrogramData {
        data: vec![
            vec![-80.0, -70.0, -60.0],
            vec![-90.0, -50.0, -40.0],
            vec![-75.0, -65.0, -55.0],
        ]
    };
    
    let image = create_spectrogram_image(&spec_data, 10, 10, ColorScheme::Grayscale, 50.0);
    
    assert_eq!(image.width(), 10);
    assert_eq!(image.height(), 10);
}

#[test]
fn test_all_color_schemes_have_stops() {
    let schemes = [
        ColorScheme::Oceanic,
        ColorScheme::Grayscale,
        ColorScheme::Inferno,
        ColorScheme::Viridis,
        ColorScheme::Synthwave,
        ColorScheme::Sunset,
    ];
    
    for scheme in schemes {
        let stops = get_color_stops(scheme);
        assert!(!stops.is_empty(), "Color scheme {:?} should have color stops", scheme);
    }
}