# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SGVR is a Rust-based spectrogram visualization tool that processes WAV audio files to generate spectrograms. The project implements a two-stage approach: data calculation and image rendering, optimized for handling large audio files efficiently.

## Core Architecture

- **`src/main.rs`**: CLI interface with argument parsing (clap) and orchestration of the processing pipeline
- **`src/scalc.rs`**: Spectral calculation module - handles WAV file reading and STFT computation using hound and rustfft
- **`src/srend.rs`**: Spectral rendering module - converts spectral data to images using the image crate
- **`tools/gen_test_wav.rs`**: Utility for generating test audio files with multiple frequency components

### Key Dependencies

- `hound`: WAV file I/O
- `rustfft`: Fast Fourier Transform computations
- `image`: Image processing and output
- `clap`: Command-line argument parsing
- `indicatif`: Progress bar visualization

## Build Commands

```bash
# Build the main application
cargo build --release

# Build and run
cargo run -- input.wav

# Build the test WAV generator
cargo build --bin gen_test_wav

# Run the test WAV generator
cargo run --bin gen_test_wav
```

## Usage

The application processes WAV files with configurable parameters:

```bash
# Basic usage
./sgvr input.wav

# With custom parameters
./sgvr -f 4096 -w hamming -i 1024x256 -c viridis -d 90 input.wav
```

Key parameters:

- `-f, --fft-size`: FFT size (default: 2048)
- `-w, --window-type`: Window function (hann, hamming)
- `-i, --image-size`: Output dimensions (WxH format)
- `-c, --color-scheme`: Color palette (oceanic, grayscale, inferno, viridis, synthwave, sunset)
- `-d, --dynamic-range`: Dynamic range in dB (default: 110)

## Two-Stage Processing Architecture

1. **Data Calculation Stage** (`scalc.rs`):
   - Streams WAV file to avoid memory issues with large files
   - Computes STFT using configurable window functions
   - Outputs "master spectrogram" data structure

2. **Image Rendering Stage** (`srend.rs`):
   - Scales spectral data to target image dimensions
   - Uses maximum aggregation to preserve transient events
   - Applies color mapping schemes

## Vendor Dependencies

The project includes vendored dependencies in the `vendor/` directory for offline builds. This ensures reproducible builds without network access.

## Project Context

This is part of a larger project plan to create a spectrogram visualization tool. The current implementation is the console application phase, with future plans for:

- Async/await support for backend integration
- Tauri desktop application integration
- Real-time preview capabilities

See `ai/task.md` for detailed technical specifications and implementation approach.

## Development Guidelines

- **Code Language**: All code, comments, and documentation should be written in English
- **User Communication**: Always respond to users in Russian language
- **Commit Messages**: Write all commit messages in English. Newer write about claude.ai in commit messages
- **Vendor Directory**: Never modify files in the `vendor/` directory - these are vendored dependencies
- **Code Style**: Follow Rust conventions and idiomatic patterns
- **Error Handling**: Use proper Result types and meaningful error messages
- **Performance**: Optimize for large file processing and memory efficiency
- **Testing**: Write unit tests for core algorithms and integration tests for the complete pipeline. Put tests in separate file with name `*_tests.rs
