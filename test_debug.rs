use std::path::PathBuf;

// Copy the structs and functions we need
use std::error::Error;
use std::fs::File;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::TimeBase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    Real,
    IQ
}

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub codec: String,
    pub sample_rate: u32,
    pub total_samples: u64,
    pub signal_type: SignalType,
}

pub struct SymphoniaReader {
    metadata: AudioMetadata,
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    channels: u16,
    time_base: TimeBase,
    sample_buf: Option<SampleBuffer<f32>>,
    buf_pos: usize,
}

impl SymphoniaReader {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn Error>> {
        let path_ref = path.as_ref();
        let src = File::open(path_ref)?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(extension) = path_ref.extension().and_then(|s| s.to_str()) {
            hint.with_extension(extension);
        }
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)?;

        let reader = probed.format;

        let track = reader.default_track().ok_or("Missing default track")?;
        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        let channels = codec_params.channels.ok_or("Missing channels")?.count() as u16;
        let signal_type = match channels {
            1 => SignalType::Real,
            2 => SignalType::IQ,
            _ => return Err(format!("Unsupported channels count: {channels}").into()),
        };
        let sample_rate = codec_params.sample_rate.ok_or("Missing sample rate")?;
        let time_base = codec_params.time_base.ok_or("Missing time base")?;

        let decoder_opts = DecoderOptions { ..Default::default() };
        let decoder = symphonia::default::get_codecs().make(&codec_params, &decoder_opts)?;

        let total_samples = codec_params.n_frames.unwrap_or(0);

        let registry = symphonia::default::get_codecs();
        let codec_name = registry
            .get_codec(codec_params.codec)
            .map(|codec_type| codec_type.short_name)
            .unwrap_or("Unknown")
            .to_string();

        let metadata = AudioMetadata {
            codec: codec_name,
            sample_rate,
            total_samples,
            signal_type,
        };

        Ok(Self {
            reader,
            decoder,
            track_id,
            channels,
            time_base,
            sample_buf: None,
            buf_pos: 0,
            metadata,
        })
    }

    pub fn seek(&mut self, frame_num: u64) -> Result<(), Box<dyn Error>> {
        let time = self.time_base.calc_time(frame_num);
        self.reader.seek(
            SeekMode::Accurate,
            SeekTo::Time { time, track_id: Some(self.track_id) }
        )?;
        self.sample_buf = None;
        self.buf_pos = 0;
        Ok(())
    }

    pub fn read(&mut self, buf: &mut [f32]) -> Result<usize, Box<dyn Error>> {
        let mut samples_written = 0;
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
        
        Ok(samples_written)
    }
}

fn main() {
    let path = PathBuf::from("tests/rl_i16-hfdl.wav");
    let mut reader = SymphoniaReader::open(&path).unwrap();
    
    // Test first 4 samples at position 0
    reader.seek(0).expect("Failed to seek to beginning");
    let mut samples_0 = vec![0.0f32; 4];
    let count = reader.read(&mut samples_0).expect("Failed to read first samples");
    
    println!("First 4 samples (count={}):", count);
    for (i, sample) in samples_0.iter().enumerate() {
        println!("  samples_0[{}] = {:.8}", i, sample);
    }
    
    println!("Expected:");
    println!("  samples_0[0] = -0.076110840");
    println!("  samples_0[1] = -0.063842770");
    println!("  samples_0[2] = 0.028442380");
    println!("  samples_0[3] = 0.068939210");
    
    // Test samples at offset 50400
    reader.seek(50400).expect("Failed to seek to offset");
    let mut samples_1 = vec![0.0f32; 4];
    let count = reader.read(&mut samples_1).expect("Failed to read offset samples");
    
    println!("\nSamples at offset 50400 (count={}):", count);
    for (i, sample) in samples_1.iter().enumerate() {
        println!("  samples_1[{}] = {:.8}", i, sample);
    }
    
    println!("Expected:");
    println!("  samples_1[0] = 0.176666260");
    println!("  samples_1[1] = 0.090545650");
    println!("  samples_1[2] = 0.021575930");
    println!("  samples_1[3] = 0.035095210");
}
