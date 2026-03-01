# AIUX - Claude Code Kontext

## Was ist AIUX?

Embodied AI - ein OS in dem eine KI lebt, nicht eine App auf einem OS.
Das Linux-System ist der Koerper, der Agent lernt ihn zu spueren und zu nutzen.
Siehe docs/PRD.md (Vision), docs/ARCHITECTURE.md (Technik), docs/ROADMAP.md (Phasen).

## Aktueller Stand (nach Phase 4.3)

Noch ein Kopf ohne Leib: REPL mit Memory, kein Daemon, keine Sinne.

- **core/src/main.rs** - REPL, Boot-Sequence, History-Persistenz
- **core/src/memory.rs** - MemoryTool (write/read/list auf context/)
- **nerve/** - Platzhalter, nicht implementiert
- **home/memory/** - soul.md, user.md, context/, conversation-*.json

## Architektur-Regeln

- **EDA ist das Ziel.** Aktuell synchrone REPL, aber Code so strukturieren
  dass Input als Abstraktion behandelt wird - nicht als hartcodiertes stdin.
- **Koerper-Metapher ist Architektur.** Nerves = Sinne, Tools = Haende,
  Memory = Gedaechtnis, Soul = Identitaet. Keine Deko, sondern Design-Entscheidungen.
- **Trennung:** Core kennt keine Nerves direkt. Kommunikation nur ueber Bus (MQTT).
- **Tools sind Rust-Code im Core.** Kein Plugin-System. Kommt spaeter.
- **Preamble = soul.md + user.md + context/*.md.** Reihenfolge ist wichtig.

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
