# Project Review — recall-sketch-notes

## What Was Built

A minimal local-first Rust desktop note-taking and sketch app with:
- Dark canvas with freehand drawing (Pen mode)
- Text notes with inline editing (Text mode; click to create, click existing to edit)
- Drag text notes to reposition
- Undo last stroke, clear all
- Save/Load board as JSON files
- Save As... with timestamped filenames
- Keyboard shortcuts: Ctrl+S (save), Ctrl+Z (undo), Ctrl+O (load dialog)
- Status bar with stroke/note count and unsaved indicator
- File path text input for loading (no native file dialog dependency)

## Architecture

```
src/
  main.rs     — Entry point, CLI args, eframe::run_native
  app.rs      — egui App trait impl, UI layout, input handling, drawing
  model.rs    — Data model (Board, DrawingStroke, TextNote) with serde
  storage.rs  — JSON save/load functions
```

- **UI Framework**: eframe/egui 0.34 (immediate mode)
- **Persistence**: serde_json (no database)
- **Dependencies**: eframe, serde, serde_json only

## Files Changed

| File | Lines | Purpose |
|------|-------|---------|
| `src/main.rs` | 24 | Entry point, window setup |
| `src/app.rs` | ~460 | Full UI: toolbar, canvas, status, drawing, text editing |
| `src/model.rs` | 61 | Board, DrawingStroke, TextNote with Serialize/Deserialize |
| `src/storage.rs` | 25 | JSON save/load with error handling |
| `Cargo.toml` | 9 | Dependencies (eframe 0.34, serde 1.0, serde_json 1.0) |
| `README.md` | 42 | Build/run/controls docs |
| `BUILD_LOOP.md` | — | Build timeline log |
| `HEALTH.md` | — | Health/friction log |
| `RUN_NOTIFICATIONS.md` | — | Notification log |
| `PROJECT_REVIEW.md` | — | This file |

## Known Limitations

- No native file dialog (avoids `rfd` dep); uses text input for load path
- No autosave
- No undo for text edits
- Text notes are single-line only
- No stroke color/width selection (hardcoded blue)
- No eraser tool
- No zoom/pan
- No multiple boards/browser
- No image export

## Next Features (priority order)

1. Multiple boards list/sidebar
2. Stroke color picker and width slider
3. Eraser tool
4. Zoom and pan
5. Export board to image
6. Lasso/select strokes
7. Markdown/single-line text notes
8. Local search over board titles/text

## Test Results

- cargo check: PASS
- cargo build: PASS
- cargo test: N/A (no tests written)
- Manual: App launches, draws strokes, creates/edits text notes, saves/loads JSON

## Build Commands

```powershell
cargo check
cargo build
cargo run                                            # new blank board
cargo run -- ./data/board_1234567890.json             # load existing board
```

## Backup

- **Path**: C:\Users\arahe\recall-sketch-notes-backups\recall-sketch-notes-mvp-20260502-153925.zip
- **Size**: 35 KB
- **Git commit**: 0422f1f (master)
