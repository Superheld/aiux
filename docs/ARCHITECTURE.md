# AIUX - Architektur

> Technische Beschreibung des Systems.
> Was AIUX ist, wie es gebaut ist, welche Entscheidungen dahinter stehen.

---

## Leitprinzip: Koerper-Architektur

AIUX ist nach dem Vorbild eines Koerpers gebaut. Nicht als 1:1 Kopie des
Menschen, sondern als Inspiration. Die Architektur spiegelt biologische
Strukturen wider, adaptiert fuer ein System dessen Gehirn ein Sprachmodell ist.

### Das Gehirn

Nicht ein Gehirn, sondern Schichten - wie beim Menschen:

```
┌─────────────────────────────────────────┐
│  Grosshirn (Core/Main)                  │
│  Bewusstes Denken. Das LLM.             │
│  Sprache, Entscheidungen, Planung.      │
├─────────────────────────────────────────┤
│  Hippocampus (Memory)                   │
│  Hoert mit, speichert automatisch.      │
│  Unbewusste Gedaechtnisbildung.         │
├─────────────────────────────────────────┤
│  Hirnstamm (Scheduler)                  │
│  Grundfunktionen, Rhythmen.             │
│  Puls, Atem, Tagesrueckblick.           │
└─────────────────────────────────────────┘
```

**Grosshirn** = der Core. Ein Sprachmodell das denkt, spricht, entscheidet.
Alles was das Grosshirn erreicht, muss Sprache sein - es kann keine Bilder
sehen, keine Toene hoeren. Andere Komponenten uebersetzen fuer es.

**Hippocampus** = automatische Gedaechtnisbildung. Ein kleiner Prozess der
auf dem Bus mithoert und wichtige Dinge speichert, ohne dass das Grosshirn
bewusst entscheiden muss. "Bruce mag keine Emojis" wird automatisch gemerkt.
Das MemoryTool bleibt als bewusstes Aufschreiben - aber das meiste passiert
im Hintergrund.

**Hirnstamm** = Scheduler. Rhythmen die ohne bewusstes Denken laufen.
Puls, Atem, Tagesrueckblick.

### Nerves (Fuehler)

Nerves sind die Endpunkte zur Umwelt. Sie sind mit Hardware oder APIs
verbunden (Dateisystem, Netzwerk, Kamera, Mikrofon, Telegram).

Jeder Nerve hat seinen **eigenen Filter** (verteilter Thalamus).
Der Nerve entscheidet selbst was relevant ist und was nicht.
Dafuer kann er ein eigenes kleines Modell nutzen.

**Alles kommt als Sprache beim Core an.** Egal ob Kamera, Syslog oder
Telegram - der Nerve uebersetzt in Text. Das Grosshirn ist ein
Sprachmodell und bekommt Sprache.

```
nerve-system: "CPU bei 95%, Prozess X verbraucht am meisten"
nerve-vision: "Bewegung erkannt, Person vor der Tuer"
nerve-file:   "Datei config.toml wurde geaendert"
```

### Chat ist kein Nerve

Wenn Bruce redet, geht das **direkt** ins Grosshirn. Chat ist kein Sensor,
es ist bewusste Kommunikation - wie ein Gespraech von Angesicht zu Angesicht.
Kein Filter, keine Vorverarbeitung, kein Nerve dazwischen.

Chat-Zugaenge (REPL, Telegram, Web) sind **Gateways** - Kanaele ueber die
der Mensch direkt mit dem Gehirn spricht.

### Tools (Haende)

Tools sind aktive Handlungen. Das Grosshirn entscheidet bewusst etwas zu tun:
- `MemoryTool` - bewusst etwas aufschreiben
- `MessageTool` - Nachricht senden (Telegram, etc.)
- `ShellTool` - Befehl ausfuehren

Der Nerve hoert (Eingang), das Tool handelt (Ausgang).
Wie Ohr und Mund - verschiedene Systeme fuer verschiedene Richtungen.

### Bus (Nervensystem)

Der Bus verbindet alles. Intern `tokio::sync::broadcast`,
extern MQTT (Mosquitto) fuer Nerves.
Siehe [EVENT-BUS.md](EVENT-BUS.md) fuer Details.

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

Zwei Ebenen: System-Config (geteilt) und Rollen-Config (pro Rolle).

```toml
# home/config.toml - Systemebene
# Definiert verfuegbare Provider. API-Keys kommen aus Env-Variablen.

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"

[providers.mistral]
api_key_env = "MISTRAL_API_KEY"

[providers.ollama]
# Kein API-Key noetig, laeuft lokal
```

```toml
# home/roles/main/config.toml - Rollenebene
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
temperature = 0.7
```

