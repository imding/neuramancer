use {
    crate::{EdgeKind, GraphEdgeInput, GraphNodeInput, MeshKind, NodeKind},
    glyph_core::{PositionConfig, compute_positions, glyph_for_embedding},
};

const GLYPH_SCALE: f32 = 1.5;

#[derive(Clone, Debug, PartialEq)]
pub struct NoteInput {
    pub id: String,
    pub snippet_count: usize,
    /// Embedding vectors from all snippets in this note.
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnippetInput {
    pub note_id: String,
    pub snippet_count: usize,
    /// Embedding vectors for snippets belonging to this note.
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KnotInput {
    pub id: String,
    pub label: Option<String>,
    pub child_knot_ids: Vec<String>,
    pub note_ids: Vec<String>,
}

pub fn build_note_nodes(note_inputs: &[NoteInput]) -> Vec<GraphNodeInput> {
    // Collect the first embedding from each note (if available) for layout.
    let embeddings: Vec<Vec<f32>> = note_inputs
        .iter()
        .map(|input| input.embeddings.first().cloned().unwrap_or_default())
        .collect();

    // Positions: PaCMAP when possible, grid fallback otherwise.
    let positions = compute_positions(&embeddings, &PositionConfig::default());

    note_inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let embedding = &embeddings[index];

            // Position: from PaCMAP/grid fallback (always available).
            let position = positions
                .get(index)
                .copied()
                .unwrap_or_else(|| grid_position(index, 8, 2.2, 0.8));

            // Glyph: real glyph from embedding, or placeholder diamond.
            let glyph = glyph_for_embedding(embedding, GLYPH_SCALE);

            GraphNodeInput {
                id: input.id.clone(),
                kind: NodeKind::Note,
                group_id: None,
                position,
                mesh_kind: MeshKind::Cube,
                color: [120, 210, 165, 255],
                glyph: Some(glyph),
            }
        })
        .collect()
}

pub fn build_note_edges(knot_inputs: &[KnotInput]) -> Vec<GraphEdgeInput> {
    knot_inputs
        .iter()
        .flat_map(|input| pairwise_edges(&input.note_ids, EdgeKind::KnotMembership, true))
        .collect()
}

pub fn build_snippet_nodes(inputs: &[SnippetInput]) -> Vec<GraphNodeInput> {
    // Flatten all snippets with their metadata.
    let flat: Vec<(String, String, Vec<f32>)> = inputs
        .iter()
        .flat_map(|input| {
            (0..input.snippet_count).map(move |index| {
                let snippet_id = format!("snippet:{}:{}", input.note_id, index);
                let embedding = input.embeddings.get(index).cloned().unwrap_or_default();
                (input.note_id.clone(), snippet_id, embedding)
            })
        })
        .collect();

    let embeddings: Vec<Vec<f32>> = flat.iter().map(|(_, _, e)| e.clone()).collect();

    // Positions: PaCMAP when possible, grid fallback otherwise.
    let positions = compute_positions(&embeddings, &PositionConfig::default());

    flat.iter()
        .enumerate()
        .map(|(index, (note_id, snippet_id, embedding))| {
            let position = positions
                .get(index)
                .copied()
                .unwrap_or_else(|| grid_position(index, 10, 1.4, 0.6));

            // Glyph: real glyph from embedding, or placeholder diamond.
            let glyph = glyph_for_embedding(embedding, GLYPH_SCALE);

            GraphNodeInput {
                id: snippet_id.clone(),
                kind: NodeKind::Snippet,
                group_id: Some(note_id.clone()),
                position,
                mesh_kind: MeshKind::Sphere,
                color: [118, 167, 255, 255],
                glyph: Some(glyph),
            }
        })
        .collect()
}

pub fn build_snippet_edges(inputs: &[SnippetInput]) -> Vec<GraphEdgeInput> {
    inputs
        .iter()
        .map(|input| {
            (0..input.snippet_count)
                .map(|index| format!("snippet:{}:{}", input.note_id, index))
                .collect::<Vec<String>>()
        })
        .flat_map(|ids| pairwise_edges(&ids, EdgeKind::NoteMembership, false))
        .collect()
}

pub fn build_knot_intermediate_nodes(knot_inputs: &[KnotInput]) -> Vec<GraphNodeInput> {
    knot_inputs
        .iter()
        .enumerate()
        .map(|(index, input)| GraphNodeInput {
            id: input.id.clone(),
            kind: NodeKind::Knot,
            group_id: None,
            position: grid_position(index, 5, 3.0, 1.0),
            mesh_kind: MeshKind::Capsule,
            color: [252, 200, 114, 255],
            glyph: None,
        })
        .collect()
}

