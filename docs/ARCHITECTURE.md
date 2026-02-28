# AIUX - Architektur

> Wie AIUX gebaut ist. Tech-Stack, Abhaengigkeiten, Plattformen.

---

## Ueberblick

```
┌─────────────────────────────────────────────────┐
│  Gateway                                         │
│  SSH, Telegram, Web, App (Plugin-Architektur)    │
└──────────────────────┬──────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────┐
│  aiux-core (Rust Daemon)                         │
│                                                  │
│  LLM-Client (rig-core)                           │
│  - Anthropic Claude (API)                        │
│  - Streaming, Tool-Use, Function Calling         │
│  - soul.md als System-Prompt (Preamble)          │
│  - user.md + journal als Kontext                 │
│                                                  │
│  Scheduler (tokio-cron-scheduler)                │
│  - Puls (5 Min), Atem (1h), Tag, Woche          │
│                                                  │
│  Memory                                          │
│  - Kurzzeit: Markdown-Dateien (context/)         │
│  - Langzeit: SQLite + RAG (rig-sqlite)           │
│                                                  │
│  Bus-Client (rumqttc)                            │
│  - MQTT Subscribe auf aiux/nerves/*              │
│  - Events empfangen, verarbeiten, reagieren      │
│                                                  │
│  Tools (rig Tool-Use)                            │
│  - Native Rust Tools                             │
│  - Shell-Execution                               │
│  - MCP-Server (spaeter)                          │
└──────────────────┬───────────────────────────────┘
                   │ MQTT publish/subscribe
                   │
        ┌──────────▼──────────┐
        │  Mosquitto (MQTT)    │
        │  Event-Bus           │
        └──────────┬──────────┘
                   │
┌──────────────────▼───────────────────────────────┐
│  aiux-nerves                                      │
│                                                   │
│  Passive, dauerhafte Beobachtung.                 │
│  Filtern selbst, melden nur Relevantes.           │
│                                                   │
│  nerve-input    Direkte Interaktion               │
│  nerve-messages Eingehende Nachrichten            │
│  nerve-system   CPU, RAM, Disk, Temperatur        │
│  nerve-log      Syslog                            │
│  nerve-net      Netzwerk                          │
│  nerve-file     Dateisystem-Events                │
│  nerve-audio    Mikrofon (spaeter)                │
│  nerve-vision   Kamera (spaeter)                  │
│                                                   │
│  Lokale Inference: tract (ONNX, Pure Rust)        │
│  Lokale LLMs: llama-cpp-2 (optional, offline)     │
└───────────────────────────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────┐
│  Betriebssystem                                   │
│  Primaer: Alpine Linux (Raspi)                    │
│  Auch: jedes Linux, macOS, Windows                │
└──────────────────────────────────────────────────┘
```

---

## Tech-Stack

### Core

| Crate | Version | Was | Warum |
|-------|---------|-----|-------|
| **rig-core** | 0.31 | LLM Framework | Anthropic, Streaming, Tool-Use, RAG |
| **rig-sqlite** | - | Vector Store | SQLite + sqlite-vec, kein Server |
| **rumqttc** | 0.24 | MQTT Client | Pure Rust, Tokio-nativ |
| **tokio** | 1 | Async Runtime | Standard |
| **tokio-cron-scheduler** | 0.13 | Scheduler | Rhythmen (Puls/Atem/Tag/Woche) |
| **serde** + **serde_json** | 1 | Serialisierung | Standard |
| **pulldown-cmark** | 0.12 | Markdown Parser | soul.md, journal, skills |

### Nerves

| Crate | Was | Warum |
|-------|-----|-------|
| **tract-onnx** | ONNX Inference | Pure Rust, bewiesen auf Raspi |
| **llama-cpp-2** | Lokale LLMs | Offline-Fallback (optional) |
| **notify** | File-Watching | nerve-file |

### Infrastruktur

| Komponente | Was |
|-----------|-----|
| **Mosquitto** | MQTT Broker (Event-Bus) |
| **SQLite** | Langzeit-Memory + Vector Store |

---

## Plattformen

AIUX ist primaer fuer Raspberry Pi gedacht, laeuft aber ueberall:

