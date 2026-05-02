use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::canvas_ops::CanvasOp;
use crate::model::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowchartSpec {
    pub version: String,
    pub title: String,
    pub concept: String,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default)]
    pub style: FlowchartStyle,
    pub nodes: Vec<FlowNode>,
    #[serde(default)]
    pub edges: Vec<FlowEdge>,
}

fn default_layout() -> String { "tree-left-to-right".into() }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowchartStyle {
    #[serde(default = "default_hgap")]
    pub horizontal_gap: f32,
    #[serde(default = "default_vgap")]
    pub vertical_gap: f32,
    #[serde(default = "default_nw")]
    pub node_width: f32,
    #[serde(default = "default_nh")]
    pub node_height: f32,
    #[serde(default = "default_color")]
    pub node_color: [u8; 3],
    #[serde(default = "default_color")]
    pub edge_color: [u8; 3],
    #[serde(default = "default_sw")]
    pub stroke_width: f32,
}

fn default_hgap() -> f32 { 200.0 }
fn default_vgap() -> f32 { 80.0 }
fn default_nw() -> f32 { 140.0 }
fn default_nh() -> f32 { 50.0 }
fn default_color() -> [u8; 3] { [180, 200, 255] }
fn default_sw() -> f32 { 2.0 }

impl Default for FlowchartStyle {
    fn default() -> Self {
        Self {
            horizontal_gap: default_hgap(),
            vertical_gap: default_vgap(),
            node_width: default_nw(),
            node_height: default_nh(),
            node_color: default_color(),
            edge_color: default_color(),
            stroke_width: default_sw(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default = "default_kind")]
    #[allow(dead_code)]
    pub kind: String,
    #[serde(default)]
    pub children: Vec<FlowNode>,
    #[serde(default)]
    pub subpage: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_kind() -> String { "leaf".into() }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_edge_kind")]
    pub kind: String,
}

fn default_edge_kind() -> String { "arrow".into() }

/// Validate a flowchart spec, returning errors if invalid
pub fn validate_spec(spec: &FlowchartSpec) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();

    if spec.version.is_empty() {
        errors.push("version is required".into());
    }
    if spec.title.is_empty() {
        errors.push("title is required".into());
    }
    if spec.nodes.is_empty() {
        errors.push("at least one node required".into());
    }

    // Collect all node IDs (flat + nested)
    let mut all_ids: HashSet<String> = HashSet::new();
    fn collect_ids(nodes: &[FlowNode], ids: &mut HashSet<String>, errors: &mut Vec<String>) {
        for n in nodes {
            if n.id.is_empty() {
                errors.push("node with empty id".into());
            } else if !ids.insert(n.id.clone()) {
                errors.push(format!("duplicate node id: {}", n.id));
            }
            collect_ids(&n.children, ids, errors);
        }
    }
    collect_ids(&spec.nodes, &mut all_ids, &mut errors);

