use std::error::Error;
use std::fs::File;
use std::path::Path;

use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

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

pub struct SymphoniaReader {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: u16,
    time_base: TimeBase,
    sample_buf: Option<SampleBuffer<f32>>, // remaining samples from the previous read call
    buf_pos: usize,
}

impl SymphoniaReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let path_ref = path.as_ref();
        let src = File::open(path_ref)?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(extension) = path_ref.extension().and_then(|s| s.to_str()) {
            hint.with_extension(extension);
        }
        let format_opts = FormatOptions { ..Default::default() };
        let metadata_opts = MetadataOptions { ..Default::default() };
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)?;

        let reader = probed.format;

        let track = reader.default_track().ok_or("Missing default track")?;
        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        let sample_rate = codec_params.sample_rate.ok_or("Missing sample rate")?;
        let channels = codec_params.channels.ok_or("Missing channels")?.count() as u16;
        let time_base = codec_params.time_base.ok_or("Missing time base")?;

        let decoder_opts = DecoderOptions { ..Default::default() };
        let decoder = symphonia::default::get_codecs().make(&codec_params, &decoder_opts)?;

        // fn sample_rate(&self) -> u32 { self.sample_rate }
        // fn channels(&self) -> u16 { self.channels } 
        // // Symphonia работает на уровне кодеков, не всегда предоставляя исходную битность.
        // fn bits_per_sample(&self) -> u16 { 0 } 
   
        Ok(Self {
            reader,
            decoder,
            track_id,
            sample_rate,
            channels,
            time_base,
            sample_buf: None,
            buf_pos: 0,
        })
    }
}

impl AudioReader for SymphoniaReader {
    fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }

    fn seek(&mut self, frame_num: u64) -> Result<(), Box<dyn Error>> {
        let time = self.time_base.calc_time(frame_num);
        self.reader.seek(
            SeekMode::Accurate,
            SeekTo::Time { time, track_id: Some(self.track_id) }
        )?;
        Ok(())
    }

// Трейт AudioReader и структура SymphoniaReader остаются без изменений

impl AudioReader for SymphoniaReader {
    // Методы sample_rate, channels, bits_per_sample, seek остаются без изменений.
    // Они были корректны.

    fn read(&mut self, buf: &mut [f32]) -> Result<usize, Box<dyn Error>> {
        let mut samples_written = 0;
        let num_channels = self.channels as usize;
        let buf_len_samples = buf.len();

        // Цикл продолжается, пока мы не заполним предоставленный буфер `buf`
        while samples_written < buf_len_samples {
            // ШАГ 1: Проверить, есть ли данные в нашем внутреннем буфере `self.sample_buf`
            if let Some(sample_buf) = self.sample_buf.as_mut() {
                // Сколько сэмплов осталось в нашем внутреннем буфере
                let remaining_in_buf = sample_buf.samples().len() - self.buf_pos;
                // Сколько сэмплов нам нужно скопировать в `buf`
                let to_copy = (buf_len_samples - samples_written).min(remaining_in_buf);
                
                if to_copy > 0 {
                    let src_slice = &sample_buf.samples()[self.buf_pos..self.buf_pos + to_copy];
                    let dst_slice = &mut buf[samples_written..samples_written + to_copy];
                    dst_slice.copy_from_slice(src_slice);

                    samples_written += to_copy;
                    self.buf_pos += to_copy;
                }

                // Если мы прочитали все из нашего внутреннего буфера, очищаем его
                if self.buf_pos >= sample_buf.samples().len() {
                    self.sample_buf = None;
                    self.buf_pos = 0;
                }
            }
            
            // Если `buf` уже заполнен, выходим
            if samples_written >= buf_len_samples {
                break;
            }

            // ШАГ 2: Если внутреннего буфера нет или он опустел, читаем новый пакет из файла
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(ref err))
                    if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break; // Нормальный конец файла
                }
                Err(e) => return Err(Box::new(e)),
            };

            // Пропускаем пакеты не из нашего трека
            if packet.track_id() != self.track_id {
                continue;
            }

            // ШАГ 3: Декодируем пакет в новый `AudioBufferRef`
            match self.decoder.decode(&packet)? {
                AudioBufferRef::F32(decoded) => {
                    // Если декодированный пакет пуст, пропускаем
                    if decoded.frames() == 0 {
                        continue;
                    }

                    // --- ЕДИНСТВЕННО ВЕРНЫЙ КОД ---
                    // Создаем наш собственный управляемый буфер, используя
                    // .frames() для получения количества кадров.
                    let mut new_s_buf = SampleBuffer::<f32>::new(decoded.frames() as u64, *decoded.spec());
                    
                    // Копируем чередующиеся (interleaved) сэмплы из `decoded` в наш буфер.
                    new_s_buf.copy_interleaved_ref(decoded);
                    
                    // Сохраняем этот новый буфер для использования на этой и следующей итерации.
                    self.sample_buf = Some(new_s_buf);
                    self.buf_pos = 0;
                }
                // Пропускаем другие форматы сэмплов для простоты
                _ => continue,
            }
        }
        
        // Возвращаем количество записанных КАДРОВ
        let frames_written = samples_written / num_channels;
        Ok(frames_written)
    }
}}