// spec-vis/src/main.rs

mod scalc;
mod srend;

use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CliWindowType {
    Hann,
    Hamming,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CliColorScheme {
    Oceanic,
    Grayscale,
    Inferno,
    Viridis,
    Synthwave,
    Sunset,
}

/// Генерирует спектрограмму из WAV файла
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Window function type (hann or hamming, default - hann)
    #[arg(short = 'w', long = "window-type", value_enum, default_value_t = CliWindowType::Hann)]
    window_type: CliWindowType,

    /// Color scheme (oceanic, grayscale, inferno, viridis, synthware, sunsen, default - oceanic)
    #[arg(short = 'c', long = "color-scheme", value_enum, default_value_t = CliColorScheme::Oceanic)]
    color_scheme: CliColorScheme,

    /// Target image size in WxH format (default - 2048x512)
    #[arg(short = 'i', long = "image-size", default_value = "2048x512")]
    image_size: String,

    /// Сохранять предварительный эскиз спектра (по умолчанию - true)
    #[arg(short = 'p', long = "preview-save", default_value_t = true)]
    preview_save: bool,

    /// Имя файла с сигналом
    file_name: String,

    /// Размерность FFT (по умолчанию - 2048)
    #[arg(short = 'f', long = "fft-size", default_value_t = 2048)]
    fft_size: usize,

    /// Шаг окна (hop length) в сэмплах
    #[arg(long, default_value_t = 512)]
    hop_length: usize,
}

// Преобразования CLI-типов во внутренние типы
impl From<CliWindowType> for scalc::WindowType {
    fn from(w: CliWindowType) -> Self {
        match w {
            CliWindowType::Hann => scalc::WindowType::Hann,
            CliWindowType::Hamming => scalc::WindowType::Hamming,
        }
    }
}

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

fn main() {
    let args = Args::parse();

    println!("Параметры выполнения:");
    println!("  Входной файл: {}", args.file_name);
    let (width, height) = parse_image_size(&args.image_size);
    println!("  Размер изображения: {}x{}", width, height);
    println!(
        "  Параметры STFT: FFT size = {}, Hop length = {}, Window type = {:?}",
        args.fft_size, args.hop_length, args.window_type
    );
    println!("--------------------------------------------------");

    // --- Этап 1: Вычисление данных ---
    println!("Этап 1: Вычисление данных спектрограммы...");
    let start_calc = Instant::now();

    let params = scalc::CalcParams {
        n_fft: args.fft_size,
        hop_length: args.hop_length,
        window_size: args.fft_size,
        window_type: args.window_type.into(),
    };

    let pb = ProgressBar::new(1); // Длина установится в коллбэке
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
        .unwrap()
        .progress_chars("#>-"));

    use std::path::Path;
    let spec_data_result = scalc::calculate_spectrogram(Path::new(&args.file_name), params, |processed, total| {
        pb.set_length(total as u64);
        pb.set_position(processed as u64);
    });

    pb.finish_with_message("Расчет завершен");

    let spec_data = match spec_data_result {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Ошибка при расчете спектрограммы: {}", e);
            return;
        }
    };
    println!("  Завершено за: {:.2?}", start_calc.elapsed());

    // --- Этап 2: Создание изображения ---
    println!("\nЭтап 2: Формирование изображения...");
    let start_view = Instant::now();

    let image = srend::create_spectrogram_image(&spec_data, width, height, args.color_scheme.into());

    println!("  Завершено за: {:.2?}", start_view.elapsed());

    // --- Этап 3: Сохранение файла ---
    println!("\nЭтап 3: Сохранение файла...");
    let output_path = format!("{}.png", args.file_name);
    match image.save(&output_path) {
        Ok(_) => println!(
            "  Изображение успешно сохранено в {}",
            output_path
        ),
        Err(e) => eprintln!("  Ошибка при сохранении изображения: {}", e),
    }

    println!("\nРабота завершена.");
}
