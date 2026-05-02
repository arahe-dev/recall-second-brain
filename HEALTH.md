# Health Check — recall-sketch-notes

## Entry 1 — Preflight (Session 2)
- **Timestamp**: 2026-05-02
- **Phase**: 0 (Preflight)
- **Builder**: PiQwen/Qwen (planned), DeepSeek (supervisor)

## Build status
- cargo check: PASS
- cargo build: PASS (after killing stale PID 31692)
- tests: N/A

## Server status
- Qwen server: starting (background job, 20.6 GB model loading)
- Endpoint: http://127.0.0.1:8080
- Thinking: disabled (agent mode)
- Overheating risk: moderate (15W TDP, model uses CPU-MoE + GPU offload)

## Friction
- `pi tps` syntax changed (no -PromptSize/-MaxTokens flags)
- Server must run as background job to survive CLI timeout
- 20.6 GB model takes significant time to load
- Previous `pi -p` usage pattern unknown; may need --provider --model flags

## Current milestone
- phase: 0 (Preflight)
- completed: All inspections done, tools verified, server starting
- next: Phase 1 / Slice 1 (Core model refactor) — first PiQwen prompt

## Risk
- data loss risk: low (backup exists at recall-sketch-notes-mvp-20260502-153925.zip)
- server crash risk: moderate (model uses ~6.6 GB VRAM on 8 GB GPU)
- Qwen hallucination risk: low for bounded code changes with cargo check gate
- DeepSeek overreach risk: need to resist direct code edits, only supervise

## Decision
- continue — wait for server, then start Phase 1
