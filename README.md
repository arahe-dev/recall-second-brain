# Recall Sketch Notes

A minimal local-first Rust desktop note-taking and sketch app.

## Stack

- **Rust** + **eframe/egui** 0.34 for UI
- **serde** + **serde_json** for persistence
- No cloud, no auth, no server

## Build

```powershell
cargo build
```

## Run

```powershell
cargo run
cargo run -- ./data/my_board.json  # load existing board
```

## Controls

| Action | Shortcut |
|--------|----------|
| Save   | Ctrl+S   |
| Undo   | Ctrl+Z   |
| Pen mode | Toolbar button |
| Text mode | Toolbar button |

## Project Structure

```
src/
  main.rs     - Entry point, window setup
  app.rs      - egui UI, drawing, input handling
  model.rs    - Data model (Board, DrawingStroke, TextNote)
  storage.rs  - JSON save/load
```

## License

MIT
