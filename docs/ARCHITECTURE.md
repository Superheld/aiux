# AIUX - Architektur

> Wie AIUX gebaut ist und gebaut werden soll.
> Aktueller Stand und Zielbild - klar getrennt.

---

## Leitprinzip: Event-Driven Architecture

AIUX ist ein event-getriebenes System. Alles was den Core erreicht, kommt als Event.
Alles was der Core tut, erzeugt Events. Der MQTT-Bus ist das Nervensystem.

```
Wahrnehmung (Nerve)  ->  Event auf Bus  ->  Core denkt  ->  Handlung (Tool)
```

**Aktueller Stand:** Noch nicht event-driven. Der Core ist eine synchrone REPL
(stdin -> LLM -> stdout). Die Event-Architektur kommt mit Phase 6 (MQTT-Bus).
Der Code sollte aber schon jetzt so strukturiert werden, dass Input als
Abstraktion behandelt wird - nicht als hartcodiertes stdin.

---

## Design Patterns

Patterns die wir bewusst einsetzen (nicht was Frameworks mitbringen):

### Eingebaut

| Pattern | Wo | Was es tut |
|---------|----|------------|
| **Repository** | MemoryTool | Abstrahiert Speicherzugriff. Agent sagt "merke dir X", nicht "schreib Datei Y". Backend austauschbar. |
| **Composite** | Preamble Assembly | System-Prompt aus Teilen zusammengebaut (soul + user + context). Neue Teile koennen dazukommen. |
| **Command** | Tool-Use | Jeder Tool-Call ist ein serialisiertes Command-Objekt (action + parameter). Neue Tools = neue Commands. |

### Geplant

| Pattern | Wann | Was es tut |
|---------|------|------------|
| **Observer** | Phase 6 | Nerves beobachten passiv, melden nur Relevantes. |
| **Publish/Subscribe** | Phase 6 | Nerves publishen, Core subscribt. Entkoppelt ueber MQTT. |
| **Mediator** | Phase 6 | Der Bus vermittelt. Komponenten kennen nur den Bus, nicht einander. |
| **Strategy** | Phase 6 | Jeder Nerve hat gleiche Schnittstelle, eigene Beobachtungs-Strategie. |

### Biologische Metaphern als Architektur

Die Metaphern sind nicht Deko - sie SIND die Architektur-Entscheidungen:

| Metapher | Pattern | Konsequenz |
|----------|---------|------------|
| Sinne (Nerves) | Observer | Passiv, filternd, dauerhaft |
| Nervensystem (Bus) | Pub/Sub + Mediator | Entkoppelt, asynchron |
| Gedaechtnis (Memory) | Repository | Abstrahiert, erweiterbar |
| Seele (soul.md) | Configuration as Identity | Persoenlichkeit = Konfiguration |
| Haende (Tools) | Command | Ausfuehrung als Objekt |
| Rhythmen (Scheduler) | Scheduled Jobs | Puls, Atem, Tagesrueckblick |

---