Jede Rolle waehlt aus den verfuegbaren Providern.
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

### Beispiel: Sysadmin-Rolle

Der Sysadmin laeuft dauerhaft im Hintergrund. Braucht kein grosses Modell -
ein kleines lokales (Ollama) reicht. Hoert auf System-Nerves, meldet sich
nur bei `main` wenn es relevant ist.

```toml
# roles/sysadmin/config.toml
provider = "ollama"
model = "qwen2.5:7b"
temperature = 0.3
nerves = ["nerve-system", "nerve-log", "nerve-net"]
```

```toml
# roles/main/config.toml
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
temperature = 0.7
```

Typischer Ablauf:
1. `sysadmin` erkennt ueber `nerve-system`: Disk 90% voll
2. Publiziert Event auf den Bus: "Warnung: Disk 90% voll"
3. `main` entscheidet: Bruce stoeren oder selbst handeln?
4. `main` delegiert an `sysadmin`: "Loesch die alten Logs"
5. `sysadmin` fuehrt aus und meldet Ergebnis

Der Mensch merkt davon nichts - ausser `main` entscheidet dass es wichtig ist.

### Rollenwechsel im Terminal

Eine REPL, mehrere Rollen. Der Prompt zeigt die aktive Rolle:

```
main> hallo
AIUX: hey, was gibt's?

main> /role assistent
assistent> hilf mir bei X
AIUX: klar, ...

assistent> /role main
main> /roles
  * main        (aktiv)
    assistent   (laeuft)
    sysadmin    (laeuft)
```

Technisch: Die REPL hat einen "aktive Rolle" State und taggt
`UserInput` Events mit der Rolle. Nur der angesprochene Core antwortet.
Hintergrund-Rollen laufen weiter auf ihren Nerves.

### Config-Trennung

Die Systemkonfiguration (Provider, API-Keys) gehoert nicht in den
Arbeitsbereich einer Rolle. Zwei Ebenen:

- **Systemebene** (`home/config.toml`) - Provider-Definitionen, API-Keys
- **Rollenebene** (`home/roles/<name>/config.toml`) - Modell, Temperature, Nerves

Die Identitaet (soul.md, user.md) wird immer geladen, unabhaengig von der Rolle.

### Conversations sind pro Rolle

Conversations (conversation-*.json) sind Tages-History pro Rolle.
Jede Rolle hat ihre eigenen Gespraeche - wenn der Mensch mit dem
Assistenten redet, ist das ein anderer Kontext als mit `main`.

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

| Metapher | Komponente | Pattern | Konsequenz |
|----------|-----------|---------|------------|
| Grosshirn | Core/Main (LLM) | - | Bewusstes Denken, alles als Sprache |
| Hippocampus | Memory (Hintergrund) | Observer | Automatisches Speichern, hoert mit |
| Hirnstamm | Scheduler | Scheduled Jobs | Rhythmen ohne bewusstes Denken |
| Fuehler (Nerves) | Nerves | Observer + Strategy | Vorverarbeitung, eigener Filter |
| Nervensystem | Bus | Pub/Sub + Mediator | Entkoppelt, asynchron |
| Haende | Tools | Command | Bewusste Handlung nach aussen |
| Seele | soul.md | Config as Identity | Persoenlichkeit = Konfiguration |
| Gespraech | Chat/Gateway | - | Direkter Zugang zum Grosshirn, kein Nerve |

---

## Ueberblick (Zielbild)

```
┌─────────────────────────────────────────────────┐
│  Chat (direkter Zugang zum Grosshirn)            │
│  REPL, Telegram, Web, SSH                        │
└──────────────────────┬──────────────────────────┘
                       │ kein Nerve, direkt
                       │
┌──────────────────────▼──────────────────────────┐
│  aiux-core (Gehirn)                              │
│                                                  │
│  Grosshirn (Core/Main)                           │
│  - LLM (rig-core, Provider per Config)           │
│  - Rollen (eigene Instanzen, eigene Config)      │
│  - Tools (Haende: Memory, Shell, Messages)       │
│                                                  │
│  Hippocampus (automatisches Memory)              │
│  - Hoert auf dem Bus mit                         │
│  - Speichert automatisch was wichtig ist         │
│                                                  │
│  Hirnstamm (Scheduler)                           │
│  - Puls (5 Min), Atem (1h), Tag, Woche          │
│                                                  │
└──────────────────┬───────────────────────────────┘
                   │ Bus (Nervensystem)
                   │ intern: tokio::broadcast
                   │ extern: MQTT (Mosquitto)
                   │
┌──────────────────▼───────────────────────────────┐
│  Nerves (Fuehler zur Umwelt)                      │
│                                                   │
│  Jeder Nerve hat eigenen Filter (Thalamus).       │
│  Vorverarbeitung vor Ort, meldet als Text.        │
│                                                   │
│  nerve-system   CPU, RAM, Disk, Temperatur        │
│  nerve-log      Syslog                            │
│  nerve-net      Netzwerk                          │
│  nerve-file     Dateisystem-Events                │
│  nerve-audio    Mikrofon                          │
│  nerve-vision   Kamera                            │
│                                                   │
│  Lokale Modelle fuer Vorverarbeitung:             │
│  tract (ONNX), Ollama, llama-cpp-2                │
└──────────────────┬───────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────┐
│  Betriebssystem / Hardware                        │
│  Alpine Linux (Raspi), jedes Linux, macOS         │
└──────────────────────────────────────────────────┘
```

