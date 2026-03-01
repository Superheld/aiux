# AIUX - Architektur

> Technische Beschreibung des Systems.
> Was AIUX ist, wie es gebaut ist, welche Entscheidungen dahinter stehen.

---

## Leitprinzip: Event-Driven Architecture

AIUX ist ein event-getriebenes System. Alles was den Core erreicht, kommt als Event.
Alles was der Core tut, erzeugt Events. Der MQTT-Bus ist das Nervensystem.

```
Wahrnehmung (Nerve)  ->  Event auf Bus  ->  Core denkt  ->  Handlung (Tool)
```

Intern: `tokio::sync::broadcast` Channel. Extern (Nerves, Gateway): MQTT (Mosquitto).
Siehe [EVENT-BUS.md](EVENT-BUS.md) fuer Details zu Events, Teilnehmern und Regeln.

---

## Agent-Factory & Provider-Abstraktion

### Problem

rig-core unterstuetzt viele Provider (Anthropic, OpenAI/Mistral, Ollama, etc.),
aber jeder erzeugt einen anderen Rust-Typ. `Agent<anthropic::Model>` und
`Agent<openai::Model>` sind fuer den Compiler verschiedene Typen.

### rig's Loesung

rig bietet eine **Application-Layer Abstraktion**: ab `client.agent("model")`
ist der Code bei allen Providern identisch. Preamble, Tools, Chat, Streaming -
alles gleich. Nur die Client-Erstellung (eine Zeile) ist provider-spezifisch:

```rust
// Anthropic
let client = anthropic::Client::from_env();

// Mistral (OpenAI-kompatible API)
let client = openai::Client::builder()
    .base_url("https://api.mistral.ai/v1")
    .api_key(&key).build();

// Ollama (lokal)
let client = ollama::Client::new(Nothing);

// Ab hier identisch fuer alle Provider:
let agent = client.agent(&model)
    .preamble(&preamble)
    .tool(memory_tool)
    .build();
```

### Agent-Factory

Eine Factory-Funktion liest die Config und baut den richtigen Agent:

