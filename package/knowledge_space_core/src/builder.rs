use crate::{EdgeKind, GraphEdgeInput, GraphNodeInput, MeshKind, NodeKind};

#[derive(Clone, Debug, PartialEq)]
pub struct NoteInput {
    pub id: String,
    pub snippet_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnippetInput {
    pub note_id: String,
    pub snippet_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KnotInput {
    pub id: String,
    pub label: Option<String>,
    pub child_knot_ids: Vec<String>,
    pub note_ids: Vec<String>,
}

pub fn build_note_nodes(note_inputs: &[NoteInput]) -> Vec<GraphNodeInput> {
    note_inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let position = grid_position(index, 8, 2.2, 0.8);
            GraphNodeInput {
                id: input.id.clone(),
                kind: NodeKind::Note,
                group_id: None,
                position,
                mesh_kind: MeshKind::Cube,
                color: [120, 210, 165, 255],
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
    inputs
        .iter()
        .flat_map(|input| {
            (0..input.snippet_count).map(move |index| {
                let snippet_id = format!("snippet:{}:{}", input.note_id, index);
                (input.note_id.clone(), snippet_id)
            })
        })
        .enumerate()
        .map(|(index, (note_id, snippet_id))| GraphNodeInput {
            id: snippet_id,
            kind: NodeKind::Snippet,
            group_id: Some(note_id),
            position: grid_position(index, 10, 1.4, 0.6),
            mesh_kind: MeshKind::Sphere,
            color: [118, 167, 255, 255],
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

fn grid_position(index: usize, per_row: usize, spacing: f32, height: f32) -> [f32; 3] {
    let row = index / per_row;
    let col = index % per_row;
    let x = col as f32 * spacing - (per_row as f32 - 1.0) * spacing * 0.5;
    let z = row as f32 * spacing;

    [x, height, z]
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
