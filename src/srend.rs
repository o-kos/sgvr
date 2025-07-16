use super::scalc::SpectrogramData;
use image::{Rgb, RgbImage};
use hsl::HSL;

/// RGB color structure for gradients and colormaps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    const fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
    const fn new_rgb(rgb: u32) -> Self {
        Self { 
            r: ((rgb >> 16) & 0xFF) as u8, 
            g: ((rgb >>  8) & 0xFF) as u8, 
            b: (rgb         & 0xFF) as u8 
        } 
    }
}

/// Supported color schemes for spectrogram rendering
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ColorScheme {
    Oceanic,   // linear-gradient(to right, #01041B, #072e69, #4da4d5, #dcf3ff)
    Grayscale, // linear-gradient(to right, #000000, #888888, #ffffff)
    Inferno,   // linear-gradient(to right, #000004, #3b0f70, #ac255e, #f98e09, #fcfd21)
    Viridis,   // linear-gradient(to right, #440154, #3b528b, #21918c, #5ec962, #fde725)
    Synthwave, // linear-gradient(to right, #0d0221, #2d134b, #a537fd, #00f6ff)
    Sunset,    // linear-gradient(to right, #3c031c, #9c1521, #fd6a02, #fec812)
}

const OCEANIC: [Color; 4] = [
    Color::new_rgb(0x01041B), 
    Color::new_rgb(0x072e69),
    Color::new_rgb(0x4da4d5),
    Color::new_rgb(0xdcf3ff),
];

const GRAYSCALE: [Color; 3] = [
    Color::new_rgb(0x000000),
    Color::new_rgb(0x888888),
    Color::new_rgb(0xffffff),
];

const INFERNO: [Color; 5] = [
    Color::new_rgb(0x000004),
    Color::new_rgb(0x3b0f70),
    Color::new_rgb(0xac255e),
    Color::new_rgb(0xf98e09),
    Color::new_rgb(0xfcfd21),
];

const VIRIDIS: [Color; 5] = [
    Color::new_rgb(0x440154),
    Color::new_rgb(0x3b528b),
    Color::new_rgb(0x21918c),
    Color::new_rgb(0x5ec962),
    Color::new_rgb(0xfde725),
];

const SYNTHWAVE: [Color; 4] = [
    Color::new_rgb(0x0d0221),
    Color::new_rgb(0x2d134b),
    Color::new_rgb(0xa537fd),
    Color::new_rgb(0x00f6ff),
];

const SUNSET: [Color; 4] = [
    Color::new_rgb(0x3c031c),
    Color::new_rgb(0x9c1521),
    Color::new_rgb(0xfd6a02),
    Color::new_rgb(0xfec812),
];

fn get_color_stops(scheme: ColorScheme) -> &'static [Color] {
    match scheme {
        ColorScheme::Oceanic   => &OCEANIC,
        ColorScheme::Grayscale => &GRAYSCALE,
        ColorScheme::Inferno   => &INFERNO,
        ColorScheme::Viridis   => &VIRIDIS,
        ColorScheme::Synthwave => &SYNTHWAVE,
        ColorScheme::Sunset    => &SUNSET,
    }
}

