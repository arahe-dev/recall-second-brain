use serde::{Deserialize, Serialize};

use crate::model::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum CanvasOp {
    CreateText { id: u64, x: f32, y: f32, text: String },
    CreateShape { id: u64, x1: f32, y1: f32, x2: f32, y2: f32, shape_type: ShapeType, color: [u8; 3], stroke_width: f32 },
    CreateArrow { id: u64, x1: f32, y1: f32, x2: f32, y2: f32, color: [u8; 3], stroke_width: f32 },
    CreateConnector { from_id: u64, to_id: u64 },
    SetSubpage { object_id: u64, subpage: bool },
}

#[derive(Clone, Debug)]
pub struct OpResult {
    pub ok: bool,
    pub error: Option<String>,
    pub object_ids: Vec<u64>,
    pub messages: Vec<String>,
}

impl OpResult {
    pub fn ok(object_ids: Vec<u64>) -> Self {
        Self { ok: true, error: None, object_ids, messages: vec!["ok".into()] }
    }
    pub fn err(msg: &str) -> Self {
        Self { ok: false, error: Some(msg.to_string()), object_ids: vec![], messages: vec![msg.into()] }
    }
}

pub fn apply_op(board: &mut Board, op: &CanvasOp, next_id: &mut u64) -> OpResult {
    match op {
        CanvasOp::CreateText { id, x, y, text } => {
            let actual_id = if *id == 0 { let n = *next_id; *next_id += 1; n } else { *id };
            board.canvas_objects.push(CanvasObject::TextNote(TextNote { id: actual_id, x: *x, y: *y, text: text.clone() }));
            OpResult::ok(vec![actual_id])
        }
        CanvasOp::CreateShape { id, x1, y1, x2, y2, shape_type, color, stroke_width } => {
            let actual_id = if *id == 0 { let n = *next_id; *next_id += 1; n } else { *id };
            board.canvas_objects.push(CanvasObject::Shape(Shape { id: actual_id, shape_type: shape_type.clone(), x1: *x1, y1: *y1, x2: *x2, y2: *y2, color: *color, stroke_width: *stroke_width }));
            OpResult::ok(vec![actual_id])
        }
        CanvasOp::CreateArrow { id, x1, y1, x2, y2, color, stroke_width } => {
            let actual_id = if *id == 0 { let n = *next_id; *next_id += 1; n } else { *id };
            board.canvas_objects.push(CanvasObject::Shape(Shape { id: actual_id, shape_type: ShapeType::Arrow, x1: *x1, y1: *y1, x2: *x2, y2: *y2, color: *color, stroke_width: *stroke_width }));
            OpResult::ok(vec![actual_id])
        }
        CanvasOp::CreateConnector { from_id, to_id } => {
            // Connectors store link metadata; create a small arrow if both objects exist
            // For v0.5, just note it. Rich connector lines deferred.
            let msg = format!("Connector {from_id} -> {to_id} (stub)");
            OpResult { ok: true, error: None, object_ids: vec![], messages: vec![msg] }
        }
        CanvasOp::SetSubpage { object_id, subpage } => {
            let msg = format!("Subpage flag {object_id}={subpage} (stub, persisted only if model supports it)");
            OpResult { ok: true, error: None, object_ids: vec![], messages: vec![msg] }
        }
    }
}

pub fn apply_ops(board: &mut Board, ops: &[CanvasOp], next_id: &mut u64) -> Vec<OpResult> {
    ops.iter().map(|op| apply_op(board, op, next_id)).collect()
}
