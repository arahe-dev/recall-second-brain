# Recall Sketch Notes

A minimal local-first Rust desktop note-taking and sketch app with concept flowchart import.

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

## CLI Tool (recall_ops)

```powershell
# Generate a flowchart board from a spec
cargo run --bin recall_ops -- flowchart examples/flowcharts/root-locus.json --out data/current-board.json

# Validate a flowchart spec
cargo run --bin recall_ops -- validate-flowchart examples/flowcharts/root-locus.json

# Inspect a board file
cargo run --bin recall_ops -- inspect data/current-board.json

# Apply raw CanvasOps
cargo run --bin recall_ops -- ops <ops.json> --out <board.json>
```

## Flowchart Import

Any AI can emit a `recall-flowchart-v0.5` JSON spec. Use the CLI to convert it to a board, then load with `cargo run -- data/current-board.json` or use the "Fl" button in the toolbar.

See `docs/concept-flow-spec.md` for the spec format and `docs/ai-flowchart-output-prompt.md` for an AI prompt template.

## Controls

| Action | Shortcut |
|--------|----------|
| Save   | Ctrl+S   |
| Undo   | Ctrl+Z   |
| Load   | Ctrl+O   |
| Pen mode | Toolbar button |
| Text mode | Toolbar button |

## Project Structure

```
src/
  main.rs         - Entry point, window setup
  lib.rs          - Module declarations
  app.rs          - egui UI, drawing, input handling
  model.rs        - Data model (Board, CanvasObject)
  storage.rs      - JSON save/load
  canvas_ops.rs   - Programmatic object creation (CanvasOp)
  flowchart.rs    - Flowchart spec format + layout engine
  bin/
    recall_ops.rs - CLI tool for flowchart ops
docs/
  concept-flow-spec.md           - Flowchart spec format
  recall-canvas-protocol.md      - Protocol docs (v0.5/v1/v2)
  ai-flowchart-output-prompt.md  - Copy-paste prompt for AIs
examples/flowcharts/
  root-locus.json        - Control systems example
  8051-timers.json       - Embedded systems example
  local-first-recall.json - Recall architecture example
```

## License

MIT
