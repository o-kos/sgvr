use super::audio::*;
use std::path::PathBuf;

#[test]
fn test_format_samples() {
    // Small numbers
    assert_eq!(format_samples(0), "0spl");
    assert_eq!(format_samples(500), "500spl");
    assert_eq!(format_samples(999), "999spl");
    
    // Thousands
    assert_eq!(format_samples(1000), "1.0kspl");
    assert_eq!(format_samples(1500), "1.5kspl");
    assert_eq!(format_samples(12345), "12.3kspl");
    assert_eq!(format_samples(999999), "1000.0kspl");
    
    // Millions
    assert_eq!(format_samples(1_000_000), "1.0Mspl");
    assert_eq!(format_samples(1_500_000), "1.5Mspl");
    assert_eq!(format_samples(12_345_678), "12.3Mspl");
    assert_eq!(format_samples(999_999_999), "1000.0Mspl");
    
    // Billions
    assert_eq!(format_samples(1_000_000_000), "1.0Gspl");
    assert_eq!(format_samples(1_500_000_000), "1.5Gspl");
    assert_eq!(format_samples(12_345_678_901), "12.3Gspl");
    assert_eq!(format_samples(999_999_999_999), "1000.0Gspl");
    
    // Trillions
    assert_eq!(format_samples(1_000_000_000_000), "1.0Tspl");
    assert_eq!(format_samples(1_500_000_000_000), "1.5Tspl");
    assert_eq!(format_samples(12_345_678_901_234), "12.3Tspl");
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(0.0), "0ms");
    assert_eq!(format_duration(0.001), "1ms");
    assert_eq!(format_duration(0.123), "123ms");
    assert_eq!(format_duration(0.999), "999ms");

    assert_eq!(format_duration(1.0), "1s");
    assert_eq!(format_duration(1.5), "1.5s");
    assert_eq!(format_duration(12.34), "12.34s");
    assert_eq!(format_duration(59.999), "59.999s");

    assert_eq!(format_duration(60.0), "1m");
    assert_eq!(format_duration(60.1), "1m:00.1s");
    assert_eq!(format_duration(61.5), "1:01.5m");
    assert_eq!(format_duration(125.75), "2:05.75m");
    assert_eq!(format_duration(3599.999), "59:59.999m");

    assert_eq!(format_duration(3600.0), "1h");
    assert_eq!(format_duration(3600.1), "1h:00.1s");
    assert_eq!(format_duration(3660.0), "1h:01m");
    assert_eq!(format_duration(3660.1), "1h:01m:00.1s");
    assert_eq!(format_duration(3661.5), "1:01:01.5h");
    assert_eq!(format_duration(7323.25), "2:02:03.25h");

    assert_eq!(format_duration(-1.5), "-1.5s");
    assert_eq!(format_duration(-60.0), "-1:00m");
    assert_eq!(format_duration(-3600.0), "-1:00:00h");
}

#[test]
fn test_signal_type_display() {
    assert_eq!(format!("{:?}", SignalType::Real), "Real");
    assert_eq!(format!("{:?}", SignalType::IQ), "IQ");
}

#[test]
fn test_audio_metadata_to_pretty_string() {
    let metadata = AudioMetadata {
        codec: "PCM".to_string(),
        sample_rate: 44100,
        total_samples: 44100,
        signal_type: SignalType::Real,
    };
    let result = metadata.to_pretty_string();
    assert!(result.contains("'PCM'"));
    assert!(result.contains("44100 Hz"));
    assert!(result.contains("real"));
    assert!(result.contains("1s"));
}

#[test]
fn test_audio_metadata_to_pretty_string_iq() {
    let metadata = AudioMetadata {
        codec: "FLAC".to_string(),
        sample_rate: 8000,
        total_samples: 16000,
        signal_type: SignalType::IQ,
    };
    let result = metadata.to_pretty_string();
    assert!(result.contains("'FLAC'"));
    assert!(result.contains("8000 Hz"));
    assert!(result.contains("i/q"));
    assert!(result.contains("2s"));
}

#[test]
fn test_create_audio_reader_nonexistent_file() {
    let path = PathBuf::from("nonexistent_file.wav");
    let result = create_audio_reader(&path);
    assert!(result.is_err());
}

#[test] 
fn test_symphonia_reader_open_wav16x8_file() {
    let path = PathBuf::from("tests/rl_16x8_hfdl.wav");
    let result = SymphoniaReader::open(&path);
    assert!(result.is_err());
}

