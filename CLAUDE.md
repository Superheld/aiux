# AIUX - Claude Code Kontext

## Projekt

AIUX ist ein neuronales, LLM-gesteuertes Betriebssystem.
Embodied AI - ein OS das wahrnimmt, versteht und handelt.
Primaer: Raspberry Pi 4 (aarch64, 8GB RAM), laeuft aber ueberall.

## Tech-Stack

- **LLM Framework**: rig-core (Anthropic Claude, Streaming, Tool-Use, RAG)
- **Vector Store**: rig-sqlite (SQLite + sqlite-vec)
- **Bus**: Mosquitto (MQTT) + rumqttc (Pure Rust Client)
- **Lokale Inference**: tract-onnx (Pure Rust ONNX)
- **Async**: tokio
- **Scheduler**: tokio-cron-scheduler
- **Sprache**: Rust

## Architektur

- **core/** - aiux-core: Rust Daemon (LLM, Bus, Scheduler, Memory, Tools)
- **nerve/** - aiux-nerve: Sinnesorgane (passiv, beobachten, melden auf Bus)
- **home/** - Agent-Home: soul.md, user.md, skills, tools (wird deployed)

soul.md = System-Prompt, wird direkt von rig-core als Preamble geladen.
Keine externen Tools die ihre eigene Struktur aufzwingen.

Siehe docs/ARCHITECTURE.md fuer Details.

## Raspi

- IP: 192.168.178.57
- User: claude (uid 1000)
- SSH mit Key-Auth, Firewall aktiv
- Configs sichern: `lbu commit -d`

## Konventionen

- Commits: feat(<scope>):, fix(<scope>):, docs:, refactor:, test:
- Merges mit --no-ff
- Docs: PRD.md (Produkt), ARCHITECTURE.md (Technik), ROADMAP.md (Phasen)
- Sprache: Deutsch (Code-Kommentare duerfen Englisch sein)
