use crate::{
    connect::ConnectFn, normalise::NormaliseStrategy, slice::SliceStrategy, stroke::StrokeStrategy,
    types::Glyph,
};

/// Build a glyph from a raw embedding using the provided strategies.
///
/// This is the core pure function: same embedding + same strategies = same
/// glyph, always.
///
/// Pipeline:
/// 1. Slice the embedding into sub-array ranges.
/// 2. For each range: normalise the values, then build a stroke.
/// 3. For strokes after the first: translate all points so the stroke's first
///    point matches the connection point from the previous stroke.
/// 4. Return the assembled glyph.
pub fn build_glyph(
    embedding: &[f32],
    slicer: &dyn SliceStrategy,
    normaliser: &dyn NormaliseStrategy,
    stroke_builder: &dyn StrokeStrategy,
    connector: &ConnectFn,
    previous_stroke_count: usize,
) -> Glyph {
    let ranges = slicer.slice(embedding);
    let mut strokes = Vec::with_capacity(ranges.len());

    for range in ranges.into_iter() {
        let raw = &embedding[range];
        let normalised = normaliser.normalise(raw);
        let mut stroke = stroke_builder.build_stroke(&normalised);

        // Get connection point from connector function
        let window_start = strokes.len().saturating_sub(previous_stroke_count);
        let previous_window = &strokes[window_start..];

        if let Some(anchor) = connector(&stroke, previous_window)
            && !stroke.points.is_empty() {
                let offset = [
                    anchor[0] - stroke.points[0][0],
                    anchor[1] - stroke.points[0][1],
                    anchor[2] - stroke.points[0][2],
                ];

                for point in &mut stroke.points {
                    point[0] += offset[0];
                    point[1] += offset[1];
                    point[2] += offset[2];
                }
            }

        strokes.push(stroke);
    }

    Glyph { strokes }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            connect::natural_connect, normalise::TanhNormalise, slice::UniformSlice,
            stroke::DefaultStroke,
        },
    };

    fn make_embedding(len: usize) -> Vec<f32> {
        // Deterministic pseudo-random values
        (0..len).map(|i| ((i as f32) * 0.37).sin()).collect()
    }

    #[test]
    fn deterministic() {
        let embedding = make_embedding(768);
        let slicer = UniformSlice {
            dims_per_stroke: 12,
        };
        let normaliser = TanhNormalise::default();
        let stroke_builder = DefaultStroke;

        let a = build_glyph(
            &embedding,
            &slicer,
            &normaliser,
            &stroke_builder,
            &natural_connect,
            1,
        );
        let b = build_glyph(
            &embedding,
            &slicer,
            &normaliser,
            &stroke_builder,
            &natural_connect,
            1,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn correct_stroke_count() {
        let embedding = make_embedding(768);
        let slicer = UniformSlice {
            dims_per_stroke: 12,
        };
        let normaliser = TanhNormalise::default();
        let glyph = build_glyph(
            &embedding,
            &slicer,
            &normaliser,
            &DefaultStroke,
            &natural_connect,
            1,
        );
        assert_eq!(glyph.strokes.len(), 64);
    }

    #[test]
    fn strokes_connected() {
        let embedding = make_embedding(768);
        let slicer = UniformSlice {
            dims_per_stroke: 12,
        };
        let normaliser = TanhNormalise::default();
        let glyph = build_glyph(
            &embedding,
            &slicer,
            &normaliser,
            &DefaultStroke,
            &natural_connect,
            4,
        );

        // With natural_connect, each stroke's first point should match some point
        // from the previous 0-3 strokes, creating a connected structure
        for i in 1..glyph.strokes.len() {
            let curr_first = &glyph.strokes[i].points[0];
            // Check that the first point matches one of the available connection points
            let mut connected = false;

            for j in 0..i.min(4) {
                let prev_stroke = &glyph.strokes[i - 1 - j];

                for point in &prev_stroke.points {
                    if (point[0] - curr_first[0]).abs() < 1e-5 &&
                        (point[1] - curr_first[1]).abs() < 1e-5 &&
                        (point[2] - curr_first[2]).abs() < 1e-5
                    {
                        connected = true;
                        break;
                    }
                }

                if connected {
                    break;
                }
            }

            assert!(
                connected,
                "stroke {} first point {:?} should connect to a point in previous 4 strokes",
                i, curr_first
            );
        }
    }

    #[test]
    fn all_colours_bounded() {
        let embedding = make_embedding(768);
        let slicer = UniformSlice {
            dims_per_stroke: 12,
        };
        let normaliser = TanhNormalise::default();
        let glyph = build_glyph(
            &embedding,
            &slicer,
            &normaliser,
            &DefaultStroke,
            &natural_connect,
            1,
        );

        for stroke in &glyph.strokes {
            for &c in &stroke.colour {
                assert!(c >= 0.0 && c <= 1.0, "colour out of bounds: {c}");
            }
        }
    }

    #[test]
    fn empty_embedding() {
        let glyph = build_glyph(
            &[],
            &UniformSlice::default(),
            &TanhNormalise::default(),
            &DefaultStroke,
            &natural_connect,
            1,
        );

        assert!(glyph.strokes.is_empty());
    }

    #[test]
    fn different_embeddings_different_glyphs() {
        let a = make_embedding(768);
        let mut b = make_embedding(768);

        b[0] += 1.0; // perturb one value

        let slicer = UniformSlice {
            dims_per_stroke: 12,
        };
        let normaliser = TanhNormalise::default();
        let ga = build_glyph(
            &a,
            &slicer,
            &normaliser,
            &DefaultStroke,
            &natural_connect,
            1,
        );
        let gb = build_glyph(
            &b,
            &slicer,
            &normaliser,
            &DefaultStroke,
            &natural_connect,
            1,
        );

        assert_ne!(ga, gb);
    }
}