## Ueberblick (Zielbild)

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
│  - user.md + context als Kontext                 │
│                                                  │
│  Scheduler (tokio-cron-scheduler)                │
│  - Puls (5 Min), Atem (1h), Tag, Woche          │
│                                                  │
│  Memory                                          │
│  - Kurzzeit: Markdown-Dateien (context/)         │
│  - Konversation: JSON pro Tag                    │
│  - Langzeit: SQLite + RAG (rig-sqlite)           │
│                                                  │
│  Bus-Client (rumqttc)                            │
│  - MQTT Subscribe auf aiux/nerves/*              │
│  - Events empfangen, verarbeiten, reagieren      │
│                                                  │
│  Tools (rig Tool-Use)                            │
│  - Native Rust Tools (hardcoded im Core)         │
│  - Shell-Execution                               │
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

## Aktueller Stand (nach Phase 4.3)

Was tatsaechlich gebaut und lauffaehig ist:

```
┌──────────────────────────────────────────────────┐
│  aiux-core (REPL, kein Daemon)                    │
│                                                   │
│  LLM-Client (rig-core 0.31)                      │
│  - Anthropic Claude (API, Streaming)              │
│  - Tool-Use (MemoryTool)                          │
│                                                   │
│  Preamble (Boot-Sequence)                         │
│  - soul.md -> user.md -> context/*.md             │
│                                                   │
│  Memory                                           │
│  - Kurzzeit: context/*.md (Agent liest/schreibt)  │
│  - Konversation: conversation-YYYY-MM-DD.json     │
│                                                   │
│  REPL                                             │
│  - stdin -> LLM -> stdout (direkt, kein Bus)      │
│  - Befehle: quit, exit, clear                     │
└──────────────────────────────────────────────────┘
```

**Nicht gebaut:** Bus, Nerves, Scheduler, Daemon, Gateway, Shell-Tool,
RAG/Vector-Suche, Skills, Journal.

---

## Tech-Stack

### Eingebaut (in Cargo.toml)

| Crate | Version | Was |
|-------|---------|-----|
| **rig-core** | 0.31 | LLM Framework (Anthropic, Streaming, Tool-Use) |
| **tokio** | 1 | Async Runtime |
| **serde** + **serde_json** | 1 | Serialisierung (History, Tool-Parameter) |
| **schemars** | 1 | JSON Schema fuer Tool-Definitionen |
| **chrono** | 0.4 | Datum fuer taegliche History-Rotation |
| **thiserror** | 2 | Error-Typen (MemoryTool) |
| **anyhow** | 1 | Error-Handling (main) |
| **futures** | 0.3 | Stream-Verarbeitung (Streaming-Ausgabe) |
| **dotenvy** | 0.15 | .env laden (API-Key) |

### Geplant (noch nicht in Cargo.toml)

| Crate | Phase | Was |
|-------|-------|-----|
| **rig-sqlite** | 4.4 | Vector Store (SQLite + sqlite-vec) fuer RAG |
| **rumqttc** | 6 | MQTT Client fuer Event-Bus |
| **tokio-cron-scheduler** | 5 | Rhythmen (Puls, Atem, Tag, Woche) |
| **tract-onnx** | Fernziel | Lokale ONNX Inference auf Raspi |
| **llama-cpp-2** | Fernziel | Lokale LLMs (Offline-Fallback) |

### Infrastruktur

| Komponente | Status | Was |
|-----------|--------|-----|
| **Mosquitto** | geplant (Phase 6) | MQTT Broker (Event-Bus) |
| **SQLite** | geplant (Phase 4.4) | Langzeit-Memory + Vector Store |

---

## Boot-Sequence

Beim Start des Core wird der System-Prompt (Preamble) zusammengebaut:

```
1. soul.md        Wer bin ich? (Persoenlichkeit, Regeln, Stil)
2. user.md        Mit wem rede ich? (Bruce, Praeferenzen)
3. context/*.md   Was weiss ich? (Agent-Notizen, alphabetisch sortiert)
```

Danach wird die Tages-History geladen (`conversation-YYYY-MM-DD.json`).

**Geplant (spaeter):**
- journal/heute + journal/gestern in die Boot-Sequence (Phase 9)
- skills/*.md als zusaetzlicher Kontext (Phase 8)
- environment.md mit System-Infos (Phase 5)

---

## Memory-Modell

Drei Speicherformen, zwei davon eingebaut:

| Typ | Format | Lebensdauer | Status |
|-----|--------|-------------|--------|
| **Kurzzeit** | context/*.md | Permanent, vom Agent verwaltet | eingebaut |
| **Konversation** | conversation-YYYY-MM-DD.json | Pro Tag, REPL-History | eingebaut |
| **Langzeit** | SQLite + RAG (rig-sqlite) | Permanent, durchsuchbar | geplant (Phase 4.4) |

**Kurzzeit:** Der Agent schreibt/liest hier ueber das MemoryTool (write/read/list).
Wird beim naechsten Start als Teil der Preamble geladen.

**Konversation:** Automatisch gespeichert nach jedem Turn. Pro Tag eine neue Datei.
Beim Start wird nur der heutige Tag geladen. `clear` loescht den heutigen Tag.

**Langzeit:** Noch nicht gebaut. Soll semantische Suche ueber alle Erinnerungen
ermoeglichen (Embeddings + Vektor-Suche statt alles in den Preamble zu laden).

---

## Verzeichnisstruktur

### Repo

```
aiux/
├── core/                  # aiux-core
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # REPL, Boot-Sequence, History
│       └── memory.rs      # MemoryTool (Tool-Use)
├── nerve/                 # aiux-nerve (Platzhalter, nicht implementiert)
│   ├── Cargo.toml
│   └── src/main.rs
├── home/                  # Agent-Home (wird deployed)
│   ├── memory/
│   │   ├── soul.md        # Persoenlichkeit (= System-Prompt)
│   │   ├── user.md        # Wissen ueber den Menschen
│   │   └── context/       # Agent-Notizen (Laufzeit, vom Agent beschreibbar)
│   ├── skills/            # Expertise als Markdown (geplant, Phase 8)
│   └── tools/             # Tool-Definitionen (geplant, Fernziel)
├── scripts/
│   ├── install.sh         # System-Installer
│   └── deploy.sh          # home/ auf Raspi deployen
├── docs/                  # PRD, Architektur, Roadmap
├── Cargo.toml             # Workspace
└── README.md
```

Laufzeit-Dateien (nicht im Repo, in .gitignore):
- `home/memory/conversation-*.json` - Tages-History

### Auf dem Zielsystem (Zielbild)

```
/home/claude/
├── memory/
│   ├── soul.md                  # Persoenlichkeit
│   ├── user.md                  # Wissen ueber den Menschen
│   ├── context/                 # Agent-Notizen
│   ├── conversation-*.json      # Tages-History
│   ├── journal/                 # Lerntagebuch (geplant, Phase 9)
│   └── memory.db               # Langzeit-SQLite (geplant, Phase 4.4)
├── skills/                      # Expertise (geplant, Phase 8)
└── nerves/                      # Nerve-Programme (geplant, Phase 6)
    └── <name>/
        ├── nerve.toml
        └── <binary>
```

---

## Bus-Protokoll (geplant, Phase 6)

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

## Plattformen

AIUX ist primaer fuer Raspberry Pi gedacht, laeuft aber ueberall:

| Plattform | Status | Hinweise |
|-----------|--------|----------|
| Linux x86_64 | Unterstuetzt | Entwicklung, Server |
| Linux aarch64 | Unterstuetzt | Raspberry Pi 4 (Primaer-Ziel) |
| macOS Intel | Unterstuetzt | Entwicklung |
| macOS Apple Silicon | Unterstuetzt | Entwicklung |
| Windows x86_64 | Unterstuetzt | Entwicklung |

Alle aktuellen Dependencies sind Pure Rust und kompilieren fuer alle Plattformen.

### Cross-Compilation

```bash
# Fuer Raspberry Pi (auf dem Entwicklungsrechner)
cargo build --release --target aarch64-unknown-linux-musl

# Lokal (Entwicklung)
cargo build --release
```

---

## Referenzen

- [rig-core](https://github.com/0xPlaygrounds/rig) - LLM Framework
- [tract](https://github.com/sonos/tract) - ONNX Inference
- [rumqttc](https://github.com/bytebeamio/rumqtt) - MQTT Client
- [sqlite-vec](https://github.com/asg017/sqlite-vec) - Vector Store

---

*Letzte Aktualisierung: 2026-03-01*
