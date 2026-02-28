# AIUX - Claude Code Kontext

## Projekt

AIUX ist ein neuronales, LLM-gesteuertes Betriebssystem auf Basis von Alpine Linux.
Embodied AI - ein OS das wahrnimmt, versteht und handelt.
Zielplattform: Raspberry Pi 4 (aarch64, 8GB RAM).

## Tech-Stack

- **Basis-OS**: Alpine Linux 3.23.3 (musl, busybox)
- **LLM-Engine**: aichat v0.30.0 (unverändert, als Subprocess via HTTP-API)
- **Core**: aiux-core (Rust Daemon) - Bus-Anbindung, Scheduler, Autonomie
- **Bus**: Mosquitto (MQTT, aus Alpine Repos)
- **Memory**: Markdown-Dateien (Kurzzeit) + SQLite/RAG via aichat (Langzeit)
- **Embedding**: Konfigurierbar (mistral-embed API oder Ollama lokal)
- **Tools**: MCP-Server + aichat llm-functions + Shell
- **Skills**: aichat Agents (Markdown Instructions)
- **Lokale Inference**: ONNX Runtime (für Nerves)
- **Dependencies**: cargo vendor (offline-fähig)

## Architektur

- **aiux-core** - Rust Daemon: wraps aichat, Bus, Scheduler, Autonomie
- **aiux-nerves** - Sinnesorgane (passiv, beobachten, melden auf MQTT-Bus)
- **aiux-gateway** - Plugin-Architektur für Zugangswege (SSH, Telegram, Web, App)
- **Mosquitto** - Event-Bus (MQTT Pub/Sub)
- **aichat** - LLM-Engine (Sessions, RAG, Tool-Use, MCP, HTTP-API)

Referenz-Architektur: OpenClaw (Konzepte übernommen, Implementierung eigen in Rust).

## Raspi

- IP: 192.168.178.57
- User: claude (uid 1000)
- SSH mit Key-Auth, Firewall aktiv
- Configs sichern: `lbu commit -d`

## Konventionen

- Commits: feat(<scope>):, fix(<scope>):, docs:, refactor:, test:
- Merges mit --no-ff
- Docs in docs/ (PRD.md = Vision, ROADMAP.md = Phasen-Plan)
- ROADMAP.md ist die zentrale Quelle für den aktuellen Stand
- Sprache: Deutsch (Code-Kommentare dürfen Englisch sein)