1. Config bestimmt Provider + Modell
2. Factory matched auf den Provider-String und erstellt den passenden Client
3. Ab `client.agent(...)` ist der Code identisch (rig's Application Layer)
4. Der fertige Agent wird an seinen Bus-Task gebunden -
   der generische Typ bleibt intern

```
Config (TOML)                Factory                     Bus
┌──────────────┐      ┌──────────────────┐       ┌──────────────┐
│ provider     │─────>│ match provider { │──────>│ Events rein  │
│ model        │      │   "anthropic"    │       │ Events raus  │
│ temperature  │      │   "mistral"      │       │ Typ ist weg  │
│ api_key_env  │      │   "ollama"       │       └──────────────┘
└──────────────┘      └──────────────────┘
```

Der Bus ist die Abstraktionsschicht. Kein eigener Adapter-Layer noetig -
der Agent-Typ lebt nur innerhalb seines Tasks, nach aussen gibt es nur Events.

### Config-Struktur

```toml
# home/config.toml

[agents.main]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
temperature = 0.7

[agents.vision]
provider = "mistral"
model = "mistral-large-latest"
api_key_env = "MISTRAL_API_KEY"
```

Jeder Agent hat einen eigenen Config-Eintrag mit Provider und Modell.
Sub-Agents (z.B. Vision-Nerve) bekommen eigene Eintraege.
Der Agent wird zur Laufzeit aus den Zutaten (Client + Preamble + Tools) gebaut -
Aenderungen an der Config erfordern kein Rekompilieren.

---

## Rollen

Der Agent hat nicht eine feste Aufgabe, sondern nimmt verschiedene Rollen ein.
Jede Rolle ist eine eigenstaendige Agent-Instanz mit eigener Config, eigenem
Memory und eigenen Nerves. Rollen koennen parallel laufen.

### Was eine Rolle definiert

- **role.md** - Wer bin ich in dieser Rolle, was darf ich, was nicht
- **Config** - Welches Modell, welche Temperature, welche Nerves
- **Memory** - Kontextspezifisches Wissen fuer diese Rolle
- **Nerves** - Auf welche Kanaele hoert diese Rolle, welche ignoriert sie

### Was IMMER gleich bleibt

- **soul.md** - Die Identitaet. Egal welche Rolle, der Agent bleibt derselbe.
- **user.md** - Der Mensch. Jede Rolle kennt Bruce.

Die Preamble einer Rolle: `soul + user + role + role-memory + role-context`.

### Main ist das Gehirn

`main` ist keine Rolle wie die anderen - es ist das Gehirn, der Boss.
Andere Rollen (Assistent, Kuenstler, System-Admin) sind Instanzen die
`main` starten, steuern und beenden kann.

Kommunikation zwischen Rollen laeuft ueber den Bus:
- `main` kann eine Rolle interviewen ("Was hast du heute gemacht?")
- Eine Rolle kann Feedback an `main` geben ("Ich brauche Zugriff auf X")
- `main` entscheidet, delegiert, koordiniert

### Beispiel

```
main (Gehirn, Anthropic Claude)
  ├── assistent (Rolle, Anthropic Claude)
  │     Config: gleicher Provider, eigener Memory
  │     Nerves: nerve-input (REPL/Gateway)
  │
  ├── maler (Rolle, Mistral/Ollama)
  │     Config: anderer Provider, eigener Memory
  │     Nerves: nerve-vision, nerve-file
  │
  └── sysadmin (Rolle, Anthropic Claude)
        Config: gleicher Provider, eigener Memory
        Nerves: nerve-system, nerve-log, nerve-net
```

Jede Rolle laeuft als eigener Task mit eigenem Agent.
Der Bus verbindet alles. Der Mensch kann mit jeder Rolle direkt sprechen,
oder `main` entscheidet wer antwortet.

### Konsequenz fuer Config und Verzeichnisse

Die Systemkonfiguration (Provider, API-Keys) gehoert nicht in den
Arbeitsbereich einer Rolle. Die Trennung:

- **Systemebene** - Provider, API-Keys, Bus-Config (geteilt)
- **Rollenebene** - Modell, Temperature, Nerves, Memory (pro Rolle)
- **Identitaet** - soul.md, user.md (geteilt, immer geladen)

Wie genau die Verzeichnisstruktur aussieht, wird entschieden wenn
die erste Rolle neben `main` entsteht.

### Conversations sind kein Rollen-Konzept

Conversations (conversation-*.json) sind reines Log - Tages-History
damit der Agent weiss wo er aufgehoert hat. Sie gehoeren nicht zu einer
Rolle, sondern sind ein technisches Detail der Kontextverwaltung.

---

## Design Patterns

Patterns die wir bewusst einsetzen (nicht was Frameworks mitbringen):

### Eingebaut

| Pattern | Wo | Was es tut |
|---------|----|------------|
| **Repository** | MemoryTool | Abstrahiert Speicherzugriff. Agent sagt "merke dir X", nicht "schreib Datei Y". Backend austauschbar. |
| **Composite** | Preamble Assembly | System-Prompt aus Teilen zusammengebaut (soul + user + context). Neue Teile koennen dazukommen. |
| **Command** | Tool-Use | Jeder Tool-Call ist ein serialisiertes Command-Objekt (action + parameter). Neue Tools = neue Commands. |

### Nerves & Bus

| Pattern | Wo | Was es tut |
|---------|----|------------|
| **Observer** | Nerves | Nerves beobachten passiv, melden nur Relevantes. |
| **Publish/Subscribe** | MQTT Bus | Nerves publishen, Core subscribt. Entkoppelt ueber MQTT. |
| **Mediator** | Bus | Der Bus vermittelt. Komponenten kennen nur den Bus, nicht einander. |
| **Strategy** | Nerves | Jeder Nerve hat gleiche Schnittstelle, eigene Beobachtungs-Strategie. |
| **Factory** | Agent-Factory | Baut Agents anhand Config. Provider-Typ bleibt intern. |

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
│  Agent-Factory + LLM-Client (rig-core)            │
│  - Provider per Config (Anthropic, Mistral, ...) │
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

## Komponenten

### Core (`core.rs`)

Das Gehirn. Kapselt den rig-Agent, Preamble und History.
Subscribt auf `UserInput` Events, publiziert `ResponseToken`/`ResponseComplete`.
Baut den Agent bei jedem Input neu (so greifen Preamble-Aenderungen sofort).

### REPL (`repl.rs`)

Kommandozeile. Liest von stdin, publiziert `UserInput` Events.
Empfaengt Response-Events und gibt sie auf stdout aus.
Austauschbar durch Gateway (HTTP, Telegram, etc.).

### Event-Bus (`bus.rs`)

Interner `tokio::sync::broadcast` Channel. Verteilt Events an alle Subscriber.
Siehe [EVENT-BUS.md](EVENT-BUS.md).

### Agent-Factory

Baut Agents anhand der Config. Matched auf Provider-String,
erstellt den passenden Client, bindet den Agent an den Bus.

### main.rs

Nur Verdrahtung: Bus erstellen, Core und REPL anschliessen, laufen lassen.

---

## Tech-Stack

| Crate / Komponente | Was |
|--------------------|-----|
| **rig-core** | LLM Framework (Multi-Provider, Streaming, Tool-Use) |
| **tokio** | Async Runtime |
| **serde** + **serde_json** | Serialisierung (History, Tool-Parameter) |
| **schemars** | JSON Schema fuer Tool-Definitionen |
| **chrono** | Datum fuer taegliche History-Rotation |
| **thiserror** | Error-Typen (MemoryTool) |
| **anyhow** | Error-Handling (main) |
| **futures** | Stream-Verarbeitung (Streaming-Ausgabe) |
| **dotenvy** | .env laden (API-Keys) |
| **rig-sqlite** | Vector Store (SQLite + sqlite-vec) fuer RAG |
| **rumqttc** | MQTT Client fuer externen Event-Bus |
| **tokio-cron-scheduler** | Rhythmen (Puls, Atem, Tag, Woche) |
| **tract-onnx** | Lokale ONNX Inference auf Raspi |
| **Mosquitto** | MQTT Broker (externes Nervensystem) |
| **SQLite** | Langzeit-Memory + Vector Store |

---

## Boot-Sequence

Beim Start des Core wird der System-Prompt (Preamble) zusammengebaut:

```
1. soul.md        Wer bin ich? (Persoenlichkeit, Regeln, Stil)
2. user.md        Mit wem rede ich? (Bruce, Praeferenzen)
3. context/*.md   Was weiss ich? (Agent-Notizen, alphabetisch sortiert)
```

Danach wird die Tages-History geladen (`conversation-YYYY-MM-DD.json`).

Spaetere Erweiterungen der Boot-Sequence:
- journal/heute + journal/gestern (Reflexion)
- skills/*.md als zusaetzlicher Kontext
- environment.md mit System-Infos

---

## Memory-Modell

Drei Speicherformen:

| Typ | Format | Lebensdauer |
|-----|--------|-------------|
| **Kurzzeit** | context/*.md | Permanent, vom Agent verwaltet |
| **Konversation** | conversation-YYYY-MM-DD.json | Pro Tag, REPL-History |
| **Langzeit** | SQLite + RAG (rig-sqlite) | Permanent, durchsuchbar |

**Kurzzeit:** Der Agent schreibt/liest hier ueber das MemoryTool (write/read/list).
Wird beim naechsten Start als Teil der Preamble geladen.

**Konversation:** Automatisch gespeichert nach jedem Turn. Pro Tag eine neue Datei.
Beim Start wird nur der heutige Tag geladen. `clear` loescht den heutigen Tag.

**Langzeit:** Semantische Suche ueber alle Erinnerungen
(Embeddings + Vektor-Suche statt alles in den Preamble zu laden).

---

## Verzeichnisstruktur

### Repo

```
aiux/
├── core/                  # aiux-core
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # Verdrahtung (Bus + Core + REPL)
│       ├── events.rs      # Event-Typen (UserInput, Response, Shutdown)
│       ├── bus.rs          # Interner Event-Bus (broadcast)
│       ├── core.rs         # Gehirn (rig-Agent, History, Preamble)
│       ├── config.rs       # Agent-Config (Provider, Modell, Temperature)
│       ├── repl.rs         # Kommandozeile (stdin/stdout ueber Bus)
│       └── memory.rs       # MemoryTool (Tool-Use)
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
