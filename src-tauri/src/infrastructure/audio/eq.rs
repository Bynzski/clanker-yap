//! FFT-based audio frequency band extraction for EQ visualization.
//!
//! Produces 7 log-spaced frequency bands from audio samples, suitable for
//! driving a multi-bar EQ display in the overlay pill.
//!
//! Follows Keyless's `EqState` pattern: Hann window → FFT → log-spaced bands
//! → attack/decay smoothing.

use realfft::{num_complex::Complex, num_traits::Zero, RealFftPlanner};

/// Number of frequency bands exposed for EQ visualization.
pub const EQ_BAND_COUNT: usize = 7;

/// Attack smoothing coefficient — how quickly bands rise to peak values.
const ATTACK_COEFF: f32 = 0.45;
/// Decay smoothing coefficient — how quickly bands fall from peak values.
const DECAY_COEFF: f32 = 0.30;

/// FFT size: 1024 samples per analysis window.
const FFT_SIZE: usize = 1024;

/// Lowest frequency bin to consider (band start).
const BAND_START_HZ: f32 = 120.0;

/// EQ state for real-time frequency band extraction.
/// Created per-recording session in the `Start` arm of `recorder_thread`.
pub struct EqState {
    /// Pre-computed log-spaced band boundaries (bin indices into FFT output).
    /// Each entry is `(start_bin, end_bin)` inclusive.
    band_bins: Vec<(usize, usize)>,
    /// Smoothed band values (0.0–1.0 normalized).
    smoothed_bands: Vec<f32>,
    /// Ring buffer of input samples for FFT.
    ring_buffer: Vec<f32>,
    /// Current write position in the ring buffer.
    write_pos: usize,
    /// Number of valid samples in the ring buffer.
    valid_count: usize,
    /// Pre-computed Hann window.
    hann_window: Vec<f32>,
    /// FFT planner — reused across frames.
    planner: RealFftPlanner<f32>,
    /// Output buffer for FFT (complex, size FFT_SIZE/2 + 1).
    fft_output: Vec<Complex<f32>>,
    /// Scratch buffer for FFT.
    fft_scratch: Vec<Complex<f32>>,
}

impl EqState {
    /// Creates a new EQ state for the given sample rate.
    /// Pre-computes FFT plan, Hann window, and log-spaced band bin ranges.
    pub fn new(sample_rate: u32) -> Self {
        let planner = RealFftPlanner::new();

        // Pre-compute Hann window
        let hann_window: Vec<f32> = (0..FFT_SIZE)
            .map(|i| {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32;
                0.5 * (1.0 - angle.cos())
            })
            .collect();

        // Pre-compute log-spaced band boundaries
        let band_bins = compute_band_bins(sample_rate, FFT_SIZE);

        Self {
            band_bins,
            smoothed_bands: vec![0.0; EQ_BAND_COUNT],
            ring_buffer: vec![0.0; FFT_SIZE],
            write_pos: 0,
            valid_count: 0,
            hann_window,
            planner,
            fft_output: vec![Complex::zero(); FFT_SIZE / 2 + 1],
            fft_scratch: vec![Complex::zero(); FFT_SIZE],
        }
    }

    /// Feeds samples into the EQ analyzer.
    ///
    /// Accumulates samples into an internal ring buffer. When `FFT_SIZE` samples
    /// have been accumulated, runs a full FFT analysis and returns the 7-band
    /// normalized values with attack/decay smoothing applied.
    ///
    /// Returns `None` if not enough samples have been accumulated yet.
    /// Returns `Some(bands)` with 7 f32 values in the range 0.0–1.0.
    pub fn feed(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        // Accumulate into ring buffer
        let mut consumed = 0;
        while consumed < samples.len() && self.valid_count < FFT_SIZE {
            let to_write = (samples.len() - consumed).min(FFT_SIZE - self.valid_count);
            let start = self.write_pos;
            let end = start + to_write;
            if end <= FFT_SIZE {
                self.ring_buffer[start..end]
                    .copy_from_slice(&samples[consumed..consumed + to_write]);
            } else {
                // Wrap around
                let first = FFT_SIZE - start;
                self.ring_buffer[start..].copy_from_slice(&samples[consumed..consumed + first]);
                self.ring_buffer[..to_write - first]
                    .copy_from_slice(&samples[consumed + first..consumed + to_write]);
            }
            self.write_pos = (self.write_pos + to_write) % FFT_SIZE;
            self.valid_count += to_write;
            consumed += to_write;
        }

        // Need at least FFT_SIZE valid samples to run FFT
        if self.valid_count < FFT_SIZE {
            return None;
        }

        // Build FFT input: grab FFT_SIZE samples ending at write_pos
        let mut fft_input = vec![0.0f32; FFT_SIZE];
        if self.write_pos >= FFT_SIZE {
            // Normal case: copy from write_pos - FFT_SIZE to write_pos
            fft_input.copy_from_slice(&self.ring_buffer[self.write_pos - FFT_SIZE..self.write_pos]);
        } else {
            // Wrap-around: copy end of buffer, then beginning
            let end_part = FFT_SIZE - self.write_pos;
            fft_input[..end_part].copy_from_slice(&self.ring_buffer[FFT_SIZE - end_part..]);
            fft_input[end_part..].copy_from_slice(&self.ring_buffer[..self.write_pos]);
        }

        // Run FFT
        let bands = self.run_fft(&fft_input);

        // Apply attack/decay smoothing
        for (i, &new_val) in bands.iter().enumerate() {
            let current = self.smoothed_bands[i];
            let smoothed = if new_val > current {
                // Attack: move toward peak
                current + ATTACK_COEFF * (new_val - current)
            } else {
                // Decay: fall away
                current + DECAY_COEFF * (new_val - current)
            };
            self.smoothed_bands[i] = smoothed;
        }

        Some(self.smoothed_bands.clone())
    }

