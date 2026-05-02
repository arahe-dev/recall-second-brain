# Recall Concept Flowchart Spec v0.5

## Overview

A model-agnostic JSON format for AI-generated concept breakdown flowcharts. Any AI (DeepSeek, Qwen, GPT, Claude, Kimi) can output this spec. Recall imports it and renders a multi-branch flowchart board.

## Format

```json
{
  "version": "recall-flowchart-v0.5",
  "title": "Concept Title",
  "concept": "Brief description of the concept",
  "layout": "tree-left-to-right",
  "style": { ... },
  "nodes": [ ... ],
  "edges": [ ... ]
}
```

### Top-level Fields

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| version | yes | string | Must be `"recall-flowchart-v0.5"` |
| title | yes | string | Board title |
| concept | yes | string | Concept being broken down |
| layout | no | string | `"tree-left-to-right"` (default) or `"tree-top-down"` |
| style | no | object | Node/edge styling (see below) |
| nodes | yes | array | Root nodes (can be nested with `children`) |
| edges | no | array | Flat edge list for non-tree graphs |

### Style

```json
{
  "horizontal_gap": 200,
  "vertical_gap": 80,
  "node_width": 140,
  "node_height": 50,
  "node_color": [180, 200, 255],
  "edge_color": [180, 200, 255],
  "stroke_width": 2
}
```

All fields optional. Defaults shown above.

### Node

```json
{
  "id": "unique-string-id",
  "label": "Short Label",
  "detail": "Optional longer explanation",
  "kind": "root|branch|leaf|note|warning|example|formula",
  "children": [ ... ],
  "subpage": false,
  "tags": []
}
```

| Field | Required | Description |
|-------|----------|-------------|
| id | yes | Unique string ID across all nodes and edges |
| label | yes | Short display label (rendered inside node rect) |
| detail | no | Secondary text (rendered below label) |
| kind | no | `"leaf"` (default), `"root"`, `"branch"`, `"note"`, `"warning"`, `"example"`, `"formula"` |
| children | no | Nested child nodes for tree layout |
| subpage | no | `true` marks node for future subpage expansion |
| tags | no | Arbitrary string tags |

### Edge

```json
{
  "from": "source-node-id",
  "to": "target-node-id",
  "label": "optional edge label",
  "kind": "arrow|line|dependency|example"
}
```

For tree layouts, edges are auto-generated from parent-child relationships. Explicit edges are used for flat graph formats.

## Layout

- **tree-left-to-right**: Root on left, branches spread vertically, children to the right.
- Horizontal gap between depth levels: `style.horizontal_gap` (default 200).
- Vertical gap between sibling nodes: `style.vertical_gap` (default 80).
- Node boxes are rounded rectangles. Text labels inside or near boxes.
- Arrows connect parent to child.

## Prompt for AI

When asking an AI to generate a flowchart spec, include:

```
You are generating a Recall flowchart spec. Return only JSON matching recall-flowchart-v0.5.
Break down <CONCEPT> into a multi-branch flowchart with root, branches, leaves,
examples, formulas where relevant. Use stable node IDs. Keep labels short.
Put explanations in detail fields.

Audience: <AUDIENCE>
Depth: <DEPTH> levels
Style: <STYLE>
```

## Validation Rules

- `version` must be non-empty
- `title` must be non-empty
- At least one node required
- All node IDs (including nested) must be unique
- Edge `from`/`to` must reference valid node IDs
