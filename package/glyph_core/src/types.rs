/// A single stroke in a glyph: a sequence of 3D control points with a colour.
///
/// For a 12-dim embedding slice: 3 points (curve) + RGB colour.
/// For a 9-dim embedding slice: 2 points (line segment) + RGB colour.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    /// Control points in local space (relative to entity position).
    pub points: Vec<[f32; 3]>,
    /// RGB colour, each channel in `[0, 1]`.
    pub colour: [f32; 3],
}

/// A complete glyph: an ordered chain of connected strokes.
///
/// Stroke 0 starts at local origin. Each subsequent stroke starts from the
/// connection point determined by the [`ConnectStrategy`](super::connect::ConnectStrategy).
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub strokes: Vec<Stroke>,
}

/// Position + glyph for one entity in the scene.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualLayout {
    pub position: [f32; 3],
    pub glyph: Glyph,
}
