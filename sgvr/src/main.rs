use clap::{Parser, ValueEnum};
use specv::{SpecvParams, WindowType as SpecvWindowType, ColorScheme as SpecvColorScheme};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Размерность FFT (по умолчанию - 2048)
    #[arg(short = 'f', long = "fft-size", default_value_t = 2048)]
    fft_size: usize,

    /// Тип оконной функции (hann или hamming, по умолчанию - hann)
    #[arg(short = 'w', long = "window-type", value_enum, default_value_t = WindowType::Hann)]
    window_type: WindowType,

    /// Размер целевого изображения в формате WxH (по умолчанию - 2048x512)
    #[arg(short = 'i', long = "image-size", default_value = "2048x512")]
    image_size: String,

    /// Цветовая схема (navy, gray, bloody, по умолчанию - navy)
    #[arg(short = 'c', long = "color-scheme", value_enum, default_value_t = ColorScheme::Navy)]
    color_scheme: ColorScheme,

    /// Сохранять предварительный эскиз спектра (по умолчанию - true)
    #[arg(short = 'p', long = "preview-save", default_value_t = true)]
    preview_save: bool,

    /// Имя файла с сигналом
    file_name: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum WindowType {
    Hann,
    Hamming,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ColorScheme {
    Navy,
    Gray,
    Bloody,
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let params = SpecvParams {
        fft_size: cli.fft_size,
        window_type: match cli.window_type {
            WindowType::Hann => SpecvWindowType::Hann,
            WindowType::Hamming => SpecvWindowType::Hamming,
        },
        image_size: parse_image_size(&cli.image_size),
        color_scheme: match cli.color_scheme {
            ColorScheme::Navy => SpecvColorScheme::Navy,
            ColorScheme::Gray => SpecvColorScheme::Gray,
            ColorScheme::Bloody => SpecvColorScheme::Bloody,
        },
        preview_save: cli.preview_save,
    };
    println!("Запуск обработки файла: {}...", cli.file_name);
    specv::process(params).await;
    println!("Готово!");
} 