---

## Komponenten

### Core (`core.rs`)

Das Gehirn. Kapselt den rig-Agent, Preamble und History.
Subscribt auf `UserInput` Events, publiziert `ResponseToken`/`ResponseComplete`.
Baut den Agent bei jedem Input neu (so greifen Preamble-Aenderungen sofort).

Interne Hilfsmethoden:
- `simple_chat()` - Non-streaming, tool-freier LLM-Call fuer interne Aufgaben
- `history_as_text()` - History als lesbaren Text (fuer Kompaktifizierung)
- `compact_history()` - Fasst die History zusammen wenn Token-Budget erreicht
- `history_for_agent()` - Gibt nur den relevanten Teil der History zurueck (ab letztem Kompaktifizierungs-Marker)

### REPL (`repl.rs`)

Kommandozeile. Liest von stdin, publiziert `UserInput` Events.
Empfaengt Response-Events und gibt sie auf stdout aus.
Zeigt `SystemMessage` Events als `[info]` an.
Aendert den Prompt waehrend Kompaktifizierung (`[kompaktifiziere...]`).
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
Bei hoher Token-Nutzung (`compact_threshold` in Config) wird die History automatisch
zusammengefasst (`[KOMPAKTIFIZIERUNG]` Marker). Der Agent sieht nur den Teil ab dem
letzten Marker, die volle History bleibt in der Datei erhalten.

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
│       ├── events.rs      # Event-Typen (UserInput, Response, SystemMessage, Compacting, etc.)
│       ├── bus.rs          # Interner Event-Bus (broadcast)
│       ├── core.rs         # Gehirn (rig-Agent, History, Preamble)
│       ├── config.rs       # Agent-Config (Provider, Modell, Temperature)
│       ├── repl.rs         # Kommandozeile (stdin/stdout ueber Bus)
│       └── memory.rs       # MemoryTool (Tool-Use)
├── nerve/                 # aiux-nerve (Platzhalter, nicht implementiert)
│   ├── Cargo.toml
│   └── src/main.rs
├── home/                  # Agent-Home (wird deployed)
│   ├── config.toml        # System-Config (Provider, API-Keys)
│   ├── .system/           # Interne Prompts (compact-preamble.md)
│   ├── memory/
│   │   ├── soul.md        # Persoenlichkeit (geteilt, immer geladen)
│   │   └── user.md        # Wissen ueber den Menschen (geteilt)
│   └── roles/
│       ├── main/
│       │   ├── config.toml    # Modell, Temperature, Nerves
│       │   ├── role.md        # Rollenbeschreibung
│       │   └── memory/        # Rollen-Memory (context/*.md)
│       └── <weitere>/
│           ├── config.toml
│           ├── role.md
│           └── memory/
├── scripts/
│   ├── install.sh         # System-Installer
│   └── deploy.sh          # home/ auf Raspi deployen
├── docs/                  # PRD, Architektur, Roadmap
├── Cargo.toml             # Workspace
└── README.md
```

Laufzeit-Dateien (nicht im Repo, in .gitignore):
- `home/roles/*/conversation-*.json` - Tages-History pro Rolle

### Auf dem Zielsystem (Zielbild)

```
/home/claude/
├── config.toml                  # System-Config (Provider)
├── memory/
│   ├── soul.md                  # Persoenlichkeit (geteilt)
│   ├── user.md                  # Wissen ueber den Menschen (geteilt)
│   └── memory.db               # Langzeit-SQLite (geplant)
├── roles/
│   ├── main/
│   │   ├── config.toml          # Modell, Temperature
│   │   ├── role.md              # Rollenbeschreibung
│   │   ├── memory/              # Rollen-Context
│   │   └── conversation-*.json  # Tages-History
│   ├── sysadmin/
│   │   ├── config.toml          # Ollama, kleines Modell
│   │   ├── role.md
│   │   ├── memory/
│   │   └── conversation-*.json
│   └── .../
├── skills/                      # Expertise (geplant)
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