    // Validate edges reference valid nodes
    for e in &spec.edges {
        if !all_ids.contains(&e.from) {
            errors.push(format!("edge from unknown node: {}", e.from));
        }
        if !all_ids.contains(&e.to) {
            errors.push(format!("edge to unknown node: {}", e.to));
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Layout a nested tree: root node with children. Returns ops to create the board.
pub fn layout_nested_tree(spec: &FlowchartSpec, style: &FlowchartStyle) -> Vec<CanvasOp> {
    let mut ops: Vec<CanvasOp> = Vec::new();
    let mut id_counter: u64 = 1;
    let root_x = 100.0;
    let root_y = 300.0;

    // Flatten the tree into positioned nodes
    struct PlacedNode {
        id: String,
        label: String,
        detail: String,
        #[allow(dead_code)]
        kind: String,
        subpage: bool,
        x: f32,
        y: f32,
    }

    let mut placed: Vec<PlacedNode> = Vec::new();

    fn place_tree(
        nodes: &[FlowNode],
        parent_x: f32,
        parent_y: f32,
        depth: usize,
        branch_offset: &mut f32,
        gap_x: f32,
        gap_y: f32,
        nw: f32,
        nh: f32,
        placed: &mut Vec<PlacedNode>,
        parent_first: bool,
    ) {
        for (i, node) in nodes.iter().enumerate() {
            let x = if parent_first && depth == 0 {
                parent_x
            } else {
                parent_x + gap_x
            };
            let y = parent_y + *branch_offset + i as f32 * gap_y;
            placed.push(PlacedNode {
                id: node.id.clone(),
                label: node.label.clone(),
                detail: node.detail.clone(),
                kind: node.kind.clone(),
                subpage: node.subpage,
                x, y,
            });
            if !node.children.is_empty() {
                let mut child_offset = -((node.children.len() as f32 - 1.0) * gap_y / 2.0);
                place_tree(
                    &node.children, x, y + nh / 2.0, depth + 1,
                    &mut (child_offset), gap_x, gap_y, nw, nh, placed, false,
                );
            }
        }
    }

    let mut branch_offset: f32 = 0.0;
    place_tree(
        &spec.nodes, root_x, root_y, 0, &mut branch_offset,
        style.horizontal_gap, style.vertical_gap,
        style.node_width, style.node_height, &mut placed, true,
    );

    // Generate ops for each placed node
    for pn in &placed {
        let nid = id_counter; id_counter += 1;
        // Create shape (rounded rect)
        ops.push(CanvasOp::CreateShape {
            id: nid, x1: pn.x, y1: pn.y,
            x2: pn.x + style.node_width, y2: pn.y + style.node_height,
            shape_type: ShapeType::Rect,
            color: style.node_color, stroke_width: style.stroke_width,
        });
        // Create text label inside the rect
        let label_id = id_counter; id_counter += 1;
        ops.push(CanvasOp::CreateText {
            id: label_id,
            x: pn.x + 8.0, y: pn.y + 6.0,
            text: pn.label.clone(),
        });
        // Detail text below label (smaller, offset further)
        if !pn.detail.is_empty() {
            let detail_id = id_counter; id_counter += 1;
            ops.push(CanvasOp::CreateText {
                id: detail_id,
                x: pn.x + 8.0, y: pn.y + 24.0,
                text: pn.detail.clone(),
            });
        }
        // Subpage metadata
        if pn.subpage {
            ops.push(CanvasOp::SetSubpage { object_id: nid, subpage: true });
        }
    }

    // Generate edges (arrows between parent and child based on tree structure)
    fn gen_edges(
        nodes: &[FlowNode],
        parent_pos: &[(String, f32, f32)], // parent (id, right_edge_x, center_y)
        placed: &[PlacedNode],
        gap_x: f32,
        nw: f32,
        nh: f32,
        ops: &mut Vec<CanvasOp>,
        id_counter: &mut u64,
    ) {
        for node in nodes {
            // Find this node's position
            let pos = placed.iter().find(|p| p.id == node.id).unwrap();
            let parent_entry = parent_pos.iter().find(|(pid, _, _)| *pid == node.id);

            // Create arrow from parent to this node
            if let Some((_, px, py)) = parent_entry {
                let aid = *id_counter; *id_counter += 1;
                ops.push(CanvasOp::CreateArrow {
                    id: aid,
                    x1: *px, y1: *py,
                    x2: pos.x, y2: pos.y + nh / 2.0,
                    color: [180, 200, 255], stroke_width: 2.0,
                });
            }

            if !node.children.is_empty() {
                let mut child_parents: Vec<(String, f32, f32)> = Vec::new();
                for child in &node.children {
                    let cpos = placed.iter().find(|p| p.id == child.id).unwrap();
                    child_parents.push((
                        child.id.clone(),
                        pos.x + nw,
                        cpos.y + nh / 2.0,
                    ));
                }
                gen_edges(&node.children, &child_parents, placed, gap_x, nw, nh, ops, id_counter);
            }
        }
    }

    // Top-level nodes have no parent — skip edge from nothing
    let root_parents: Vec<(String, f32, f32)> = placed.iter().map(|p| {
        (p.id.clone(), p.x + style.node_width, p.y + style.node_height / 2.0)
    }).collect();
    gen_edges(&spec.nodes, &root_parents, &placed, style.horizontal_gap, style.node_width, style.node_height, &mut ops, &mut id_counter);

    ops
}

/// Build a flowchart spec from examples, then generate ops
pub fn spec_to_board(spec: &FlowchartSpec) -> Result<Board, Vec<String>> {
    validate_spec(spec)?;
    let style = &spec.style;
    let ops = layout_nested_tree(spec, style);
    let mut board = Board::new(&spec.title);
    let mut next_id: u64 = 1;
    let results = crate::canvas_ops::apply_ops(&mut board, &ops, &mut next_id);
    let errors: Vec<String> = results.iter().filter(|r| !r.ok).filter_map(|r| r.error.clone()).collect();
    if errors.is_empty() { Ok(board) } else { Err(errors) }
}
