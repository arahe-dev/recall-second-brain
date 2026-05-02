# AI Flowchart Output Prompt

Copy and paste this prompt into any AI (DeepSeek, Qwen, GPT, Claude, Kimi) to generate a Recall-compatible concept flowchart spec.

---

```
You are generating a Recall flowchart spec. Return only valid JSON matching the recall-flowchart-v0.5 format described below. Do not include any explanation or markdown outside the JSON.

FORMAT:
{
  "version": "recall-flowchart-v0.5",
  "title": "<CONCEPT> Breakdown",
  "concept": "Brief description of <CONCEPT>",
  "layout": "tree-left-to-right",
  "style": {
    "horizontal_gap": 200,
    "vertical_gap": 80,
    "node_width": 140,
    "node_height": 48
  },
  "nodes": [
    {
      "id": "root",
      "label": "Root Concept",
      "detail": "One-line summary",
      "kind": "root",
      "children": [
        {
          "id": "branch1",
          "label": "Branch Name",
          "detail": "Explanation",
          "kind": "branch",
          "children": [
            { "id": "leaf1", "label": "Detail", "detail": "Explanation", "kind": "leaf" }
          ]
        }
      ]
    }
  ],
  "edges": []
}

RULES:
- Node IDs must be unique across all nodes and edges.
- Nested nodes use the "children" array.
- Use kind: root, branch, leaf, note, warning, example, or formula.
- Keep labels short (2-5 words). Put longer explanations in "detail".
- Create 3-5 top-level branches minimum for useful depth.
- Each branch should have 2-4 child nodes where applicable.
- Include an "example" branch with concrete examples.
- Include a "formula" kind leaf for any mathematical/technical formulas.
- Stable, semantic IDs (not "node1", "node2").

CONCEPT: <CONCEPT>
AUDIENCE: <AUDIENCE>
DEPTH: <DEPTH> levels
STYLE: <STYLE>
```

---

## Concrete Example

**Prompt:**

```
Break down root locus into a multi-branch flowchart for a control systems student.
```

**Expected output:** A JSON spec with root → definition, construction rules, example, stability branches, each with detailed child nodes. See `examples/flowcharts/root-locus.json` for a working example.
