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
}

// ── Backward-compatible deserialization ──
// Old files used { strokes: [...], text_notes: [...] }.
// New files use { canvas_objects: [{ type: "stroke", ... }, ...] }.
// This deserializer handles both formats transparently.

mod backward_compat {
    use super::*;

    #[derive(Deserialize)]
    struct LegacyBoard {
        id: String,
        title: String,
        strokes: Vec<DrawingStroke>,
        text_notes: Vec<TextNote>,
        created_at: String,
        updated_at: String,
    }

    #[derive(Deserialize)]
    struct NewBoard {
        id: String,
        title: String,
        canvas_objects: Vec<CanvasObject>,
        created_at: String,
        updated_at: String,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Board, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(obj) = value.as_object() {
            if obj.contains_key("canvas_objects") {
                return NewBoard::deserialize(value)
                    .map(|nb| {
                        Board {
                            id: nb.id,
                            title: nb.title,
                            canvas_objects: nb.canvas_objects,
                            created_at: nb.created_at,
                            updated_at: nb.updated_at,
                        }
                    })
                    .map_err(serde::de::Error::custom);
            }
            if obj.contains_key("strokes") {
                return LegacyBoard::deserialize(value)
                    .map(|lb| {
                        let mut objects: Vec<CanvasObject> = lb
                            .strokes
                            .into_iter()
                            .map(|s| CanvasObject::Stroke(s))
                            .collect();
                        objects.extend(
                            lb.text_notes
                                .into_iter()
                                .map(|n| CanvasObject::TextNote(n)),
                        );
                        Board {
                            id: lb.id,
                            title: lb.title,
                            canvas_objects: objects,
                            created_at: lb.created_at,
                            updated_at: lb.updated_at,
                        }
                    })
                    .map_err(serde::de::Error::custom);
            }
        }
        Err(serde::de::Error::custom(
            "Expected board with 'canvas_objects' or legacy 'strokes'/'text_notes'",
        ))
    }
}

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
    backward_compat::deserialize(deserializer).map(|b| b.canvas_objects)
}

#[derive(Serialize, Deserialize)]
struct BoardSerializationHelper {
    id: String,
    title: String,
    #[serde(serialize_with = "serialize_objects", deserialize_with = "deserialize_objects")]
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
        let helper = BoardSerializationHelper::deserialize(deserializer)?;
        Ok(Board {
            id: helper.id,
            title: helper.title,
            canvas_objects: helper.objects,
            created_at: helper.created_at,
            updated_at: helper.updated_at,
        })
    }
}
