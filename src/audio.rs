use std::error::Error;
use std::fs::File;
use std::path::Path;

use symphonia::core::audio::{SampleBuffer, Signal};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::TimeBase;

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

    let mut ms = (duration * 1000.0).round() as u64;
    if duration < 1.0 {
        return format!("{}ms", ms);
    }

    let sec = ms / 1000;
    ms %= 1000;
    let ms_str = if ms != 0 {
        format!(".{}", ms).trim_end_matches('0').to_string()
    } else {
        String::new()
    };

    if sec < 60 {
        return format!("{}{}s", sec, ms_str);
    }  
    
    if sec < 3600 {
        let minutes = sec / 60;
        let seconds = sec % 60;
        return format!("{}:{:02}{}m", minutes, seconds, ms_str);
    } 
    
    let hours = sec / 3600;
    let remainder = sec % 3600;
    let minutes = remainder / 60;
    let seconds = remainder % 60;
    format!("{}:{:02}:{:02}{}h", hours, minutes, seconds, ms_str)
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
    fn read_samples(&mut self) -> Result<Vec<f32>, Box<dyn Error>>;
}

pub fn create_audio_reader(path: &Path) -> Result<Box<dyn AudioReader>, Box<dyn Error>> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        "wav" | "flac" => {
            Ok(Box::new(SymphoniaReader::open(path)?))
        },
        _ => {
            match SymphoniaReader::open(path) {
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
    sample_buf: Option<SampleBuffer<f32>>,
    buf_pos: usize,
    metadata: AudioMetadata,
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

        let total_samples = codec_params.n_frames.unwrap_or(0);
        let codec_name = codec_params.codec.to_string();
        
        let metadata = AudioMetadata {
            codec: codec_name,
            sample_rate,
            total_samples,
            signal_type: SignalType::Real,
            sample_type: SampleType::F32,
        };

        Ok(Self {
            reader,
            decoder,
            track_id,
            sample_rate,
            channels,
            time_base,
            sample_buf: None,
            buf_pos: 0,
            metadata,
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
        self.sample_buf = None;
        self.buf_pos = 0;
        Ok(())
    }

    fn read(&mut self, buf: &mut [f32]) -> Result<usize, Box<dyn Error>> {
        let mut samples_written = 0;
        let num_channels = self.channels as usize;
        let buf_len_samples = buf.len();

        while samples_written < buf_len_samples {
            if let Some(sample_buf) = self.sample_buf.as_mut() {
                let remaining_in_buf = sample_buf.samples().len() - self.buf_pos;
                let to_copy = (buf_len_samples - samples_written).min(remaining_in_buf);
                
                if to_copy > 0 {
                    let src_slice = &sample_buf.samples()[self.buf_pos..self.buf_pos + to_copy];
                    let dst_slice = &mut buf[samples_written..samples_written + to_copy];
                    dst_slice.copy_from_slice(src_slice);

                    samples_written += to_copy;
                    self.buf_pos += to_copy;
                }

                if self.buf_pos >= sample_buf.samples().len() {
                    self.sample_buf = None;
                    self.buf_pos = 0;
                }
            }
            
            if samples_written >= buf_len_samples {
                break;
            }
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(ref err))
                    if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(Box::new(e)),
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            let decoded = self.decoder.decode(&packet)?;
            if decoded.frames() == 0 {
                continue;
            }

            let mut new_s_buf = SampleBuffer::<f32>::new(decoded.frames() as u64, *decoded.spec());
            new_s_buf.copy_interleaved_ref(decoded);
            
            self.sample_buf = Some(new_s_buf);
            self.buf_pos = 0;
        }
        
        let frames_written = samples_written / num_channels;
        Ok(frames_written)
    }

    fn read_samples(&mut self) -> Result<Vec<f32>, Box<dyn Error>> {
        let total_samples = self.metadata.total_samples as usize * self.channels as usize;
        let mut samples = vec![0.0f32; total_samples];
        let mut pos = 0;
        
        while pos < total_samples {
            let remaining = total_samples - pos;
            let to_read = remaining.min(8192);
            let frames_read = self.read(&mut samples[pos..pos + to_read])?;
            
            if frames_read == 0 {
                break;
            }
            
            pos += frames_read * self.channels as usize;
        }
        
        samples.truncate(pos);
        Ok(samples)
    }
}