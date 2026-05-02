# Supervisor Notes — recall-sketch-notes

Role: DeepSeek/OpenCode supervises PiQwen/Qwen implementation.
PiQwen is the primary builder. DeepSeek verifies, restarts, logs.

## Quick Reference

- **App repo**: C:\Users\arahe\recall-sketch-notes
- **PiQwen repo**: C:\Users\arahe\piqwen
- **Model**: Qwen3.6-35B-A3B-Q4_K_M (20.6 GB)
- **Server**: http://127.0.0.1:8080 (port 8080)
- **Pi command**: `pi -p "<prompt>" --provider local-qwen-agent --model qwen3.6-35b-a3b-local --no-session`
- **Signal ping**: C:\signal_ping\dist\signal\signal-cli.exe

## Working with Pi

Pi commands go through `pi.ps1` in PATH. Key flags:
- `-p "<prompt>"` — the prompt
- `--provider local-qwen-agent` — use local Qwen
- `--model qwen3.6-35b-a3b-local` — model alias
- `--no-session` — single-turn, no conversation history

Pi saves artifacts to the working directory under `.pi/` by default.

## Retry Protocol

1. Send prompt to PiQwen
2. Run cargo check
3. If fail: extract error, refine prompt, retry (max 2x per slice)
4. If still fail: simplify slice, document blocker
5. If pass: commit, update logs, move to next slice

## Current Status

- Phase 0: Preflight in progress
- Server: starting...
