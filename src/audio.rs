use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum SignalType {
    Real,
    IQ
}

#[derive(Debug, Clone, Copy)]
pub enum SampleType {
    U8,
    I16,
    I24,
    I32,
    F32,
}

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub codec: String,
    pub sample_rate: u32,
    pub total_samples: u64,
    pub signal_type: SignalType,
    pub sample_type: SampleType,
}

fn format_duration(duration: f64) -> String {
    if duration < 0.0 {
        return format!("-{}", format_duration(-duration));
    }

    if duration < 1.0 {
        let ms = (duration * 1000.0).round() as u32;
        return format!("{}ms", ms);
    }

    let zero_ms = ((duration * 1000.0).round() as u32) % 1000 == 0;
    if duration < 60.0 {
        return if zero_ms {
            format!("{:.0}s", duration)
        } else {
            format!("{:.3}s", duration)
        }
    }  
    
    if duration < 3600.0 {
        let minutes = (duration / 60.0).trunc() as u32;
        let seconds = duration % 60.0;
        return if zero_ms {
            format!("{}:{:02.0}m", minutes, seconds)
        } else {
            format!("{}:{:06.3}m", minutes, seconds)
        }
    } 
    
    let hours = (duration / 3600.0).trunc() as u32;
    let remainder = duration % 3600.0;
    let minutes = (remainder / 60.0).trunc() as u32;
    let seconds = remainder % 60.0;
    if zero_ms {
        format!("{}:{:02}:{:02.0}h", hours, minutes, seconds)
    } else {
        format!("{}:{:02}:{:06.3}h", hours, minutes, seconds)
    }
}

impl AudioMetadata {
    pub fn to_pretty_string(&self) -> String {
        let total_seconds = self.total_samples as f64 / self.sample_rate as f64;
        format!(
            "'{}', {} Hz, {} {}, {}",
            self.codec,
            self.sample_rate,
            match self.signal_type {
                SignalType::Real => "real",
                SignalType::IQ => "i/q",
            },
            match self.sample_type {
                SampleType::U8 => "u8",
                SampleType::I16 => "i16",
                SampleType::I24 => "i24",
                SampleType::I32 => "i32",
                SampleType::F32 => "f32",
            },
            format_duration(total_seconds)
        )
    }
}

pub trait AudioReader {
    fn metadata(&self) -> &AudioMetadata;
    
    fn seek(&mut self, sample_num: u64) -> Result<(), Box<dyn Error>>;
    fn read(&mut self, samples: &mut [f32]) -> Result<usize, Box<dyn Error>>;
}

pub fn create_audio_reader(path: &Path) -> Result<Box<dyn AudioReader>, Box<dyn Error>> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        "wav" => {
            Ok(Box::new(WavReader::open(path)?))
        },
        "flac" => {
            Ok(Box::new(FlacReader::open(path)?))
        },
        _ => {
            match WavReader::open(path) {
                Ok(reader) => Ok(Box::new(reader)),
                Err(_) => Err(format!("Unsupported audio format: '{}' from path {:?}", extension, path).into()),
            }
        },
    }
}

pub struct WavReader {
    metadata: AudioMetadata,
    reader: hound::WavReader<std::io::BufReader<std::fs::File>>,
}

impl WavReader {
    pub fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        let reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        
        let metadata = AudioMetadata {
            codec: "wav".to_string(),
            sample_rate: spec.sample_rate,
            total_samples: reader.duration() as u64,
            signal_type: match spec.channels {
                1 => SignalType::Real,
                2 => SignalType::IQ,
                _ => return Err(format!("Unsupported channels count: {}", spec.channels).into()),
            },
            sample_type: match spec.sample_format {
                hound::SampleFormat::Int => {
                    match spec.bits_per_sample {
                        8 => SampleType::U8,
                        16 => SampleType::I16,
                        24 => SampleType::I24,
                        32 => SampleType::I32,
                        _ => SampleType::I32, 
                    }
                },
                hound::SampleFormat::Float => SampleType::F32,
            },
        };
        
        Ok(WavReader { 
            metadata, 
            reader
        })
    }
}

impl AudioReader for WavReader {
    fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }
    
    fn seek(&mut self, sample_num: u64) -> Result<(), Box<dyn Error>> {
        let sample_num_32: u32 = sample_num.try_into()
            .map_err(|e| format!("Seek position {} is too large for a WAV file: {}", sample_num, e))?;
        self.reader.seek(sample_num_32).map_err(|e| e.into())
    }
    
    fn read(&mut self, samples: &mut [f32]) -> Result<usize, Box<dyn Error>> {
        let samples_iterator = self.reader.samples::<f32>();

        let mut read_count = 0;
        for (i, sample_result) in samples_iterator.take(samples.len()).enumerate() {
            let sample = sample_result?;
            samples[i] = sample;
            read_count += 1;
        }
        if matches!(self.metadata.signal_type, SignalType::Real) { read_count /= 2 }
        Ok(read_count)
    }
}

pub struct FlacReader {
    metadata: AudioMetadata,
    reader: claxon::FlacReader<std::io::BufReader<std::fs::File>>,
}

