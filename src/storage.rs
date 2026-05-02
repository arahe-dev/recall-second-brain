use std::fs;
use std::path::Path;

use crate::model::Board;

pub fn save_board(board: &Board, path: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create data dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(board)
        .map_err(|e| format!("Failed to serialize board: {e}"))?;
    fs::write(path, &json).map_err(|e| format!("Failed to write file: {e}"))?;
    Ok(())
}

pub fn load_board(path: &str) -> Result<Board, String> {
    if !Path::new(path).exists() {
        return Err(format!("File not found: {path}"));
    }
    let json = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
    let board: Board =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse board: {e}"))?;
    Ok(board)
}