| Plattform | Status | Hinweise |
|-----------|--------|----------|
| Linux x86_64 | Unterstuetzt | Entwicklung, Server |
| Linux aarch64 | Unterstuetzt | Raspberry Pi 4 (Primaer-Ziel) |
| macOS Intel | Unterstuetzt | Entwicklung |
| macOS Apple Silicon | Unterstuetzt | Entwicklung |
| Windows x86_64 | Unterstuetzt | Entwicklung |

Alle Core-Dependencies sind Pure Rust und kompilieren fuer alle Plattformen.

Einschraenkungen:
- **sqlite-vec**: musl-Builds muessen getestet werden (C-Dependency)
- **llama-cpp-2**: C++-Dependency, braucht Compiler auf dem Zielsystem
- **Mosquitto**: Muss auf dem Zielsystem installiert sein

### Cross-Compilation

```bash
# Fuer Raspberry Pi (auf dem Entwicklungsrechner)
cargo build --release --target aarch64-unknown-linux-musl

# Lokal (Entwicklung)
cargo build --release
```

---

## Verzeichnisstruktur

### Repo

```
aiux/
├── core/                # aiux-core (Rust Daemon)
│   ├── Cargo.toml
│   └── src/main.rs
├── nerve/               # aiux-nerve (Rust)
│   ├── Cargo.toml
│   └── src/main.rs
├── home/                # Agent-Home (wird deployed)
│   ├── memory/
│   │   ├── soul.md      # Persoenlichkeit (= System-Prompt)
│   │   ├── user.md      # Wissen ueber den Menschen
│   │   ├── context/     # Kurzzeit (Laufzeit)
│   │   └── journal/     # Lerntagebuch (Laufzeit)
│   ├── skills/          # Expertise als Markdown
│   └── tools/           # Tool-Definitionen
├── build/               # Alpine Image-Build Config
├── scripts/             # deploy.sh
├── docs/                # PRD, Architektur, Roadmap
├── Cargo.toml           # Workspace
└── README.md
```

### Auf dem Zielsystem

```
/home/claude/                    # Agent-Home
├── memory/
│   ├── soul.md                  # Persoenlichkeit (wächst mit der Zeit)
│   ├── user.md                  # Wissen ueber den Menschen
│   ├── context/                 # Kurzzeit-Gedaechtnis
│   ├── journal/                 # Lerntagebuch (YYYY-MM-DD.md)
│   └── memory.db               # Langzeit (SQLite + Vektoren)
├── skills/                      # Expertise
├── tools/                       # Tool-Definitionen
└── nerves/                      # Nerve-Programme + Configs
    └── <name>/
        ├── nerve.toml           # Config
        ├── <binary>             # Nerve-Programm
        └── model.onnx           # Optional: lokales Modell
```

---

## Bus-Protokoll

MQTT Topics:

```
aiux/nerves/<name>/events    # Nerve-Events
aiux/core/commands           # Befehle an den Core
aiux/core/status             # Core-Status
```

Event-Format (JSON):

```json
{
  "source": "nerve-log",
  "type": "anomaly",
  "priority": "medium",
  "data": { "line": "sshd: failed login from ...", "score": 0.87 },
  "timestamp": "2026-02-28T14:30:00Z"
}
```

Prioritaeten:
- **low** - Core schaut beim naechsten Heartbeat
- **medium** - Core wird sofort aktiv
- **high** - Core wird aktiv + Mensch wird benachrichtigt
- **critical** - Sofortige Benachrichtigung ueber alle Kanaele

---

## Session-Modell

Inspiriert von OpenClaw:

- **Eine Main-Session** fuer Mensch + Heartbeat (geteilter Kontext)
- **Nerve-Events** brauchen keine eigene Session (kontextbasiert)
- **Boot-Sequence**: soul.md -> user.md -> journal/heute -> journal/gestern
- **Heartbeat**: Periodisch, stille Bestaetigung wenn nichts los ist

---

## Referenzen

- [rig-core](https://github.com/0xPlaygrounds/rig) - LLM Framework
- [tract](https://github.com/sonos/tract) - ONNX Inference
- [rumqttc](https://github.com/bytebeamio/rumqtt) - MQTT Client
- [sqlite-vec](https://github.com/asg017/sqlite-vec) - Vector Store
- [OpenClaw](https://github.com/openclaw/openclaw) - Referenz-Architektur (Konzepte)

---

*Letzte Aktualisierung: 2026-02-28*