pub fn build_knot_intermediate_edges(knot_inputs: &[KnotInput]) -> Vec<GraphEdgeInput> {
    knot_inputs
        .iter()
        .flat_map(|input| {
            input.child_knot_ids.iter().map(|child_id| GraphEdgeInput {
                from: input.id.clone(),
                to: child_id.clone(),
                kind: EdgeKind::ParentChild,
                visible: true,
            })
        })
        .collect()
}

pub fn build_knot_root_nodes(knot_inputs: &[KnotInput]) -> Vec<GraphNodeInput> {
    let mut child_ids = std::collections::HashSet::new();

    for input in knot_inputs {
        for child_id in &input.child_knot_ids {
            child_ids.insert(child_id.clone());
        }
    }

    knot_inputs
        .iter()
        .filter(|input| !child_ids.contains(&input.id))
        .enumerate()
        .map(|(index, input)| GraphNodeInput {
            id: input.id.clone(),
            kind: NodeKind::Knot,
            group_id: None,
            position: grid_position(index, 5, 3.0, 1.0),
            mesh_kind: MeshKind::Capsule,
            color: [250, 186, 120, 255],
            glyph: None,
        })
        .collect()
}

fn pairwise_edges(ids: &[String], kind: EdgeKind, visible: bool) -> Vec<GraphEdgeInput> {
    let mut edges = Vec::new();

    for (i, from) in ids.iter().enumerate() {
        for to in ids.iter().skip(i + 1) {
            edges.push(GraphEdgeInput {
                from: from.clone(),
                to: to.clone(),
                kind: kind.clone(),
                visible,
            });
        }
    }

    edges
}

