use std::path::PathBuf;

mod audio;
use audio::*;

fn main() {
    let path = PathBuf::from("tests/rl_i16-hfdl.wav");
    let mut reader = SymphoniaReader::open(&path).unwrap();
    
    // Test first 4 samples at position 0
    reader.seek(0).expect("Failed to seek to beginning");
    let mut samples_0 = vec![0.0f32; 4];
    let count = reader.read(&mut samples_0).expect("Failed to read first samples");
    
    println!("First 4 samples:");
    for (i, sample) in samples_0.iter().enumerate() {
        println!("  samples_0[{}] = {:.8}", i, sample);
    }
    
    // Test samples at offset 50400
    reader.seek(50400).expect("Failed to seek to offset");
    let mut samples_1 = vec![0.0f32; 4];
    let count = reader.read(&mut samples_1).expect("Failed to read offset samples");
    
    println!("Samples at offset 50400:");
    for (i, sample) in samples_1.iter().enumerate() {
        println!("  samples_1[{}] = {:.8}", i, sample);
    }
}