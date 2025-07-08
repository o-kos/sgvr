use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum WindowType {
    Hann,
    Hamming,
}

impl FromStr for WindowType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hann" => Ok(WindowType::Hann),
            "hamming" => Ok(WindowType::Hamming),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColorScheme {
    Navy,
    Gray,
    Bloody,
}

impl FromStr for ColorScheme {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "navy" => Ok(ColorScheme::Navy),
            "gray" => Ok(ColorScheme::Gray),
            "bloody" => Ok(ColorScheme::Bloody),
            _ => Err(()),
        }
    }
}

pub struct SpecvParams {
    pub fft_size: usize,
    pub window_type: WindowType,
    pub image_size: (u32, u32),
    pub color_scheme: ColorScheme,
    pub preview_save: bool,
}

pub async fn process(params: SpecvParams) {
    // Здесь будет основная логика обработки
    println!("[specv] Обработка с параметрами: {params:?}");
    // имитация асинхронной работы
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    println!("[specv] Обработка завершена");
} 