impl FlacReader {
    pub fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        let reader = claxon::FlacReader::open(path)?;
        let info = reader.streaminfo();
        
        let metadata = AudioMetadata {
            codec: "flac".to_string(),
            sample_rate: info.sample_rate,
            total_samples: info.samples.unwrap_or(0),
            signal_type: match info.channels {
                1 => SignalType::Real,
                2 => SignalType::IQ,
                _ => return Err(format!("Unsupported channels count: {}", info.channels).into()),
            },
            sample_type: SampleType::F32, // FlacReader always reads as F32
        };
        
        Ok(FlacReader { 
            metadata, 
            reader
        })
    }
}

impl AudioReader for FlacReader {
    fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }
    
    fn seek(&mut self, sample_num: u64) -> Result<(), Box<dyn Error>> {
        self.reader.seek(sample_num).map_err(|e| e.into())
    }
    
    fn read(&mut self, samples: &mut [f32], count: usize) -> Result<usize, Box<dyn Error>> {
        let remaining_samples = (self.samples.len() as u64 - self.current_position) as usize;
        let samples_to_read = std::cmp::min(count, remaining_samples);
        let samples_to_read = std::cmp::min(samples_to_read, samples.len());
        
        if samples_to_read == 0 {
            return Ok(0);
        }
        
        let start_pos = self.current_position as usize;
        let end_pos = start_pos + samples_to_read;
        
        samples[..samples_to_read].copy_from_slice(&self.samples[start_pos..end_pos]);
        self.current_position += samples_to_read as u64;
        
        Ok(samples_to_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flac_reader_seek() {
        // Создаем mock FlacReader с тестовыми данными
        let metadata = AudioMetadata {
            codec: "flac".to_string(),
            sample_rate: 44100,
            total_samples: 10,
            signal_type: SignalType::Real,
            sample_type: SampleType::F32,
        };
        
        let test_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let mut reader = FlacReader {
            metadata,
            samples: test_samples,
            current_position: 0,
        };
        
        // Тест seek
        assert!(reader.seek(5).is_ok());
        assert_eq!(reader.current_position, 5);
        
        // Тест seek за пределы
        assert!(reader.seek(15).is_err());
        
        // Тест read после seek
        let mut buffer = [0.0f32; 3];
        let read_count = reader.read(&mut buffer, 3).unwrap();
        assert_eq!(read_count, 3);
        assert_eq!(buffer, [0.6, 0.7, 0.8]);
        assert_eq!(reader.current_position, 8);
    }

    #[test]
    fn test_flac_reader_read() {
        let metadata = AudioMetadata {
            codec: "flac".to_string(),
            sample_rate: 44100,
            total_samples: 5,
            signal_type: SignalType::Real,
            sample_type: SampleType::F32,
        };
        
        let test_samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut reader = FlacReader {
            metadata,
            samples: test_samples,
            current_position: 0,
        };
        
        // Читаем 3 сэмпла
        let mut buffer = [0.0f32; 3];
        let read_count = reader.read(&mut buffer, 3).unwrap();
        assert_eq!(read_count, 3);
        assert_eq!(buffer, [1.0, 2.0, 3.0]);
        assert_eq!(reader.current_position, 3);
        
        // Читаем оставшиеся 2 сэмпла
        let mut buffer2 = [0.0f32; 5];
        let read_count2 = reader.read(&mut buffer2, 5).unwrap();
        assert_eq!(read_count2, 2);
        assert_eq!(&buffer2[..2], &[4.0, 5.0]);
        assert_eq!(reader.current_position, 5);
        
        // Попытка чтения после конца файла
        let mut buffer3 = [0.0f32; 2];
        let read_count3 = reader.read(&mut buffer3, 2).unwrap();
        assert_eq!(read_count3, 0);
    }

    #[test]
    fn test_audio_metadata_pretty_string() {
        let metadata = AudioMetadata {
            codec: "flac".to_string(),
            sample_rate: 44100,
            total_samples: 44100, // 1 секунда
            signal_type: SignalType::Real,
            sample_type: SampleType::F32,
        };
        
        let pretty = metadata.to_pretty_string();
        assert!(pretty.contains("flac"));
        assert!(pretty.contains("44100 Hz"));
        assert!(pretty.contains("real"));
        assert!(pretty.contains("f32"));
        assert!(pretty.contains("1s"));
    }

    #[test]
    fn test_format_duration() {
        // Тест миллисекунд
        assert_eq!(format_duration(0.5), "500ms");
        assert_eq!(format_duration(0.001), "1ms");
        
        // Тест секунд
        assert_eq!(format_duration(1.0), "1s");
        assert_eq!(format_duration(5.123), "5.123s");
        
        // Тест минут
        assert_eq!(format_duration(65.0), "1:05m");
        assert_eq!(format_duration(125.456), "2:05.456m");
        
        // Тест часов
        assert_eq!(format_duration(3665.0), "1:01:05h");
        assert_eq!(format_duration(7325.123), "2:02:05.123h");
    }
}