    /// Runs FFT on the given input window and computes log-spaced band magnitudes.
    fn run_fft(&mut self, fft_input: &[f32]) -> Vec<f32> {
        // Build input with Hann window applied
        let mut windowed = vec![0.0f32; FFT_SIZE];
        for i in 0..FFT_SIZE {
            windowed[i] = fft_input[i] * self.hann_window[i];
        }

        // Plan and execute FFT
        let r2c = self.planner.plan_fft_forward(FFT_SIZE);
        r2c.process_with_scratch(&mut windowed, &mut self.fft_output, &mut self.fft_scratch)
            .ok();

        // Compute magnitudes (only need first half — DC to Nyquist)
        let half = FFT_SIZE / 2;
        let magnitudes: Vec<f32> = self.fft_output[..half]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        // Map magnitudes to log-spaced frequency bands
        let mut bands = vec![0.0f32; EQ_BAND_COUNT];
        for (i, &(start, end)) in self.band_bins.iter().enumerate() {
            let start = start.min(half - 1);
            let end = end.min(half - 1).max(start);

            let energy: f32 = magnitudes[start..=end].iter().copied().sum();
            let avg_energy = if end > start {
                energy / (end - start + 1) as f32
            } else {
                energy
            };

            // Normalize to 0.0–1.0 with a noise floor
            let normalized = (avg_energy / FFT_SIZE as f32 * 4.0).clamp(0.0, 1.0);
            bands[i] = normalized;
        }

        bands
    }
}

/// Computes log-spaced frequency band boundaries in FFT bin indices.
fn compute_band_bins(sample_rate: u32, fft_size: usize) -> Vec<(usize, usize)> {
    let bin_width = sample_rate as f32 / fft_size as f32;
    let nyquist = sample_rate as f32 / 2.0;

    // Log-spaced center frequencies from 120 Hz to ~8 kHz
    let center_hz: [f32; 7] = [120.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0];

    center_hz
        .iter()
        .map(|&center| {
            // Each band spans ±30% of center frequency
            let low_f = center * 0.7_f32;
            let high_f = center * 1.3_f32;
            let low = if low_f > BAND_START_HZ {
                low_f
            } else {
                BAND_START_HZ
            };
            let high = if high_f < nyquist { high_f } else { nyquist };

            let start_bin = (low / bin_width).round() as usize;
            let end_bin = (high / bin_width).round() as usize;

            (start_bin, end_bin)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_state_creation() {
        // Should not panic
        let eq = EqState::new(48000);
        assert_eq!(eq.smoothed_bands.len(), EQ_BAND_COUNT);
    }

    #[test]
    fn test_compute_band_bins() {
        let bins = compute_band_bins(48000, 1024);
        assert_eq!(bins.len(), EQ_BAND_COUNT);
        // Each band should have start <= end
        for (start, end) in &bins {
            assert!(
                start <= end,
                "Band start {} should be <= end {}",
                start,
                end
            );
        }
    }

    #[test]
    fn test_feed_returns_none_for_partial() {
        let mut eq = EqState::new(48000);
        // With only 3 samples, we don't have a full FFT window
        let result = eq.feed(&[0.1, 0.2, 0.3]);
        assert!(result.is_none());
    }

    #[test]
    fn test_feed_smoothing_attack_and_decay() {
        let mut eq = EqState::new(48000);

        // Feed a full window of zeros first
        let zeros = vec![0.0f32; FFT_SIZE];
        let result = eq.feed(&zeros);
        assert!(
            result.is_some(),
            "Feed with full FFT window should return Some"
        );
        let initial = result.unwrap();
        // All zeros should produce all-zero bands
        assert!(
            initial.iter().all(|&v| v <= 0.01),
            "Zeros should produce near-zero bands, got {:?}",
            initial
        );

        // Now feed a full window of signal
        let signal: Vec<f32> = (0..FFT_SIZE)
            .map(|i| {
                let t = i as f32 / 48000.0;
                (t * 440.0 * 2.0 * std::f32::consts::PI).sin() // 440 Hz sine wave
            })
            .collect();
        let result = eq.feed(&signal);
        assert!(result.is_some());
        let bands = result.unwrap();

        // Band values should be non-negative
        assert!(
            bands.iter().all(|&v| v >= 0.0),
            "Bands should be non-negative, got {:?}",
            bands
        );
    }

    #[test]
    fn test_band_values_in_valid_range() {
        let mut eq = EqState::new(48000);
        let signal: Vec<f32> = (0..FFT_SIZE).map(|_| 1.0f32).collect();
        let result = eq.feed(&signal);
        assert!(result.is_some());
        let bands = result.unwrap();
        assert_eq!(bands.len(), EQ_BAND_COUNT);
        for (i, &val) in bands.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&val),
                "Band {} value {} is outside valid range [0, 1]",
                i,
                val
            );
        }
    }
}