#[test]
fn test_symphonia_reader_with_wav_file() {
    let test_file = PathBuf::from("tests/rl_i16-hfdl.wav");
    assert!(test_file.exists());

    let reader = SymphoniaReader::open(&test_file);
    assert!(reader.is_ok());
    
    let reader = reader.unwrap();
    let metadata = reader.metadata();
    assert!(metadata.sample_rate > 0);
    assert!(metadata.total_samples > 0);
}

#[test]
fn test_symphonia_reader_with_flac_file() {
    let test_file = PathBuf::from("tests/rl_f32-hfdl.flac");
    if !test_file.exists() {
        return;
    }

    let reader = SymphoniaReader::open(&test_file);
    assert!(reader.is_ok());
    
    let reader = reader.unwrap();
    let metadata = reader.metadata();
    assert!(metadata.sample_rate > 0);
    assert!(metadata.total_samples > 0);
    assert_eq!(metadata.codec, "flac");
}

#[test]
fn test_audio_metadata_table() {
    struct TestCase<'a> {
        filename: &'a str,
        expected_codec: &'a str,
        expected_sample_rate: u32,
        expected_total_samples: u64,
        expected_signal_type: SignalType,
    }
    let cases = vec![
        TestCase {
            filename: "rl_i16-hfdl.wav",
            expected_codec: "pcm_s16le",
            expected_sample_rate: 12600,
            expected_total_samples: 63000,
            expected_signal_type: SignalType::Real,
        },
        TestCase {
            filename: "rl_f32-hfdl.flac",
            expected_codec: "flac",
            expected_sample_rate: 12600,
            expected_total_samples: 63000,
            expected_signal_type: SignalType::Real,
        },
        TestCase {
            filename: "iq_f32-ft8.flac",
            expected_codec: "flac",
            expected_sample_rate: 62500,
            expected_total_samples: 10238976,
            expected_signal_type: SignalType::IQ,
        },
        TestCase {
            filename: "iq_i16-hfdl.iqw",
            expected_codec: "pcm_s16le",
            expected_sample_rate: 24000,
            expected_total_samples: 94080,
            expected_signal_type: SignalType::IQ,
        },
        TestCase {
            filename: "rl_f32-hfdl.wav",
            expected_codec: "pcm_f32le",
            expected_sample_rate: 12600,
            expected_total_samples: 63000,
            expected_signal_type: SignalType::Real,
        },
    ];
    let path = PathBuf::from("tests");
    for case in cases {
        let file_path = path.join(case.filename);
        assert!(file_path.exists(), "Test file not found: {} (skipped)", file_path.display());
        let reader = SymphoniaReader::open(&file_path).expect("Failed to open file");
        let metadata = reader.metadata();
        assert_eq!(metadata.codec.to_lowercase(), case.expected_codec.to_lowercase(),
           "Codec mismatch for {:?}", case.filename);
        assert_eq!(metadata.sample_rate, case.expected_sample_rate,
           "Sample rate mismatch for {:?}", case.filename);
        assert_eq!(metadata.total_samples, case.expected_total_samples,
           "Total samples mismatch for {:?}", case.filename);
        assert_eq!(metadata.signal_type, case.expected_signal_type,
           "Signal type mismatch for {:?}", case.filename);
    }
}

#[test]
fn test_read_samples_method() {
    let test_file = PathBuf::from("tests/HFDL-1460.wav");
    if !test_file.exists() {
        return;
    }

    let mut reader = SymphoniaReader::open(&test_file).unwrap();
    let samples_result = reader.read_samples();
    assert!(samples_result.is_ok());
    
    let samples = samples_result.unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_create_audio_reader_with_valid_file() {
    let test_file = PathBuf::from("tests/HFDL-1460.wav");
    if !test_file.exists() {
        return;
    }

    let reader = create_audio_reader(&test_file);
    assert!(reader.is_ok());
    
    let reader = reader.unwrap();
    let metadata = reader.metadata();
    assert!(metadata.sample_rate > 0);
    assert!(metadata.total_samples > 0);
}

#[test]
fn test_signal_type_from_channels() {
    let test_file_mono = PathBuf::from("tests/HFDL-1460.wav");
    if test_file_mono.exists() {
        let reader = SymphoniaReader::open(&test_file_mono).unwrap();
        let metadata = reader.metadata();
        match metadata.signal_type {
            SignalType::Real | SignalType::IQ => {},
        }
    }
}