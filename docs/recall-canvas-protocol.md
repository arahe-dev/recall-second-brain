# Recall Canvas Protocol

## Version: v0.5 (Concept Flowchart)

### CanvasOps v0.5

Primitives for programmatic board object creation.

**Ops:**
- `CreateText` — add text note at position
- `CreateShape` — add shape (rect, line, oval) with geometry
- `CreateArrow` — add arrow shape
- `CreateConnector` — stub for linking objects
- `SetSubpage` — stub for subpage metadata

**OpResult:**
- `ok: bool`
- `error: Option<String>` — human-readable error
- `object_ids: Vec<u64>` — IDs of created objects
- `messages: Vec<String>` — status messages

### Concept Flowcharts

- AI emits `recall-flowchart-v0.5` JSON spec (see `docs/concept-flow-spec.md`)
- `recall_ops` CLI validates and lays out the spec into a Board
- Board saves as normal `canvas_objects` JSON

### CLI

```powershell
cargo run --bin recall_ops -- flowchart <spec.json> --out <board.json>
cargo run --bin recall_ops -- validate-flowchart <spec.json>
cargo run --bin recall_ops -- inspect <board.json>
cargo run --bin recall_ops -- ops <ops.json> --out <board.json>
```

## Future v1

- Templates: reusable flowchart layouts
- Subpages: nested boards per node
- Links: typed connections between objects
- Richer assertions on spec validation
- Workspace inspection CLI

## Future v2

- Workspace protocol: multi-board management
- Documents/imports: import external formats
- Provenance/permissions: who created what
- Agent output mode: structured output for AI agents
- Event log: `ops.jsonl` append-only operation log
- Web/desktop portability: core protocol decoupled from renderer
