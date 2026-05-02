use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Board {
    pub id: String,
    pub title: String,
    pub canvas_objects: Vec<CanvasObject>,
    pub created_at: String,
    pub updated_at: String,
}

impl Board {
    pub fn new(title: &str) -> Self {
        Self {
            id: String::new(),
            title: title.to_string(),
            canvas_objects: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CanvasObject {
    Stroke(DrawingStroke),
    TextNote(TextNote),
    Shape(Shape),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrawingStroke {
    pub id: u64,
    pub points: Vec<[f32; 2]>,
    pub color: [u8; 3],
    pub width: f32,
}

impl DrawingStroke {
    pub fn new(color: [u8; 3], width: f32) -> Self {
        Self {
            id: 0,
            points: Vec::new(),
            color,
            width,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextNote {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shape {
    pub id: u64,
    pub shape_type: ShapeType,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub color: [u8; 3],
    pub stroke_width: f32,
}

impl Shape {
    pub fn new(id: u64, shape_type: ShapeType, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { id, shape_type, x1, y1, x2, y2, color: [180, 200, 255], stroke_width: 2.0 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    Line,
    Arrow,
    Rect,
    Oval,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EraserMode {
    Element,
    Brush,
}

// ── Backward-compatible deserialization removed ──
// Old format {strokes, text_notes} handled inline in Deserialize for Board.
// backward_compat module removed in CanvasOps v0.5 patch.

fn serialize_objects<S>(objects: &Vec<CanvasObject>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    objects.serialize(serializer)
}

fn deserialize_objects<'de, D>(deserializer: D) -> Result<Vec<CanvasObject>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<CanvasObject>::deserialize(deserializer)
}

#[derive(Serialize, Deserialize)]
struct BoardSerializationHelper {
    id: String,
    title: String,
    #[serde(rename = "canvas_objects", serialize_with = "serialize_objects", deserialize_with = "deserialize_objects")]
    objects: Vec<CanvasObject>,
    created_at: String,
    updated_at: String,
}

impl Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        BoardSerializationHelper {
            id: self.id.clone(),
            title: self.title.clone(),
            objects: self.canvas_objects.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Try new format first, fall back to legacy {strokes, text_notes}
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(obj) = value.as_object() {
            if obj.contains_key("canvas_objects") {
                let helper: BoardSerializationHelper = serde_json::from_value(value)
                    .map_err(serde::de::Error::custom)?;
                return Ok(Board {
                    id: helper.id,
                    title: helper.title,
                    canvas_objects: helper.objects,
                    created_at: helper.created_at,
                    updated_at: helper.updated_at,
                });
            }
            if obj.contains_key("strokes") {
                let lb: LegacyBoard = serde_json::from_value(value)
                    .map_err(serde::de::Error::custom)?;
                let mut objects: Vec<CanvasObject> = lb.strokes.into_iter()
                    .map(|s| CanvasObject::Stroke(s)).collect();
                objects.extend(lb.text_notes.into_iter().map(|n| CanvasObject::TextNote(n)));
                return Ok(Board {
                    id: lb.id, title: lb.title,
                    canvas_objects: objects,
                    created_at: lb.created_at, updated_at: lb.updated_at,
                });
            }
        }
        Err(serde::de::Error::custom(
            "Expected board with 'canvas_objects' or legacy 'strokes'/'text_notes'",
        ))
    }
}

#[derive(Deserialize)]
struct LegacyBoard {
    id: String,
    title: String,
    strokes: Vec<DrawingStroke>,
    text_notes: Vec<TextNote>,
    created_at: String,
    updated_at: String,
}
