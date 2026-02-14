use std::ops::Range;

/// Strategy for partitioning an embedding into sub-array ranges.
///
/// Each range maps to one stroke. The strategy is a pure function of the
/// embedding, so the same embedding always produces the same partition.
pub trait SliceStrategy: Send + Sync {
    fn slice(&self, embedding: &[f32]) -> Vec<Range<usize>>;
}

/// Uniform slicing: every stroke gets exactly `dims_per_stroke` values.
///
/// Leftover dimensions (if `embedding.len()` is not divisible) are silently
/// dropped. For 768D with `dims_per_stroke = 12`: 64 strokes, 0 leftover.
pub struct UniformSlice {
    pub dims_per_stroke: usize,
}

impl Default for UniformSlice {
    fn default() -> Self {
        Self {
            dims_per_stroke: 12,
        }
    }
}

impl SliceStrategy for UniformSlice {
    fn slice(&self, embedding: &[f32]) -> Vec<Range<usize>> {
        let n = self.dims_per_stroke;

        if n == 0 {
            return Vec::new();
        }

        let count = embedding.len() / n;

        (0..count).map(|i| (i * n)..((i + 1) * n)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_768_by_12() {
        let slicer = UniformSlice {
            dims_per_stroke: 12,
        };
        let embedding = vec![0.0; 768];
        let slices = slicer.slice(&embedding);

        assert_eq!(slices.len(), 64);
        assert_eq!(slices[0], 0..12);
        assert_eq!(slices[63], 756..768);
    }

    #[test]
    fn uniform_768_by_9() {
        let slicer = UniformSlice { dims_per_stroke: 9 };
        let embedding = vec![0.0; 768];
        let slices = slicer.slice(&embedding);

        assert_eq!(slices.len(), 85);
        // 85 * 9 = 765, so 3 dims are dropped
    }

    #[test]
    fn uniform_empty_embedding() {
        let slicer = UniformSlice::default();
        let slices = slicer.slice(&[]);

        assert!(slices.is_empty());
    }

    #[test]
    fn uniform_zero_dims_per_stroke() {
        let slicer = UniformSlice { dims_per_stroke: 0 };
        let slices = slicer.slice(&[1.0, 2.0, 3.0]);

        assert!(slices.is_empty());
    }

    #[test]
    fn uniform_smaller_than_one_stroke() {
        let slicer = UniformSlice {
            dims_per_stroke: 12,
        };
        let slices = slicer.slice(&[1.0; 11]);

        assert!(slices.is_empty());
    }
}
