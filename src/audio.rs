use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_samples: u64,
}

pub trait AudioReader {
    fn metadata(&self) -> &AudioMetadata;
    fn read_samples(&mut self) -> Result<Vec<f32>, Box<dyn Error>>;
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
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            duration_samples: reader.duration() as u64,
        };
        
        Ok(WavReader { metadata, reader })
    }
}

impl AudioReader for WavReader {
    fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }
    
    fn read_samples(&mut self) -> Result<Vec<f32>, Box<dyn Error>> {
        let spec = self.reader.spec();
        let samples: Result<Vec<f32>, _> = match spec.sample_format {
            hound::SampleFormat::Int => {
                match spec.bits_per_sample {
                    16 => {
                        self.reader.samples::<i16>()
                            .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
                            .collect()
                    }
                    24 => {
                        self.reader.samples::<i32>()
                            .map(|s| s.map(|sample| sample as f32 / (1 << 23) as f32))
                            .collect()
                    }
                    32 => {
                        self.reader.samples::<i32>()
                            .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
                            .collect()
                    }
                    _ => return Err("Unsupported bit depth".into()),
                }
            }
            hound::SampleFormat::Float => {
                self.reader.samples::<f32>()
                    .collect()
            }
        };
        
        let mut samples = samples?;
        
        if spec.channels > 1 {
            samples = samples.into_iter()
                .enumerate()
                .filter(|(i, _)| i % spec.channels as usize == 0)
                .map(|(_, sample)| sample)
                .collect();
        }
        
        Ok(samples)
    }
}

pub struct FlacReader {
    metadata: AudioMetadata,
    samples: Vec<f32>,
}

impl FlacReader {
    pub fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut reader = claxon::FlacReader::open(path)?;
        let info = reader.streaminfo();
        
        let metadata = AudioMetadata {
            sample_rate: info.sample_rate,
            channels: info.channels as u16,
            duration_samples: info.samples.unwrap_or(0),
        };
        
        let mut samples = Vec::new();
        let max_value = (1 << (info.bits_per_sample - 1)) as f32;
        
        for sample in reader.samples() {
            let sample = sample? as f32 / max_value;
            samples.push(sample);
        }
        
        if info.channels > 1 {
            samples = samples.into_iter()
                .enumerate()
                .filter(|(i, _)| i % info.channels as usize == 0)
                .map(|(_, sample)| sample)
                .collect();
        }
        
        Ok(FlacReader { metadata, samples })
    }
}

impl AudioReader for FlacReader {
    fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }
    
    fn read_samples(&mut self) -> Result<Vec<f32>, Box<dyn Error>> {
        Ok(self.samples.clone())
    }
}