use wasm_bindgen::prelude::*;
use js_sys::{Uint8Array, Uint8ClampedArray, Float32Array};

/// Benchmark: TypedArray/Buffer Operations
///
/// Simulates pixel buffer manipulation (like Canvas ImageData) and
/// audio buffer processing (like WebAudio).
///
/// Tests:
/// - TypedArray creation
/// - Buffer manipulation
/// - Zero-copy data transfer patterns
/// - Uint8ClampedArray (RGBA pixels)
/// - Float32Array (audio samples)
pub fn run() {
    // Test 1: Image buffer processing (1920×1080 RGBA)
    process_image_buffer(1920, 1080);

    // Test 2: Audio buffer processing (48kHz, 10 seconds, stereo)
    process_audio_buffer(48_000, 10);

    // Test 3: Large data transfer simulation
    transfer_large_buffer(1024 * 1024); // 1MB
}

/// Simulate image buffer processing (like Canvas operations)
fn process_image_buffer(width: u32, height: u32) {
    let pixel_count = width * height;
    let byte_count = pixel_count * 4; // RGBA

    // Create ImageData-style buffer
    let buffer = Uint8ClampedArray::new_with_length(byte_count);

    // Fill with gradient pattern
    for i in 0..pixel_count {
        let offset = i * 4;
        let x = i % width;
        let y = i / width;

        // Red channel: horizontal gradient
        buffer.set_index(offset, (x * 255 / width) as u8);
        // Green channel: vertical gradient
        buffer.set_index(offset + 1, (y * 255 / height) as u8);
        // Blue channel: fixed
        buffer.set_index(offset + 2, 128);
        // Alpha: opaque
        buffer.set_index(offset + 3, 255);
    }

    // Process: apply simple transformation (invert colors)
    for i in 0..pixel_count {
        let offset = i * 4;
        let r = buffer.get_index(offset);
        let g = buffer.get_index(offset + 1);
        let b = buffer.get_index(offset + 2);

        buffer.set_index(offset, 255 - r);
        buffer.set_index(offset + 1, 255 - g);
        buffer.set_index(offset + 2, 255 - b);
    }

    // Verify work was done
    assert_eq!(buffer.length(), byte_count);
}

/// Simulate audio buffer processing (like WebAudio operations)
fn process_audio_buffer(sample_rate: u32, duration_secs: u32) {
    let sample_count = sample_rate * duration_secs;
    let stereo_samples = sample_count * 2; // Left + Right channels

    // Create audio buffer
    let buffer = Float32Array::new_with_length(stereo_samples);

    // Generate sine wave (left channel) and cosine wave (right channel)
    let frequency = 440.0; // A4 note
    let two_pi = 2.0 * std::f32::consts::PI;

    for i in 0..sample_count {
        let t = i as f32 / sample_rate as f32;
        let angle = two_pi * frequency * t;

        // Left channel: sine
        buffer.set_index(i * 2, angle.sin() * 0.5);
        // Right channel: cosine
        buffer.set_index(i * 2 + 1, angle.cos() * 0.5);
    }

    // Apply processing: simple gain and compression
    for i in 0..stereo_samples {
        let sample = buffer.get_index(i);
        let processed = sample * 0.8; // Reduce gain
        let compressed = if processed > 0.5 {
            0.5 + (processed - 0.5) * 0.5
        } else if processed < -0.5 {
            -0.5 + (processed + 0.5) * 0.5
        } else {
            processed
        };
        buffer.set_index(i, compressed);
    }

    // Verify
    assert_eq!(buffer.length(), stereo_samples);
}

/// Simulate large buffer transfer and processing
fn transfer_large_buffer(size: u32) {
    // Create buffer
    let buffer = Uint8Array::new_with_length(size);

    // Fill with pattern
    for i in 0..size {
        buffer.set_index(i, (i % 256) as u8);
    }

    // Process: simple checksum calculation
    let mut checksum: u32 = 0;
    for i in 0..size {
        checksum = checksum.wrapping_add(buffer.get_index(i) as u32);
    }

    // Verify work done
    assert!(checksum > 0);
    assert_eq!(buffer.length(), size);
}

/// Test creating buffers from Rust Vec (zero-copy when possible)
#[wasm_bindgen]
pub fn create_buffer_from_vec(size: u32) -> Uint8Array {
    let mut vec = Vec::with_capacity(size as usize);
    for i in 0..size {
        vec.push((i % 256) as u8);
    }

    // This should be zero-copy in many cases
    let array = Uint8Array::from(&vec[..]);
    array
}
