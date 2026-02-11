/// Strategy for normalising raw embedding values to bounded glyph-space values.
///
/// Pure function: same input → same output. No global state.
pub trait NormaliseStrategy: Send + Sync {
    fn normalise(&self, values: &[f32]) -> Vec<f32>;
}

/// Applies `tanh(v * scale)` to each value.
///
/// Produces values in `(-1, 1)`. Self-contained (no global stats needed),
/// outlier-resistant, smooth, deterministic.
///
/// `scale` controls the spread: higher values push towards the tanh
/// asymptotes (±1), lower values keep output near zero.
pub struct TanhNormalise {
    pub scale: f32,
}

impl Default for TanhNormalise {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl NormaliseStrategy for TanhNormalise {
    fn normalise(&self, values: &[f32]) -> Vec<f32> {
        values.iter().map(|&v| (v * self.scale).tanh()).collect()
    }
}

/// Percentile-based normalisation: maps each value to its percentile rank
/// within the slice, then remaps from `[0, 1]` to `(-1, 1)`.
///
/// Robust to outliers and guarantees the full output range is used regardless
/// of input distribution. Deterministic: same input → same output.
///
/// Tied values receive the same output (mean of the ranks they span).
/// A single-element slice maps to `0.0` (midpoint).
pub struct PercentileNormalise;

impl NormaliseStrategy for PercentileNormalise {
    fn normalise(&self, values: &[f32]) -> Vec<f32> {
        let n = values.len();

        if n == 0 {
            return Vec::new();
        }

        if n == 1 {
            return vec![0.0];
        }

        // Build (value, original_index) pairs and sort by value.
        let mut indexed: Vec<(f32, usize)> = values
            .iter()
            .copied()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();

        indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Assign ranks, averaging across ties.
        let mut ranks = vec![0.0_f32; n];
        let mut i = 0;

        while i < n {
            let mut j = i;

            // Find the run of equal values.
            while j < n && indexed[j].0 == indexed[i].0 {
                j += 1;
            }

            // Average rank for the tied group (ranks are 0-based).
            let avg_rank = (i + j - 1) as f32 / 2.0;

            for item in &indexed[i..j] {
                ranks[item.1] = avg_rank;
            }

            i = j;
        }

        // Remap rank from [0, n-1] to (-1, 1).
        let max_rank = (n - 1) as f32;

        ranks
            .into_iter()
            .map(|r| r / max_rank * 2.0 - 1.0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TanhNormalise ──────────────────────────────────────────────

    #[test]
    fn tanh_zero_maps_to_zero() {
        let norm = TanhNormalise::default();
        let result = norm.normalise(&[0.0]);

        assert!((result[0]).abs() < 1e-6);
    }

    #[test]
    fn tanh_bounded() {
        let norm = TanhNormalise { scale: 10.0 };
        let result = norm.normalise(&[-100.0, 100.0, 0.5, -0.5]);

        for v in &result {
            assert!(*v >= -1.0 && *v <= 1.0, "value out of tanh range: {v}");
        }
    }

    #[test]
    fn tanh_deterministic() {
        let norm = TanhNormalise { scale: 0.3 };
        let input = vec![1.5, -2.0, 0.7, 3.14];
        let a = norm.normalise(&input);
        let b = norm.normalise(&input);

        assert_eq!(a, b);
    }

    #[test]
    fn tanh_empty() {
        let norm = TanhNormalise::default();
        let result = norm.normalise(&[]);

        assert!(result.is_empty());
    }

    #[test]
    fn tanh_preserves_sign() {
        let norm = TanhNormalise::default();
        let result = norm.normalise(&[-2.0, 2.0]);

        assert!(result[0] < 0.0);
        assert!(result[1] > 0.0);
    }

    // ── PercentileNormalise ────────────────────────────────────────

    #[test]
    fn percentile_empty() {
        let norm = PercentileNormalise;
        let result = norm.normalise(&[]);

        assert!(result.is_empty());
    }

    #[test]
    fn percentile_single() {
        let norm = PercentileNormalise;
        let result = norm.normalise(&[42.0]);

        assert!((result[0]).abs() < 1e-6, "single value should map to 0.0");
    }

    #[test]
    fn percentile_two_values() {
        let norm = PercentileNormalise;
        let result = norm.normalise(&[10.0, 20.0]);

        assert!((result[0] - (-1.0)).abs() < 1e-6, "smallest → -1");
        assert!((result[1] - 1.0).abs() < 1e-6, "largest → +1");
    }

    #[test]
    fn percentile_sorted_ascending() {
        let norm = PercentileNormalise;
        let result = norm.normalise(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        // Ranks: 0,1,2,3,4 → mapped to -1, -0.5, 0, 0.5, 1
        assert!((result[0] - (-1.0)).abs() < 1e-6);
        assert!((result[1] - (-0.5)).abs() < 1e-6);
        assert!((result[2] - 0.0).abs() < 1e-6);
        assert!((result[3] - 0.5).abs() < 1e-6);
        assert!((result[4] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn percentile_preserves_order() {
        let norm = PercentileNormalise;
        let input = vec![
            0.05, -0.03, 0.01, -0.08, 0.1, -0.01, 0.02, 0.07, -0.05, 0.0, -0.02, 0.03,
        ];
        let result = norm.normalise(&input);

        // Min value (-0.08 at index 3) should map to -1.
        assert!((result[3] - (-1.0)).abs() < 1e-6);
        // Max value (0.1 at index 4) should map to +1.
        assert!((result[4] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn percentile_bounded() {
        let norm = PercentileNormalise;
        let input: Vec<f32> = (0..12).map(|i| (i as f32 * 0.37).sin()).collect();
        let result = norm.normalise(&input);

        for v in &result {
            assert!(*v >= -1.0 && *v <= 1.0, "value out of range: {v}");
        }
    }

    #[test]
    fn percentile_ties_get_same_rank() {
        let norm = PercentileNormalise;
        // Two identical values should get the same output.
        let result = norm.normalise(&[1.0, 3.0, 3.0, 5.0]);

        assert!(
            (result[1] - result[2]).abs() < 1e-6,
            "tied values must be equal"
        );
    }

    #[test]
    fn percentile_all_same() {
        let norm = PercentileNormalise;
        let result = norm.normalise(&[7.0, 7.0, 7.0, 7.0]);

        // All tied → average rank = 1.5 out of max_rank 3 → 1.5/3*2-1 = 0.0
        for v in &result {
            assert!(
                (v - 0.0).abs() < 1e-6,
                "all-same should map to 0.0, got {v}"
            );
        }
    }

    #[test]
    fn percentile_deterministic() {
        let norm = PercentileNormalise;
        let input = vec![
            0.05, -0.03, 0.01, -0.08, 0.1, -0.01, 0.02, 0.07, -0.05, 0.0, -0.02, 0.03,
        ];
        let a = norm.normalise(&input);
        let b = norm.normalise(&input);

        assert_eq!(a, b);
    }

    #[test]
    fn percentile_spreads_narrow_values() {
        // This is the key test: narrow values (like SigLIP embeddings) should
        // still span the full (-1, 1) range after percentile normalisation.
        let norm = PercentileNormalise;
        let input = vec![
            0.05, -0.03, 0.01, -0.08, 0.1, -0.01, 0.02, 0.07, -0.05, 0.0, -0.02, 0.03,
        ];
        let result = norm.normalise(&input);

        let min = result.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = result.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!((min - (-1.0)).abs() < 1e-6, "min should be -1.0, got {min}");
        assert!((max - 1.0).abs() < 1e-6, "max should be 1.0, got {max}");
    }
}
