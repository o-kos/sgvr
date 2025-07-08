// spec-vis/src/main.rs

mod scalc;
mod sview;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;

/// Генерирует спектрограмму из WAV файла
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Путь к входному WAV файлу
    #[arg(short, long)]
    input: PathBuf,

    /// Путь для сохранения выходного PNG файла
    #[arg(short, long, default_value = "spectrogram.png")]
    output: PathBuf,

    /// Ширина выходного изображения в пикселях
    #[arg(long, default_value_t = 1200)]
    width: u32,

    /// Высота выходного изображения в пикселях
    #[arg(long, default_value_t = 800)]
    height: u32,

    /// Размер окна FFT (Fast Fourier Transform)
    #[arg(long, default_value_t = 2048)]
    fft_size: usize,

    /// Шаг окна (hop length) в сэмплах
    #[arg(long, default_value_t = 512)]
    hop_length: usize,
}

fn main() {
    let args = Args::parse();

    println!("Параметры выполнения:");
    println!("  Входной файл: {}", args.input.display());
    println!("  Выходной файл: {}", args.output.display());
    println!("  Размер изображения: {}x{}", args.width, args.height);
    println!(
        "  Параметры STFT: FFT size = {}, Hop length = {}",
        args.fft_size, args.hop_length
    );
    println!("--------------------------------------------------");

    // --- Этап 1: Вычисление данных ---
    println!("Этап 1: Вычисление данных спектрограммы...");
    let start_calc = Instant::now();

    let params = scalc::CalcParams {
        n_fft: args.fft_size,
        hop_length: args.hop_length,
        window_size: args.fft_size, // Для простоты берем размер окна равным размеру FFT
    };

    let pb = ProgressBar::new(1); // Длина установится в коллбэке
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
        .unwrap()
        .progress_chars("#>-"));

    let spec_data_result = scalc::calculate_spectrogram(&args.input, params, |processed, total| {
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

    let image = sview::create_spectrogram_image(&spec_data, args.width, args.height);

    println!("  Завершено за: {:.2?}", start_view.elapsed());

    // --- Этап 3: Сохранение файла ---
    println!("\nЭтап 3: Сохранение файла...");
    match image.save(&args.output) {
        Ok(_) => println!(
            "  Изображение успешно сохранено в {}",
            args.output.display()
        ),
        Err(e) => eprintln!("  Ошибка при сохранении изображения: {}", e),
    }

    println!("\nРабота завершена.");
}
