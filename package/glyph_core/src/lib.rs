pub mod connect;
pub mod glyph;
pub mod normalise;
pub mod position;
pub mod slice;
pub mod stroke;
pub mod types;

pub use {
    connect::{ConnectFn, natural_connect},
    glyph::build_glyph,
    normalise::{NormaliseStrategy, PercentileNormalise, TanhNormalise},
    position::{PositionConfig, compute_positions},
    slice::{SliceStrategy, UniformSlice},
    stroke::{DefaultStroke, StrokeStrategy},
    types::{Glyph, Stroke, VisualLayout},
};

/// A placeholder diamond glyph for nodes without embeddings (e.g. pending
/// optimistic creates). Four strokes forming a diamond outline in the XY
/// plane, coloured neutral grey.
pub fn placeholder_glyph(scale: f32) -> Glyph {
    let top = [0.0, scale, 0.0];
    let right = [scale, 0.0, 0.0];
    let bottom = [0.0, -scale, 0.0];
    let left = [-scale, 0.0, 0.0];
    let grey = [0.6, 0.6, 0.6];

    Glyph {
        strokes: vec![
            Stroke {
                points: vec![top, right],
                colour: grey,
            },
            Stroke {
                points: vec![right, bottom],
                colour: grey,
            },
            Stroke {
                points: vec![bottom, left],
                colour: grey,
            },
            Stroke {
                points: vec![left, top],
                colour: grey,
            },
        ],
    }
}

/// Build a single glyph from an embedding.
///
/// If the embedding is empty, returns a placeholder diamond.
/// Otherwise, returns a deterministic glyph derived from the embedding.
pub fn glyph_for_embedding(embedding: &[f32], _glyph_scale: f32) -> Glyph {
    if embedding.is_empty() {
        return placeholder_glyph(1.5);
    }

    let slicer = UniformSlice::default();
    let normaliser = TanhNormalise::default();
    let stroke_builder = DefaultStroke;

    build_glyph(
        embedding,
        &slicer,
        &normaliser,
        &stroke_builder,
        &natural_connect,
        4,
    )
}

/// Build visual layouts for a batch of embeddings using default strategies.
///
/// This is the primary entry point. For each embedding it computes a 3D
/// position (via PaCMAP when the `pacmap` feature is enabled, otherwise a
/// grid fallback) and a deterministic glyph.
pub fn compute_visual_layouts(
    embeddings: &[Vec<f32>],
    position_config: &PositionConfig,
) -> Vec<VisualLayout> {
    let positions = compute_positions(embeddings, position_config);

    let slicer = UniformSlice::default();
    let normaliser = TanhNormalise::default();
    let stroke_builder = DefaultStroke;

    positions
        .into_iter()
        .zip(embeddings.iter())
        .map(|(position, embedding)| VisualLayout {
            position,
            glyph: build_glyph(
                embedding,
                &slicer,
                &normaliser,
                &stroke_builder,
                &natural_connect,
                4,
            ),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_embeddings(count: usize, dim: usize) -> Vec<Vec<f32>> {
        (0..count)
            .map(|i| {
                (0..dim)
                    .map(|j| ((i * dim + j) as f32 * 0.37).sin())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn placeholder_diamond_has_four_strokes() {
        let glyph = placeholder_glyph(0.5);

        assert_eq!(glyph.strokes.len(), 4);

        for stroke in &glyph.strokes {
            assert_eq!(stroke.points.len(), 2);
            assert_eq!(stroke.colour, [0.6, 0.6, 0.6]);
        }
    }

    #[test]
    fn glyph_for_empty_embedding_is_diamond() {
        let glyph = glyph_for_embedding(&[], 0.5);

        assert_eq!(glyph.strokes.len(), 4); // diamond
    }

    #[test]
    fn glyph_for_real_embedding_is_not_diamond() {
        let embedding: Vec<f32> = (0..768).map(|i| (i as f32 * 0.1).sin()).collect();
        let glyph = glyph_for_embedding(&embedding, 0.5);

        assert_eq!(glyph.strokes.len(), 64); // 768 / 12
    }

    #[test]
    fn visual_layouts_deterministic() {
        let embeddings = make_embeddings(5, 768);
        let config = PositionConfig::default();
        let a = compute_visual_layouts(&embeddings, &config);
        let b = compute_visual_layouts(&embeddings, &config);

        assert_eq!(a, b);
    }

    #[test]
    fn visual_layouts_correct_count() {
        let embeddings = make_embeddings(3, 768);
        let layouts = compute_visual_layouts(&embeddings, &PositionConfig::default());

        assert_eq!(layouts.len(), 3);
    }

    #[test]
    fn visual_layouts_empty() {
        let layouts = compute_visual_layouts(&[], &PositionConfig::default());

        assert!(layouts.is_empty());
    }

    #[test]
    fn each_layout_has_glyph() {
        let embeddings = make_embeddings(2, 768);
        let layouts = compute_visual_layouts(&embeddings, &PositionConfig::default());

        for layout in &layouts {
            assert!(!layout.glyph.strokes.is_empty());
            assert_eq!(layout.glyph.strokes.len(), 64); // 768 / 12
        }
    }
}
