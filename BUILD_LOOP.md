# Build Loop — recall-sketch-notes

## Start
- **Date**: 2026-05-02
- **Goal**: Local-first Rust desktop note-taking/sketch app
- **Stack**: Rust + eframe/egui + serde + serde_json
- **Budget**: Up to 4 hours or stable MVP

## Timeline

### Phase 0 — Preflight
- [x] Inspect PiQwen AGENTS.md, README.md, wiki
- [x] Check toolchain (rustc 1.94.0, cargo 1.94.0, git 2.53.0)
- [x] Check target project (not exists → created)
- [x] Create BUILD_LOOP.md, HEALTH.md, RUN_NOTIFICATIONS.md
- [x] Initialize git
- [x] Check Signal/ping tool (found: C:\signal_ping\dist\signal\signal-cli.exe)

### Phase 1 — MVP Architecture (in progress)
- [ ] cargo init eframe project
- [ ] Data model (Board, Stroke, TextNote)
- [ ] Storage module (save/load JSON)
- [ ] App shell with dark canvas
- [ ] cargo check + cargo build pass
- [ ] git commit: MVP scaffold builds

### Phase 2 — Drawing Canvas
- [ ] Freehand drawing with pointer drag
- [ ] Store strokes in memory
- [ ] Undo last stroke
- [ ] Clear canvas
- [ ] Toolbar/status line
- [ ] cargo check + build pass
- [ ] git commit: Add freehand drawing canvas

### Phase 3 — Text Notes
- [ ] Add text note action
- [ ] Render text note on canvas
- [ ] Edit selected/latest text note
- [ ] Drag note
- [ ] cargo check + build pass
- [ ] git commit: Add text notes

### Phase 4 — Save/Load
- [ ] Save board to JSON
- [ ] Load board from JSON
- [ ] Status line feedback
- [ ] cargo check + build pass
- [ ] git commit: Add local save and load

### Phase 5 — MVP Polish
- [ ] README usage instructions
- [ ] Keyboard shortcuts (Ctrl+S, Ctrl+O, Ctrl+Z)
- [ ] Error handling (no panics on missing files)
- [ ] Visual style cleanup
- [ ] cargo check + build + test pass
- [ ] git commit: Polish MVP
- [ ] Create backup archive
- [ ] Ping/log: MVP complete

### Phase 6 — PiQwen Review Bundle
- [ ] Create PROJECT_REVIEW.md

### Phase 7 — Feature Experiments (if early)
- [ ] Branch: feature/experiments
- [ ] Candidate features (order dependent on time)

## Milestones
- (filled in as we go)

## Backup
- (filled in at MVP completion)