/// Create a spectrogram image from data, with given size, color scheme, and dynamic range (dB)
///
/// - `spec_data`: Spectrogram data (matrix of dB values)
/// - `width`, `height`: Output image size in pixels
/// - `color_scheme`: Color scheme for rendering
/// - `dynamic_range`: Dynamic range in dB (e.g., 110.0)
///
/// Returns: RGB image
pub fn create_spectrogram_image(
    spec_data: &SpectrogramData,
    width: u32,
    height: u32,
    color_scheme: ColorScheme,
    dynamic_range: f32,
) -> RgbImage {
    let color_stops = get_color_stops(color_scheme);
    let gradient = generate_gradient_hsl(color_stops);

    let mut img = RgbImage::new(width, height);

    if spec_data.data.is_empty() {
        return img;
    }

    let master_width  = spec_data.data.len();     
    let master_height = spec_data.data[0].len(); 

    // Find global min and max dB for color normalization
    let max_db = spec_data.data.iter()
        .flat_map(|col| col.iter())
        .cloned()
        .fold(f32::MIN, f32::max);
    let min_db = max_db - dynamic_range;

    for x in 0..width {
        // Determine the range of columns in master data covered by this pixel column `x`
        let start_col = (x as usize * master_width) / width as usize;
        let end_col = ((x as usize + 1) * master_width) / width as usize;

        let end_col = end_col.max(start_col + 1);

        for y in 0..height {
            // Scale vertical axis (frequencies) using nearest neighbor interpolation
            // Invert `y` because (0,0) is top-left in image, but we want low frequencies at the bottom
            let freq_bin_index = ((height - 1 - y) as usize * master_height) / height as usize;

            // Find MAX value in [start_col, end_col) for this frequency bin 
            // for preserves peaks and short events
            let mut max_val = f32::NEG_INFINITY;
            for i in start_col..end_col {
                if let Some(col) = spec_data.data.get(i) {
                    if let Some(val) = col.get(freq_bin_index) {
                        if *val > max_val {
                            max_val = *val;
                        }
                    }
                }
            }

            // Normalize value and map to color using the selected gradient
            let normalized_val = (max_val - min_db) / (max_db - min_db);
            let idx = (normalized_val.clamp(0.0, 1.0) * (GRADIENT_SIZE as f32 - 1.0)).round() as usize;
            let idx = idx.min(GRADIENT_SIZE - 1);
            let c = gradient[idx];
            img.put_pixel(x, y, Rgb([c.r, c.g, c.b]));
        }
    }

    img
}

const GRADIENT_SIZE: usize = 256;

/// Generate a smooth HSL gradient from a list of color stops
///
/// - `stops`: Reference colors (at least 2)
///
/// Returns: Array of 256 interpolated Color values
fn generate_gradient_hsl(stops: &[Color]) -> [Color; GRADIENT_SIZE] {
    if stops.is_empty() { panic!("List of reference colors cannot be empty"); }
    if stops.len() == 1 { return [stops[0]; GRADIENT_SIZE]; }

    // Convert our RGB colors to HSL
    let hsl_stops: Vec<HSL> = stops.iter()
        .map(|c| HSL::from_rgb(&[c.r, c.g, c.b]))
        .collect();

    let mut gradient = [Color::new(0, 0, 0); GRADIENT_SIZE];
    let num_segments = hsl_stops.len() - 1;

    for i in 0..GRADIENT_SIZE {
        let progress = i as f64 / (GRADIENT_SIZE - 1) as f64;

        let (segment_index, segment_progress) = if progress >= 1.0 {
            (num_segments - 1, 1.0)
        } else {
            let segment_float = progress * num_segments as f64;
            (segment_float.floor() as usize, segment_float.fract())
        };

        let start_hsl = hsl_stops[segment_index];
        let end_hsl = hsl_stops[segment_index + 1];

        // Interpolation of H, S, L components

        // S and L are interpolated linearly, as before
        let s = start_hsl.s + (end_hsl.s - start_hsl.s) * segment_progress;
        let l = start_hsl.l + (end_hsl.l - start_hsl.l) * segment_progress;

        // For Hue we need special logic for the "short path" around the circle
        let mut h_start = start_hsl.h;
        let h_end = end_hsl.h;
        let h_diff = h_end - h_start;

        if h_diff.abs() > 180.0 {
            if h_diff > 0.0 {
                h_start += 360.0;
            } else {
                h_start -= 360.0;
            }
        }
        let h = (h_start + (h_end - h_start) * segment_progress) % 360.0;

        let new_hsl = HSL { h, s, l };

        // Convert the result back to RGB
        let (r, g, b) = new_hsl.to_rgb();
        gradient[i] = Color::new(r, g, b);
    }

    gradient
}

#[cfg(test)]
mod tests {
    include!("srend_tests.rs");
}