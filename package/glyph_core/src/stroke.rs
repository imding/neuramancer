use crate::types::Stroke;

/// Strategy for interpreting a normalised slice as a [`Stroke`].
///
/// The input is a normalised slice of embedding values (all in some bounded
/// range, typically `(-1, 1)`). The strategy must produce a `Stroke` with
/// control points and an RGB colour.
pub trait StrokeStrategy: Send + Sync {
    fn build_stroke(&self, normalised: &[f32]) -> Stroke;
}

/// Default stroke interpretation:
/// - Last 3 values → RGB colour (remapped from `(-1,1)` to `[0,1]`).
/// - Every 3 preceding values → a 3D control point.
///
/// For a 12-value slice: 3 points + RGB.
/// For a 9-value slice: 2 points + RGB.
/// For a 6-value slice: 1 point + RGB.
///
/// If there are fewer than 6 values, returns a stroke with no points.
pub struct DefaultStroke;

impl StrokeStrategy for DefaultStroke {
    fn build_stroke(&self, normalised: &[f32]) -> Stroke {
        let len = normalised.len();
        if len < 6 {
            return Stroke {
                points: Vec::new(),
                colour: if len >= 3 {
                    let r = (normalised[len - 3] + 1.0) * 0.5;
                    let g = (normalised[len - 2] + 1.0) * 0.5;
                    let b = (normalised[len - 1] + 1.0) * 0.5;

                    [r, g, b]
                }
                else {
                    [0.5, 0.5, 0.5]
                },
            };
        }

        // Last 3 values → colour (remap from (-1,1) to [0,1])
        let r = (normalised[len - 3] + 1.0) * 0.5;
        let g = (normalised[len - 2] + 1.0) * 0.5;
        let b = (normalised[len - 1] + 1.0) * 0.5;
        let colour = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];

        // Preceding values → 3D points
        let point_values = &normalised[..len - 3];
        let num_points = point_values.len() / 3;
        let points: Vec<[f32; 3]> = (0..num_points)
            .map(|i| {
                let base = i * 3;
                [
                    point_values[base],
                    point_values[base + 1],
                    point_values[base + 2],
                ]
            })
            .collect();

        Stroke { points, colour }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_dim_gives_three_points_and_colour() {
        let builder = DefaultStroke;
        let input = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, -0.5, 0.0, 0.5];
        let stroke = builder.build_stroke(&input);

        assert_eq!(stroke.points.len(), 3);
        assert_eq!(stroke.points[0], [0.1, 0.2, 0.3]);
        assert_eq!(stroke.points[1], [0.4, 0.5, 0.6]);
        assert_eq!(stroke.points[2], [0.7, 0.8, 0.9]);

        // Color: (-0.5 + 1) * 0.5 = 0.25, (0 + 1) * 0.5 = 0.5, (0.5 + 1) * 0.5 = 0.75
        assert!((stroke.colour[0] - 0.25).abs() < 1e-6);
        assert!((stroke.colour[1] - 0.5).abs() < 1e-6);
        assert!((stroke.colour[2] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn nine_dim_gives_two_points() {
        let builder = DefaultStroke;
        let input = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.0, 0.0, 0.0];
        let stroke = builder.build_stroke(&input);

        assert_eq!(stroke.points.len(), 2);
        assert_eq!(stroke.points[0], [0.1, 0.2, 0.3]);
        assert_eq!(stroke.points[1], [0.4, 0.5, 0.6]);
    }

    #[test]
    fn six_dim_gives_one_point() {
        let builder = DefaultStroke;
        let input = [0.1, 0.2, 0.3, 0.0, 0.0, 0.0];
        let stroke = builder.build_stroke(&input);

        assert_eq!(stroke.points.len(), 1);
        assert_eq!(stroke.points[0], [0.1, 0.2, 0.3]);
    }

    #[test]
    fn fewer_than_six_no_points() {
        let builder = DefaultStroke;
        let stroke = builder.build_stroke(&[0.0, 0.0, 0.0]);
        assert!(stroke.points.is_empty());
    }

    #[test]
    fn colour_clamped() {
        let builder = DefaultStroke;
        // Values well outside (-1,1) after normalisation should still clamp
        let input = [0.0, 0.0, 0.0, 2.0, -2.0, 0.0];
        let stroke = builder.build_stroke(&input);
        assert!(stroke.colour[0] >= 0.0 && stroke.colour[0] <= 1.0);
        assert!(stroke.colour[1] >= 0.0 && stroke.colour[1] <= 1.0);
        assert!(stroke.colour[2] >= 0.0 && stroke.colour[2] <= 1.0);
    }
}
