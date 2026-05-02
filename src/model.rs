use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Board {
    pub id: String,
    pub title: String,
    pub strokes: Vec<DrawingStroke>,
    pub text_notes: Vec<TextNote>,
    pub created_at: String,
    pub updated_at: String,
}

impl Board {
    pub fn new(title: &str) -> Self {
        Self {
            id: String::new(),
            title: title.to_string(),
            strokes: Vec::new(),
            text_notes: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrawingStroke {
    pub points: Vec<[f32; 2]>,
    pub color: [u8; 3],
    pub width: f32,
}

impl DrawingStroke {
    pub fn new(color: [u8; 3], width: f32) -> Self {
        Self {
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
