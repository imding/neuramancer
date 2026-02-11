/// Configuration for 3D position computation.
#[derive(Clone, Debug)]
pub struct PositionConfig {
    /// Random seed for PaCMAP determinism.
    pub seed: u64,
    /// Minimum distance between any two entity positions. Glyph bounding
    /// radius should be smaller than half of this value to avoid overlap.
    pub min_distance: f32,
    /// Number of iterative repulsion passes after PaCMAP.
    pub repulsion_iterations: usize,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            min_distance: 2.0,
            repulsion_iterations: 50,
        }
    }
}

/// Compute 3D positions from high-dimensional embeddings using PaCMAP.
///
/// Requires the `pacmap` feature. Without it, this function falls back to a
/// simple grid layout.
///
/// Steps:
/// 1. Run PaCMAP to reduce each embedding to 3D.
/// 2. Apply iterative repulsion to ensure no two positions are closer than
///    `config.min_distance`.
pub fn compute_positions(embeddings: &[Vec<f32>], config: &PositionConfig) -> Vec<[f32; 3]> {
    if embeddings.is_empty() {
        return Vec::new();
    }

    let mut positions = reduce_to_3d(embeddings, config);

    apply_repulsion(
        &mut positions,
        config.min_distance,
        config.repulsion_iterations,
    );

    positions
}

#[cfg(feature = "pacmap")]
fn reduce_to_3d(embeddings: &[Vec<f32>], config: &PositionConfig) -> Vec<[f32; 3]> {
    use ndarray::Array2;

    let n = embeddings.len();

    if n == 0 {
        return Vec::new();
    }

    // Find the target dimension: the most common non-zero length.
    let target_dim = {
        let mut counts = std::collections::HashMap::new();

        for emb in embeddings {
            if !emb.is_empty() {
                *counts.entry(emb.len()).or_insert(0usize) += 1;
            }
        }

        counts.into_iter().max_by_key(|&(_, c)| c).map(|(d, _)| d)
    };

    let Some(d) = target_dim
    else {
        // All embeddings are empty — fall back to grid.
        return fallback_grid(n);
    };

    // Collect indices of valid embeddings (matching target dimension).
    let valid_indices: Vec<usize> = embeddings
        .iter()
        .enumerate()
        .filter(|(_, e)| e.len() == d)
        .map(|(i, _)| i)
        .collect();

    let valid_count = valid_indices.len();

    if valid_count < 2 {
        // Not enough points for PaCMAP — fall back to grid.
        return fallback_grid(n);
    }

    let mut data = Array2::<f32>::zeros((valid_count, d));

    for (row, &orig_idx) in valid_indices.iter().enumerate() {
        for (j, &val) in embeddings[orig_idx].iter().enumerate() {
            data[[row, j]] = val;
        }
    }

    let pacmap_config = pacmap::Configuration::builder()
        .embedding_dimensions(3)
        .initialization(pacmap::Initialization::Random(Some(config.seed)))
        .learning_rate(1.0)
        .num_iters((100, 100, 250))
        .build();

    let reduced = match pacmap::fit_transform(data.view(), pacmap_config) {
        Ok((embedding, _)) => (0..valid_count)
            .map(|i| [embedding[[i, 0]], embedding[[i, 1]], embedding[[i, 2]]])
            .collect::<Vec<_>>(),
        Err(_) => fallback_grid(valid_count),
    };

    // Map back to original indices. Invalid embeddings get grid positions.
    let grid = fallback_grid(n);
    let mut positions = grid;

    for (row, &orig_idx) in valid_indices.iter().enumerate() {
        positions[orig_idx] = reduced[row];
    }

    positions
}

#[cfg(not(feature = "pacmap"))]
fn reduce_to_3d(embeddings: &[Vec<f32>], _config: &PositionConfig) -> Vec<[f32; 3]> {
    fallback_grid(embeddings.len())
}

fn fallback_grid(n: usize) -> Vec<[f32; 3]> {
    let per_row = (n as f32).sqrt().ceil().max(1.0) as usize;
    let spacing = 2.5;

    (0..n)
        .map(|i| {
            let row = i / per_row;
            let col = i % per_row;
            let x = col as f32 * spacing - (per_row as f32 - 1.0) * spacing * 0.5;
            let z = row as f32 * spacing;
            [x, 0.8, z]
        })
        .collect()
}

/// Iterative repulsion to ensure minimum distance between all pairs.
///
/// For each pair closer than `min_distance`, push them apart along their
/// connecting vector. Deterministic: pairs are processed in index order.
fn apply_repulsion(positions: &mut [[f32; 3]], min_distance: f32, iterations: usize) {
    let n = positions.len();

    if n < 2 {
        return;
    }

    let min_dist_sq = min_distance * min_distance;

    for _ in 0..iterations {
        let mut any_overlap = false;

        for i in 0..n {
            for j in (i + 1)..n {
                let dx = positions[j][0] - positions[i][0];
                let dy = positions[j][1] - positions[i][1];
                let dz = positions[j][2] - positions[i][2];
                let dist_sq = dx * dx + dy * dy + dz * dz;

                if dist_sq < min_dist_sq && dist_sq > 1e-10 {
                    any_overlap = true;

                    let dist = dist_sq.sqrt();
                    let push = (min_distance - dist) * 0.5;
                    let nx = dx / dist * push;
                    let ny = dy / dist * push;
                    let nz = dz / dist * push;

                    positions[i][0] -= nx;
                    positions[i][1] -= ny;
                    positions[i][2] -= nz;
                    positions[j][0] += nx;
                    positions[j][1] += ny;
                    positions[j][2] += nz;
                }
            }
        }

        if !any_overlap {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_embeddings() {
        let positions = compute_positions(&[], &PositionConfig::default());

        assert!(positions.is_empty());
    }

    #[test]
    fn single_embedding() {
        let embeddings = vec![vec![0.0; 384]];
        let positions = compute_positions(&embeddings, &PositionConfig::default());

        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn repulsion_enforces_min_distance() {
        let mut positions = vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [5.0, 0.0, 0.0]];

        apply_repulsion(&mut positions, 2.0, 100);

        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let dx = positions[j][0] - positions[i][0];
                let dy = positions[j][1] - positions[i][1];
                let dz = positions[j][2] - positions[i][2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                assert!(dist >= 1.99, "positions {} and {} too close: {dist}", i, j);
            }
        }
    }

    #[test]
    fn repulsion_deterministic() {
        let mut a = vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [0.0, 0.5, 0.0]];
        let mut b = a.clone();

        apply_repulsion(&mut a, 2.0, 50);
        apply_repulsion(&mut b, 2.0, 50);

        assert_eq!(a, b);
    }

    #[test]
    fn fallback_grid_layout() {
        let positions = fallback_grid(9);

        assert_eq!(positions.len(), 9);

        // 9 points → 3x3 grid
        // No two should be at the same position
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                assert_ne!(positions[i], positions[j]);
            }
        }
    }
}
