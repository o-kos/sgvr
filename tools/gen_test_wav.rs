// spec-vis/tools/gen_test_wav.rs

use hound::{WavSpec, WavWriter};
use std::f32::consts::PI;

const SAMPLE_RATE: u32 = 44100;
const DURATION_S: u32 = 10;
const FILENAME: &str = "test_signal.wav";

fn main() -> Result<(), hound::Error> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(FILENAME, spec)?;
    let num_samples = SAMPLE_RATE * DURATION_S;
    let amplitude = i16::MAX as f32 * 0.5;

    for t in 0..num_samples {
        let time = t as f32 / SAMPLE_RATE as f32;

        // Сигнал, состоящий из трех синусоид разной частоты и в разное время
        let mut sample = 0.0;

        // Низкая частота (220 Гц) на всем протяжении
        sample += (2.0 * PI * 220.0 * time).sin() * 0.4;

        // Средняя частота (880 Гц) в первой половине
        if time < (DURATION_S / 2) as f32 {
            sample += (2.0 * PI * 880.0 * time).sin() * 0.3;
        }

        // Высокая частота (3520 Гц) во второй половине
        if time >= (DURATION_S / 2) as f32 {
            sample += (2.0 * PI * 3520.0 * time).sin() * 0.3;
        }

        writer.write_sample((sample * amplitude) as i16)?;
    }

    writer.finalize()?;
    println!("Generated '{}' with duration {}s.", FILENAME, DURATION_S);
    Ok(())
}
