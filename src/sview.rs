use super::scalc::SpectrogramData;
use image::{Rgb, RgbImage};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ColorScheme {
    Navy,
    Gray,
    Bloody,
}


/// Основная функция: создает изображение из данных спектрограммы
pub fn create_spectrogram_image(spec_data: &SpectrogramData, width: u32, height: u32, color_scheme: ColorScheme) -> RgbImage {
    let _ = color_scheme;
    let mut img = RgbImage::new(width, height);

    if spec_data.data.is_empty() {
        return img;
    }

    let master_width = spec_data.data.len(); // Количество временных отсчетов
    let master_height = spec_data.data[0].len(); // Количество частотных бинов

    // Находим глобальный минимум и максимум dB для нормализации цвета
    let (min_db, max_db) = find_db_range(&spec_data.data);

    for x in 0..width {
        // Определяем диапазон столбцов в мастер-данных, который покрывает
        // текущий пиксельный столбец `x`.
        let start_col = (x as usize * master_width) / width as usize;
        let end_col = ((x as usize + 1) * master_width) / width as usize;

        // Если диапазон пуст (при сильном растяжении), берем один столбец
        let end_col = end_col.max(start_col + 1);

        for y in 0..height {
            // Масштабируем вертикальную ось (частоты)
            // Используем простую интерполяцию по ближайшему соседу.
            // Инвертируем `y`, т.к. в изображении (0,0) - верхний левый угол,
            // а мы хотим низкие частоты внизу.
            let freq_bin_index = ((height - 1 - y) as usize * master_height) / height as usize;

            // Находим МАКСИМАЛЬНОЕ значение в диапазоне [start_col, end_col)
            // для данной частоты. Это сохраняет пики и короткие события.
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

            // Нормализуем значение и преобразуем в цвет
            let normalized_val = (max_val - min_db) / (max_db - min_db);
            let color = value_to_rgb(normalized_val.clamp(0.0, 1.0));
            img.put_pixel(x, y, color);
        }
    }

    img
}

/// Находит минимальное и максимальное значение dB в данных
fn find_db_range(data: &[Vec<f32>]) -> (f32, f32) {
    let mut min_db = f32::MAX;
    let mut max_db = f32::MIN;
    for col in data {
        for &val in col {
            if val < min_db {
                min_db = val;
            }
            if val > max_db {
                max_db = val;
            }
        }
    }
    // Для адекватного вида можно ограничить динамический диапазон
    let min_db = max_db - 80.0; // Показываем диапазон в 80 dB
    (min_db, max_db)
}

/// Преобразует нормализованное значение (0.0 .. 1.0) в цвет Rgb
/// Простой colormap: черный -> синий -> зеленый -> желтый -> красный
fn value_to_rgb(value: f32) -> Rgb<u8> {
    let r = (255.0 * (value * 2.0).min(1.0)) as u8;
    let g = (255.0 * (value * 2.0 - 0.5).max(0.0).min(1.0)) as u8;
    let b = (255.0 * (1.0 - value * 2.0).max(0.0)) as u8;
    Rgb([r, g, b])
}
