// spec-vis/src/main.rs

mod scalc;
mod srend;

use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

/// Window function type for FFT
#[derive(Copy, Clone, Debug, ValueEnum)]
enum CliWindowType {
    /// Hann window
    Hann,
    /// Hamming window
    Hamming,
}

/// Color scheme for spectrogram rendering
#[derive(Copy, Clone, Debug, ValueEnum)]
enum CliColorScheme {
    /// Oceanic: blue gradients
    Oceanic,
    /// Grayscale: black to white
    Grayscale,
    /// Inferno: perceptually uniform, dark to bright
    Inferno,
    /// Viridis: perceptually uniform, greenish
    Viridis,
    /// Synthwave: purple/cyan
    Synthwave,
    /// Sunset: red/orange/yellow
    Sunset,
}

/// Generates a spectrogram from a WAV file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Window function type
    #[arg(short = 'w', long = "window-type", value_enum, default_value_t = CliWindowType::Hann)]
    window_type: CliWindowType,

    /// Color scheme
    #[arg(short = 'c', long = "color-scheme", value_enum, default_value_t = CliColorScheme::Oceanic)]
    color_scheme: CliColorScheme,

    /// Target image size in WxH format (e.g. 2048x512)
    #[arg(short = 'i', long = "image-size", default_value = "2048x512")]
    image_size: String,

    /// Save preview spectrogram
    #[arg(short = 'p', long = "preview-save", default_value_t = true)]
    preview_save: bool,

    /// Input signal filename
    file_name: String,

    /// FFT size
    #[arg(short = 'f', long = "fft-size", default_value_t = 2048)]
    fft_size: usize,

    /// Hop length
    #[arg(long, default_value_t = 512)]
    hop_length: usize,

    /// Dynamic range in dB
    #[arg(short = 'd', long = "dynamic-range", default_value_t = 110.0)]
    dynamic_range: f32,
}

/// Convert CLI window type to internal window type
impl From<CliWindowType> for scalc::WindowType {
    fn from(w: CliWindowType) -> Self {
        match w {
            CliWindowType::Hann => scalc::WindowType::Hann,
            CliWindowType::Hamming => scalc::WindowType::Hamming,
        }
    }
}

/// Convert CLI color scheme to internal color scheme
impl From<CliColorScheme> for srend::ColorScheme {
    fn from(c: CliColorScheme) -> Self {
        match c {
            CliColorScheme::Grayscale => srend::ColorScheme::Grayscale,
            CliColorScheme::Inferno => srend::ColorScheme::Inferno,
            CliColorScheme::Oceanic => srend::ColorScheme::Oceanic,
            CliColorScheme::Sunset => srend::ColorScheme::Sunset,
            CliColorScheme::Synthwave => srend::ColorScheme::Synthwave,
            CliColorScheme::Viridis => srend::ColorScheme::Viridis,
        }
    }
}

/// Parse image size from string in WxH format
fn parse_image_size(s: &str) -> (u32, u32) {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse().unwrap_or(2048);
        let h = parts[1].parse().unwrap_or(512);
        (w, h)
    } else {
        (2048, 512)
    }
}

/// Main entry point
fn main() {
    let args = Args::parse();

    println!("Process file: '{}'", args.file_name);
    let (width, height) = parse_image_size(&args.image_size);
    println!("Generate {}x{}px spec image with color scheme '{:?}'", width, height, args.color_scheme);
    println!(
        "FFT size = {}, Hop length = {}, Window type = {:?}, Dynamic range = {} dB",
        args.fft_size, args.hop_length, args.window_type, args.dynamic_range
    );
    println!();

    // --- Step 1: Data calculation ---
    println!("Step 1: Calculating spectrogram data...");
    let start_calc = Instant::now();

    let params = scalc::CalcParams {
        n_fft: args.fft_size,
        hop_length: args.hop_length,
        window_size: args.fft_size,
        window_type: args.window_type.into(),
    };

    let pb = ProgressBar::new(1); // Length will be set in callback
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
        .unwrap()
        .progress_chars("#>-"));

    use std::path::Path;
    let spec_data_result = scalc::calculate_spectrogram(Path::new(&args.file_name), params, |processed, total| {
        pb.set_length(total as u64);
        pb.set_position(processed as u64);
    });

    pb.finish_with_message("Calculation completed");

    let spec_data = match spec_data_result {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error calculating spectrogram: {}", e);
            return;
        }
    };
    println!("  Completed in: {:.2?}", start_calc.elapsed());

    // --- Step 2: Image creation ---
    println!("\nStep 2: Creating image...");
    let start_view = Instant::now();

    let image = srend::create_spectrogram_image(&spec_data, width, height, args.color_scheme.into(), args.dynamic_range);

    println!("  Completed in: {:.2?}", start_view.elapsed());

    // --- Step 3: File saving ---
    println!("\nStep 3: Saving file...");
    let output_path = format!("{}.png", args.file_name);
    match image.save(&output_path) {
        Ok(_) => println!(
            "  Image successfully saved to {}",
            output_path
        ),
        Err(e) => eprintln!("  Error saving image: {}", e),
    }

    println!("\nWork completed.");
}
