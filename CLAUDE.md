# AIUX - Claude Code Kontext

## Projekt

AIUX ist ein neuronales, LLM-gesteuertes Betriebssystem auf Basis von Alpine Linux.
Zielplattform: Raspberry Pi 4 (aarch64, 8GB RAM).

## Tech-Stack

- **Basis-OS**: Alpine Linux (musl, busybox)
- **Sprache**: Rust (Cross-Compile für aarch64-unknown-linux-musl)
- **LLM-Framework**: llm crate (Multi-Provider: Anthropic, Mistral)
- **Lokale Inference**: ONNX Runtime (Rust-Bindings)
- **IPC**: Unix Sockets
- **TUI**: ratatui
- **Web**: htmx + axum/actix
- **Dependencies**: cargo vendor (lokal, offline-fähig)

## Architektur

- **aiux-agent** - LLM-Client, Konversationen, Tool-Use
- **aiux-hub** - Verbindet Agent, Nerven und System
- **aiux-nerve** - Kleine lokale neuronale Netze (Daemons)
- **build/** - Image-Build-System (Alpine + AIUX = flashbares Image)

## Konventionen

- Commits: feat(<scope>):, fix(<scope>):, docs:, refactor:, test:
- Merges mit --no-ff
- Docs in docs/ (PRD.md, ROADMAP.md)
- ROADMAP.md ist die zentrale Quelle für den aktuellen Stand