fn grid_position(index: usize, per_row: usize, spacing: f32, height: f32) -> [f32; 3] {
    let row = index / per_row;
    let col = index % per_row;
    let x = col as f32 * spacing - (per_row as f32 - 1.0) * spacing * 0.5;
    let z = row as f32 * spacing;

    [x, height, z]
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- snippet builders ---

    #[test]
    fn snippet_nodes_multiple_notes() {
        let inputs = vec![
            SnippetInput {
                note_id: "note:a".into(),
                snippet_count: 2,
                embeddings: Vec::new(),
            },
            SnippetInput {
                note_id: "note:b".into(),
                snippet_count: 3,
                embeddings: Vec::new(),
            },
        ];
        let nodes = build_snippet_nodes(&inputs);

        assert_eq!(nodes.len(), 5);
        assert!(nodes.iter().all(|n| n.kind == NodeKind::Snippet));
        assert!(nodes.iter().all(|n| n.mesh_kind == MeshKind::Sphere));

        // First two belong to note:a
        assert_eq!(nodes[0].group_id.as_deref(), Some("note:a"));
        assert_eq!(nodes[1].group_id.as_deref(), Some("note:a"));
        // Next three belong to note:b
        assert_eq!(nodes[2].group_id.as_deref(), Some("note:b"));
        assert_eq!(nodes[3].group_id.as_deref(), Some("note:b"));
        assert_eq!(nodes[4].group_id.as_deref(), Some("note:b"));

        // IDs are synthetic
        assert_eq!(nodes[0].id, "snippet:note:a:0");
        assert_eq!(nodes[1].id, "snippet:note:a:1");
        assert_eq!(nodes[2].id, "snippet:note:b:0");
    }

    #[test]
    fn snippet_nodes_empty() {
        let nodes = build_snippet_nodes(&[]);
        assert!(nodes.is_empty());
    }

    #[test]
    fn snippet_nodes_zero_snippets() {
        let inputs = vec![SnippetInput {
            note_id: "note:a".into(),
            snippet_count: 0,
            embeddings: Vec::new(),
        }];
        let nodes = build_snippet_nodes(&inputs);
        assert!(nodes.is_empty());
    }

    #[test]
    fn snippet_edges_per_note() {
        let inputs = vec![
            SnippetInput {
                note_id: "note:a".into(),
                snippet_count: 3,
                embeddings: Vec::new(),
            },
            SnippetInput {
                note_id: "note:b".into(),
                snippet_count: 2,
                embeddings: Vec::new(),
            },
        ];
        let edges = build_snippet_edges(&inputs);

        // note:a has 3 snippets → 3 pairwise edges; note:b has 2 → 1 edge
        assert_eq!(edges.len(), 4);
        assert!(edges.iter().all(|e| e.kind == EdgeKind::NoteMembership));
        assert!(edges.iter().all(|e| !e.visible));

        // All edges for note:a reference note:a snippets
        assert!(edges[0].from.starts_with("snippet:note:a:"));
        assert!(edges[0].to.starts_with("snippet:note:a:"));
        // The last edge is for note:b
        assert!(edges[3].from.starts_with("snippet:note:b:"));
        assert!(edges[3].to.starts_with("snippet:note:b:"));
    }

    #[test]
    fn snippet_edges_single_snippet_no_edges() {
        let inputs = vec![SnippetInput {
            note_id: "note:a".into(),
            snippet_count: 1,
            embeddings: Vec::new(),
        }];
        let edges = build_snippet_edges(&inputs);
        assert!(edges.is_empty());
    }

    // --- note builders ---

    #[test]
    fn note_nodes() {
        let inputs = vec![
            NoteInput {
                id: "note:1".into(),
                snippet_count: 3,
                embeddings: Vec::new(),
            },
            NoteInput {
                id: "note:2".into(),
                snippet_count: 1,
                embeddings: Vec::new(),
            },
            NoteInput {
                id: "note:3".into(),
                snippet_count: 0,
                embeddings: Vec::new(),
            },
        ];
        let nodes = build_note_nodes(&inputs);

        assert_eq!(nodes.len(), 3);
        assert!(nodes.iter().all(|n| n.kind == NodeKind::Note));
        assert!(nodes.iter().all(|n| n.mesh_kind == MeshKind::Cube));
        assert!(nodes.iter().all(|n| n.group_id.is_none()));
        assert_eq!(nodes[0].id, "note:1");
        assert_eq!(nodes[1].id, "note:2");
        assert_eq!(nodes[2].id, "note:3");
    }

    #[test]
    fn note_nodes_includes_pending() {
        let inputs = vec![NoteInput {
            id: "note:temp:42".into(),
            snippet_count: 1,
            embeddings: Vec::new(),
        }];
        let nodes = build_note_nodes(&inputs);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "note:temp:42");
    }

    #[test]
    fn note_nodes_empty() {
        let nodes = build_note_nodes(&[]);
        assert!(nodes.is_empty());
    }

    #[test]
    fn note_edges() {
        let inputs = vec![KnotInput {
            id: "knot:1".into(),
            label: Some("test".into()),
            child_knot_ids: vec![],
            note_ids: vec!["note:a".into(), "note:b".into(), "note:c".into()],
        }];
        let edges = build_note_edges(&inputs);

        // 3 notes → 3 pairwise edges
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().all(|e| e.kind == EdgeKind::KnotMembership));
        assert!(edges.iter().all(|e| e.visible));
    }

    #[test]
    fn note_edges_multiple_knots() {
        let inputs = vec![
            KnotInput {
                id: "knot:1".into(),
                label: None,
                child_knot_ids: vec![],
                note_ids: vec!["note:a".into(), "note:b".into()],
            },
            KnotInput {
                id: "knot:2".into(),
                label: None,
                child_knot_ids: vec![],
                note_ids: vec!["note:c".into(), "note:d".into(), "note:e".into()],
            },
        ];
        let edges = build_note_edges(&inputs);

        // knot:1 → 1 edge, knot:2 → 3 edges
        assert_eq!(edges.len(), 4);
    }

    // --- knot intermediate builders ---

    #[test]
    fn knot_intermediate_nodes() {
        let inputs = vec![
            KnotInput {
                id: "knot:a".into(),
                label: Some("parent".into()),
                child_knot_ids: vec![],
                note_ids: vec![],
            },
            KnotInput {
                id: "knot:b".into(),
                label: Some("child".into()),
                child_knot_ids: vec![],
                note_ids: vec![],
            },
        ];
        let nodes = build_knot_intermediate_nodes(&inputs);

        assert_eq!(nodes.len(), 2);
        assert!(nodes.iter().all(|n| n.kind == NodeKind::Knot));
        assert!(nodes.iter().all(|n| n.mesh_kind == MeshKind::Capsule));
        assert_eq!(nodes[0].id, "knot:a");
        assert_eq!(nodes[1].id, "knot:b");
    }

    #[test]
    fn knot_intermediate_nodes_includes_pending() {
        let inputs = vec![KnotInput {
            id: "knot:temp:1".into(),
            label: None,
            child_knot_ids: vec![],
            note_ids: vec![],
        }];
        let nodes = build_knot_intermediate_nodes(&inputs);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "knot:temp:1");
    }

    #[test]
    fn knot_intermediate_edges() {
        let inputs = vec![
            KnotInput {
                id: "knot:parent".into(),
                label: Some("parent".into()),
                child_knot_ids: vec!["knot:child1".into(), "knot:child2".into()],
                note_ids: vec![],
            },
            KnotInput {
                id: "knot:child1".into(),
                label: Some("child1".into()),
                child_knot_ids: vec![],
                note_ids: vec![],
            },
        ];
        let edges = build_knot_intermediate_edges(&inputs);

        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|e| e.kind == EdgeKind::ParentChild));
        assert!(edges.iter().all(|e| e.visible));
        assert_eq!(edges[0].from, "knot:parent");
        assert_eq!(edges[0].to, "knot:child1");
        assert_eq!(edges[1].from, "knot:parent");
        assert_eq!(edges[1].to, "knot:child2");
    }

    #[test]
    fn knot_intermediate_edges_no_children() {
        let inputs = vec![KnotInput {
            id: "knot:leaf".into(),
            label: None,
            child_knot_ids: vec![],
            note_ids: vec![],
        }];
        let edges = build_knot_intermediate_edges(&inputs);
        assert!(edges.is_empty());
    }

    // --- knot root builders ---

    #[test]
    fn knot_root_filters_children() {
        let inputs = vec![
            KnotInput {
                id: "knot:root1".into(),
                label: Some("root1".into()),
                child_knot_ids: vec!["knot:child".into()],
                note_ids: vec![],
            },
            KnotInput {
                id: "knot:child".into(),
                label: Some("child".into()),
                child_knot_ids: vec![],
                note_ids: vec![],
            },
            KnotInput {
                id: "knot:root2".into(),
                label: Some("root2".into()),
                child_knot_ids: vec![],
                note_ids: vec![],
            },
        ];
        let nodes = build_knot_root_nodes(&inputs);

        assert_eq!(nodes.len(), 2);
        let ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"knot:root1"));
        assert!(ids.contains(&"knot:root2"));
        assert!(!ids.contains(&"knot:child"));
    }

    #[test]
    fn knot_root_includes_pending() {
        let inputs = vec![KnotInput {
            id: "knot:temp:1".into(),
            label: None,
            child_knot_ids: vec![],
            note_ids: vec![],
        }];
        let nodes = build_knot_root_nodes(&inputs);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "knot:temp:1");
    }

    #[test]
    fn knot_root_empty() {
        let nodes = build_knot_root_nodes(&[]);
        assert!(nodes.is_empty());
    }

    // --- pairwise edges ---

    #[test]
    fn pairwise_four_ids() {
        let ids: Vec<String> = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let edges = pairwise_edges(&ids, EdgeKind::NoteMembership, false);

        // 4 choose 2 = 6
        assert_eq!(edges.len(), 6);
        assert!(edges.iter().all(|e| e.kind == EdgeKind::NoteMembership));
        assert!(edges.iter().all(|e| !e.visible));
    }

    #[test]
    fn pairwise_two_ids() {
        let ids: Vec<String> = vec!["x".into(), "y".into()];
        let edges = pairwise_edges(&ids, EdgeKind::KnotMembership, true);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, "x");
        assert_eq!(edges[0].to, "y");
        assert!(edges[0].visible);
    }

    #[test]
    fn pairwise_single_id() {
        let ids: Vec<String> = vec!["solo".into()];
        let edges = pairwise_edges(&ids, EdgeKind::ParentChild, true);
        assert!(edges.is_empty());
    }

    #[test]
    fn pairwise_empty() {
        let edges = pairwise_edges(&[], EdgeKind::ParentChild, true);
        assert!(edges.is_empty());
    }

    // --- grid_position ---

    #[test]
    fn grid_position_first_row() {
        let pos = grid_position(0, 4, 2.0, 1.0);
        // col=0, row=0: x = 0*2.0 - 3.0*1.0 = -3.0, z = 0, y = 1.0
        assert_eq!(pos[0], -3.0);
        assert_eq!(pos[1], 1.0);
        assert_eq!(pos[2], 0.0);

        let pos = grid_position(3, 4, 2.0, 1.0);
        // col=3, row=0: x = 3*2.0 - 3.0*1.0 = 3.0
        assert_eq!(pos[0], 3.0);
        assert_eq!(pos[2], 0.0);
    }

    #[test]
    fn grid_position_second_row() {
        let pos = grid_position(4, 4, 2.0, 1.0);
        // col=0, row=1: x = -3.0, z = 1*2.0 = 2.0
        assert_eq!(pos[0], -3.0);
        assert_eq!(pos[2], 2.0);
    }
}
