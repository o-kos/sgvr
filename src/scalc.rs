use hound::WavReader;
use rustfft::{num_complex::Complex, FftPlanner};
use std::error::Error;
use std::path::Path;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum WindowType {
    Hann,
    Hamming,
}

/// Параметры для вычисления спектрограммы
#[derive(Debug, Clone, Copy)]
pub struct CalcParams {
    pub n_fft: usize,
    pub hop_length: usize,
    pub window_size: usize,
    pub window_type: WindowType,
}

/// Результат вычисления - "мастер-спектрограмма"
/// Содержит все необходимые данные для последующей визуализации
pub struct SpectrogramData {
    /// Данные спектрограммы: Vec<столбец_частот>
    /// Каждый столбец - это вектор амплитуд (в dB) для одного временного отсчета
    pub data: Vec<Vec<f32>>,
    /// Частота дискретизации исходного файла
    pub sample_rate: u32,
    /// Размер FFT, он же определяет количество частотных бинов
    pub n_fft: usize,
}

/// Основная функция модуля: читает WAV и вычисляет спектрограмму
pub fn calculate_spectrogram<F>(
    path: &Path,
    params: CalcParams,
    mut progress_callback: F,
) -> Result<SpectrogramData, Box<dyn Error>>
where
    F: FnMut(usize, usize),
{
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    // Читаем все сэмплы и конвертируем их в f32 в диапазоне [-1.0, 1.0]
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    // NOTE: Для ОЧЕНЬ больших файлов здесь нужна потоковая обработка,
    // а не загрузка всего файла в память. Но для демонстрации алгоритма
    // и для большинства файлов этот подход работает отлично и проще.

    let window = match params.window_type {
        WindowType::Hann => hann_window(params.window_size),
        WindowType::Hamming => hamming_window(params.window_size),
    };

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(params.n_fft);

    // Вычисляем общее количество временных кадров (столбцов спектрограммы)
    let total_frames = (samples.len() - params.window_size) / params.hop_length;
    let mut spectrogram_data: Vec<Vec<f32>> = Vec::with_capacity(total_frames);

    let mut frame_buffer = vec![Complex::new(0.0, 0.0); params.n_fft];

    // Двигаемся по сэмплам с шагом hop_length
    for i in 0..total_frames {
        let start = i * params.hop_length;
        let _end = start + params.window_size;

        // Копируем кадр данных в буфер, применяя оконную функцию
        for j in 0..params.window_size {
            frame_buffer[j].re = samples[start + j] * window[j];
            frame_buffer[j].im = 0.0;
        }
        // Дополняем нулями, если n_fft > window_size
        for j in params.window_size..params.n_fft {
            frame_buffer[j].re = 0.0;
            frame_buffer[j].im = 0.0;
        }

        // Выполняем FFT
        fft.process(&mut frame_buffer);

        // Вычисляем амплитуды (модуль) и конвертируем в dB
        // Нам нужна только первая половина спектра (n_fft / 2 + 1)
        let num_bins = params.n_fft / 2 + 1;
        let mut magnitudes_db = Vec::with_capacity(num_bins);
        for k in 0..num_bins {
            let magnitude = frame_buffer[k].norm();
            // Преобразуем в децибелы, добавляя малое число, чтобы избежать log10(0)
            let db = 20.0 * magnitude.max(1.0e-9).log10();
            magnitudes_db.push(db);
        }

        spectrogram_data.push(magnitudes_db);

        // Вызываем коллбэк для обновления прогресс-бара
        if i % 10 == 0 || i == total_frames - 1 {
            progress_callback(i + 1, total_frames);
        }
    }

    Ok(SpectrogramData {
        data: spectrogram_data,
        sample_rate: spec.sample_rate,
        n_fft: params.n_fft,
    })
}

/// Window function Hann
fn hann_window(size: usize) -> Vec<f32> {
    let mut window = Vec::with_capacity(size);
    for i in 0..size {
        let val = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32).cos());
        window.push(val);
    }
    window
}

/// Window function Hamming
fn hamming_window(size: usize) -> Vec<f32> {
    let mut window = Vec::with_capacity(size);
    for i in 0..size {
        let val = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32).cos();
        window.push(val);
    }
    window
}
