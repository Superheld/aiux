# AIUX - Claude Code Kontext

## Was ist AIUX?

Embodied AI - ein OS in dem eine KI lebt, nicht eine App auf einem OS.
Das Linux-System ist der Koerper, der Agent lernt ihn zu spueren und zu nutzen.
Siehe docs/PRD.md (Vision), docs/ARCHITECTURE.md (Technik), docs/ROADMAP.md (Phasen).

## Aktueller Stand

Noch ein Kopf ohne Leib: REPL mit Memory, kein Daemon, keine Sinne.
Aber: Event-Bus steht, Code ist modular, Provider per Config steuerbar.

- **core/src/main.rs** - Verdrahtung (Bus + Core + REPL)
- **core/src/core.rs** - Gehirn (rig-Agent, History, Preamble, Agent-Factory)
- **core/src/bus.rs** - Interner Event-Bus (tokio::sync::broadcast)
- **core/src/events.rs** - Event-Typen (UserInput, ResponseToken, etc.)
- **core/src/repl.rs** - Kommandozeile (stdin/stdout ueber Bus)
- **core/src/config.rs** - Agent-Config aus home/config.toml
- **core/src/memory.rs** - MemoryTool (write/read/list auf context/)
- **home/config.toml** - Provider, Modell, Temperature
- **home/memory/** - soul.md, user.md, context/, conversation-*.json
- **nerve/** - Platzhalter, nicht implementiert

## Architektur-Regeln

- **Koerper-Architektur.** Das System ist nach biologischem Vorbild gebaut:
  - **Grosshirn** = Core/LLM. Denkt in Sprache. Alles muss als Text ankommen.
  - **Hippocampus** = automatisches Memory. Hoert mit, speichert unbewusst.
  - **Nerves** = Fuehler zur Umwelt. Eigener Filter, Vorverarbeitung, melden als Text.
  - **Tools** = Haende. Bewusste Handlungen nach aussen.
  - **Chat** = direkter Zugang zum Grosshirn. Kein Nerve, kein Filter.
- **Trennung:** Core kennt keine Nerves direkt. Kommunikation nur ueber Bus.
- **Rollen:** Parallele Agent-Instanzen mit eigener Config/Memory. Main ist der Boss.
- **Tools sind Rust-Code im Core.** Kein Plugin-System. Kommt spaeter.
- **Preamble = soul.md + user.md + role.md + context/*.md.** Reihenfolge ist wichtig.

## Coding-Regeln

- **Sprache:** Rust
- **Error-Handling:** `anyhow` in main, `thiserror` fuer eigene Error-Typen
- **Async:** tokio Runtime, alles async
- **LLM:** rig-core 0.31 (Anthropic, Streaming, Tool-Use)
- **Serialisierung:** serde/serde_json, schemars fuer Tool-Schemas
- **Kein Over-Engineering.** Nur bauen was jetzt gebraucht wird.
- **Einfachstes zuerst.** Shell-Skript vor Rust-Daemon, wenn es reicht.
- **Code-Kommentare:** Englisch erlaubt, Deutsch bevorzugt

## Konventionen

- Commits: `feat(<scope>):`, `fix(<scope>):`, `docs:`, `refactor:`, `test:`
- Merges: `--no-ff`
- Sprache: Deutsch (Docs, Kommunikation, Commits)

## Raspi (Zielsystem)

- IP: 192.168.178.57, User: claude (uid 1000)
- SSH mit Key-Auth, Firewall aktiv
- Alpine Linux, Configs sichern: `lbu commit -d`
