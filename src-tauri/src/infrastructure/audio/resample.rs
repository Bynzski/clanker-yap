//! Audio resampling to whisper.cpp required 16 kHz mono f32.

use crate::domain::constants::WHISPER_SAMPLE_RATE;

/// Resamples audio to 16 kHz mono f32.
pub fn resample(samples: &[f32], input_rate: u32, _output_rate: u32) -> Vec<f32> {
    if input_rate == WHISPER_SAMPLE_RATE {
        return samples.to_vec();
    }

    // Use linear interpolation for simple resampling
    let ratio = input_rate as f32 / WHISPER_SAMPLE_RATE as f32;
    let output_len = (samples.len() as f32 / ratio) as usize;
    let mut result = Vec::with_capacity(output_len);
    
    for i in 0..output_len {
        let src_idx = i as f32 * ratio;
        let src_idx0 = src_idx as usize;
        let src_idx1 = (src_idx0 + 1).min(samples.len().saturating_sub(1));
        let frac = src_idx - src_idx0 as f32;
        result.push(samples[src_idx0] * (1.0 - frac) + samples[src_idx1] * frac);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_48k_to_16k() {
        // Generate a 1 second sine wave at 48 kHz (48000 samples)
        let sample_rate: f32 = 48000.0;
        let frequency: f32 = 440.0; // A4 note
        let duration_samples = sample_rate as usize;
        
        let samples: Vec<f32> = (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (t * frequency * 2.0 * std::f32::consts::PI).sin()
            })
            .collect();

        let resampled = resample(&samples, 48000, 16000);
        
        // 1 second at 48kHz = 48000 samples
        // After resampling to 16kHz = 16000 samples
        let expected_len = 16000;
        assert_eq!(resampled.len(), expected_len, "Resampled length should be 16000");
        
        // Verify we have non-zero energy
        let energy: f32 = resampled.iter().map(|s| s * s).sum::<f32>() / resampled.len() as f32;
        assert!(energy > 0.1, "Resampled signal should have non-trivial energy");
    }

    #[test]
    fn test_no_resample_when_same_rate() {
        let samples: Vec<f32> = (0..100).map(|i| i as f32 * 0.01).collect();
        let result = resample(&samples, 16000, 16000);
        assert_eq!(result.len(), 100);
        assert_eq!(result, samples);
    }
}