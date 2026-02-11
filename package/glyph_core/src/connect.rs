use crate::types::Stroke;

/// A pure function that determines the connection point for a stroke.
///
/// Given:
/// - `current`: The stroke being built (before connection translation)
/// - `previous`: A list of previously built strokes (e.g., last 2 strokes, or empty)
///
/// Returns:
/// - `Some(point)`: A point on one of the previous strokes where current should begin
/// - `None`: If no valid connection point exists (e.g., empty previous list)
///
/// Must be pure: same inputs always produce same output.
pub type ConnectFn = dyn Fn(&Stroke, &[Stroke]) -> Option<[f32; 3]> + Send + Sync;

/// Deterministically connects to a "random" point from the previous 0-3 strokes.
///
/// This creates a more organic, branching glyph appearance by varying which
/// previous stroke and which point within it to connect to.
///
/// - Uses hash of current stroke to deterministically select from up to 4 previous strokes
/// - If previous is empty, returns None (first stroke)
/// - If fewer than 4 strokes exist, uses all available strokes
pub fn natural_connect(current: &Stroke, previous: &[Stroke]) -> Option<[f32; 3]> {
    if previous.is_empty() {
        return None;
    }

    // Deterministic hash of current stroke
    let hash = current.points.iter().fold(0u64, |acc, p| {
        acc.wrapping_mul(31)
            .wrapping_add(p[0].to_bits() as u64)
            .wrapping_add(p[1].to_bits() as u64)
            .wrapping_add(p[2].to_bits() as u64)
    });

    // Use first few bits to select which stroke (0-3 strokes back)
    let stroke_idx = (hash as usize) % previous.len().min(4);
    let selected_stroke = &previous[previous.len() - 1 - stroke_idx];

    if selected_stroke.points.is_empty() {
        // Fallback to most recent stroke if selected has no points
        return previous.last()?.points.last().copied();
    }

    // Use remaining bits to select point within stroke
    let point_idx = ((hash >> 2) as usize) % selected_stroke.points.len();

    selected_stroke.points.get(point_idx).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stroke(points: Vec<[f32; 3]>) -> Stroke {
        Stroke {
            points,
            colour: [0.5, 0.5, 0.5],
        }
    }

    #[test]
    fn natural_connect_empty_returns_none() {
        let current = make_stroke(vec![[0.0, 0.0, 0.0]]);
        let previous: Vec<Stroke> = vec![];
        let point = natural_connect(&current, &previous);

        assert_eq!(point, None);
    }

    #[test]
    fn natural_connect_is_pure() {
        let current = make_stroke(vec![[1.0, 2.0, 3.0]]);
        let previous = vec![
            make_stroke(vec![[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]),
            make_stroke(vec![[13.0, 14.0, 15.0], [16.0, 17.0, 18.0]]),
            make_stroke(vec![[19.0, 20.0, 21.0], [22.0, 23.0, 24.0]]),
            make_stroke(vec![[25.0, 26.0, 27.0], [28.0, 29.0, 30.0]]),
        ];
        let a = natural_connect(&current, &previous);
        let b = natural_connect(&current, &previous);

        assert_eq!(a, b);
    }

    #[test]
    fn natural_connect_returns_valid_point() {
        // Build a set of all possible points from previous strokes
        let all_points: Vec<[f32; 3]> = vec![
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
            [4.0, 4.0, 4.0],
            [5.0, 5.0, 5.0],
            [6.0, 6.0, 6.0],
            [7.0, 7.0, 7.0],
            [8.0, 8.0, 8.0],
        ];

        let previous = vec![
            make_stroke(vec![all_points[0], all_points[1]]),
            make_stroke(vec![all_points[2], all_points[3]]),
            make_stroke(vec![all_points[4], all_points[5]]),
            make_stroke(vec![all_points[6], all_points[7]]),
        ];

        // Test with different current strokes to ensure variety
        let mut found_points = std::collections::HashSet::new();

        for i in 0..20 {
            let current = make_stroke(vec![[i as f32, (i * 2) as f32, (i * 3) as f32]]);

            if let Some(point) = natural_connect(&current, &previous) {
                found_points.insert(point);

                assert!(
                    all_points.contains(&point),
                    "Point {:?} should be from previous strokes",
                    point
                );
            }
        }

        // Should find multiple different connection points
        assert!(
            found_points.len() > 1,
            "natural_connect should produce varied connection points, found: {:?}",
            found_points
        );
    }

    #[test]
    fn natural_connect_uses_different_strokes() {
        // Create strokes with distinct points to verify we can connect to different strokes
        let previous = vec![
            make_stroke(vec![[1.0, 0.0, 0.0]]), // stroke 0
            make_stroke(vec![[2.0, 0.0, 0.0]]), // stroke 1
            make_stroke(vec![[3.0, 0.0, 0.0]]), // stroke 2
            make_stroke(vec![[4.0, 0.0, 0.0]]), // stroke 3
        ];

        // Test multiple current strokes to see variety
        let mut found_strokes = std::collections::HashSet::new();

        for i in 0..100 {
            let current = make_stroke(vec![[i as f32 * 0.1, i as f32 * 0.2, i as f32 * 0.3]]);

            if let Some(point) = natural_connect(&current, &previous) {
                found_strokes.insert(point[0] as i32); // x coordinate identifies the stroke
            }
        }

        // Should connect to multiple different strokes
        assert!(
            found_strokes.len() >= 2,
            "Should connect to at least 2 different strokes, found: {:?}",
            found_strokes
        );
    }
}
