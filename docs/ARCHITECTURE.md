# AIUX - Architektur

> Koerper-Architektur: Ein System dessen Gehirn ein Sprachmodell ist.

---

## Ueberblick

```mermaid
block-beta
  columns 1
  block:gehirn["Gehirn (aiux-core)"]
    columns 3
    A["Cortex\nGrosshirn/LLM"] B["Hippocampus\nMemory"] C["Brainstem\nReflexe/Sandbox"]
  end
  block:bus["Nervensystem"]
    columns 2
    D["tokio::broadcast\n(intern)"] E["MQTT/Mosquitto\n(extern)"]
  end
  block:nerves["Nerves (Fuehler)"]
    columns 4
    F["file"] G["system"] H["log"] I["..."]
  end
  OS["Betriebssystem / Hardware"]

  gehirn --> bus
  bus --> nerves
  nerves --> OS
```

| Komponente | Biologisch | Aufgabe |
|------------|-----------|---------|
| **Cortex** | Grosshirn | LLM. Denkt, spricht, entscheidet. |
| **Hippocampus** | Gedaechtnis | Destilliert Wissen in Memory-Dateien. Vom Code gesteuert, nicht vom LLM. |
| **Brainstem** | Hirnstamm | Sandbox fuer Nerve-Verarbeitung + Heartbeat. Keine eigene Logik. |
| **Nerves** | Sinnesorgane | Eigene Prozesse, passive Sensoren, kommunizieren ueber MQTT. |
| **Tools** | Haende | Aktive Handlungen. Cortex entscheidet bewusst. |
| **Chat** | Gespraech | Direkter Zugang zum Cortex (REPL, spaeter Gateway). Kein Nerve. |

---

## Kommunikation

Zwei Bus-Systeme, verbunden durch die Bridge:

```mermaid
graph TB
  subgraph core["Core-Prozess"]
    REPL <-->|"Events (Rust enum)"| BUS["tokio::broadcast"]
    BUS <--> Cortex
    BUS <--> Hippocampus
    BUS <--> Brainstem
    BUS <--> Bridge["MQTT-Bridge"]
  end
  Bridge <-->|JSON| Mosquitto
  Mosquitto <--> NF["nerve-file"]
  Mosquitto <--> NS["nerve-system"]
  Mosquitto <--> NX["nerve-..."]
```

**Interner Bus** (`tokio::broadcast`): In-process, typsicher, zero-copy. Fuer REPL ↔ Core.
Ohne externe Dependencies — AIUX laeuft auch ohne MQTT als reiner Chat.

**Externer Bus** (MQTT/Mosquitto): Prozessuebergreifend, sprachunabhaengig. Fuer Nerves.
Bridge uebersetzt selektiv — der Cortex weiss nicht dass MQTT existiert.

### Events

| Event | Richtung | MQTT |
|-------|----------|------|
| `UserInput` | REPL → Core | nein |
| `ResponseToken` | Core → REPL | nein |
| `ResponseComplete` | Core → REPL | → `aiux/cortex/response` |
| `SystemMessage` | Core → REPL | → `aiux/cortex/system` |
| `ToolCall` | Core → REPL | → `aiux/cortex/toolcall` |
| `NerveSignal` | Bridge → Core | ← `aiux/nerve/#` |
| `Compacting` / `Compacted` | Core → REPL | nein |
| `ClearHistory` | REPL → Core | nein |
| `Shutdown` | REPL → alle | nein |

### MQTT Topics

```
aiux/
├── nerve/                  # Nerves → Bridge (incoming)
│   ├── register            # Nerve meldet sich an
│   └── <name>/<event>      # Nerve-spezifische Events
├── cortex/                 # Bridge → aussen (outgoing)
│   ├── response            # LLM-Antworten
│   ├── system              # System-Nachrichten
│   └── toolcall            # Tool-Aufrufe
└── brainstem/              # Verarbeitete Ergebnisse (D.2)
    └── <name>/
```

### MQTT Message-Schema

Jede Nachricht auf `aiux/nerve/<name>/<event>` **muss** dieses Format haben:

```json
{
  "ts": "2026-03-02T14:30:00Z",
  "source": "nerve/file",
  "event": "changed",
  "data": { }
}
```

| Feld | Typ | Pflicht | Beschreibung |
|------|-----|---------|-------------|
| `ts` | String (ISO 8601) | ja | Zeitstempel des Events |
| `source` | String | ja | Absender, z.B. `"nerve/file"` |
| `event` | String | ja | Was passiert ist, z.B. `"changed"` |
| `data` | Object | nein | Nerve-spezifische Daten (frei) |

Die Bridge validiert Pflichtfelder — fehlende Felder oder kein JSON → Warnung, Message verworfen.

---

## Agents

```mermaid
flowchart LR
  subgraph Cortex
    direction TB
    C1[soul + user + shortterm] --> C2[rig-Agent]
    C2 --> C3[Streaming + Tools]
  end
  subgraph Hippocampus
    direction TB
    H1[compact-preamble.md] --> H2[rig-Agent]
    H2 --> H3[Memory-Flush]
  end
  Bus -->|UserInput| Cortex
  Cortex -->|"Rust-Call\n(kein LLM-Entscheid)"| Hippocampus
```

| Agent | Tools | History | Ausloeser |
|-------|-------|---------|-----------|
| **Cortex** | soul, user, memory | ja, Streaming | UserInput via Bus |
| **Hippocampus** | soul, user, memory | nein | Schwellwert, /clear, /quit |

### Agent-Factory

Provider-Typ wird intern aufgeloest — nach aussen nur Events:

```mermaid
flowchart LR
  Config[".system/config.toml"] --> Factory
  Factory -->|anthropic| A[Client]
  Factory -->|mistral| B[Client]
  Factory -->|ollama| C[Client]
  A & B & C --> Agent["Agent + Preamble + Tools"]
  Agent --> Bus
```

---

## Memory

```mermaid
flowchart TD
  subgraph Preamble["Preamble (bei jedem LLM-Call)"]
    soul.md --> P[" "]
    user.md --> P
    shortterm.md --> P
  end
  P --> Cortex
  Conv["conversations/\nYYYY-MM-DD.json"] --> Cortex
  Config[".system/config.toml"] --> Cortex
```

| Typ | Format | Lebensdauer |
|-----|--------|-------------|
| **Kurzzeit** | shortterm.md | Permanent, Agent verwaltet (MemoryTool) |
| **Konversation** | conversation-YYYY-MM-DD.json | Pro Tag |
| **Langzeit** | SQLite + RAG (geplant) | Permanent, durchsuchbar |

Kompaktifizierung bei `compact_threshold`: History zusammenfassen,
Wissen destillieren, `[KOMPAKTIFIZIERUNG]`-Marker setzen.

---

## Nerve-System

Ein Nerve = eigenstaendiger Prozess + Verzeichnis unter `nerves/`:

```
nerves/file-watcher/
├── manifest.toml       # Pflicht: Name, Binary, Beschreibung
├── channels.toml       # Pflicht: MQTT-Topics + Schema
├── interpret.*         # Verarbeitung fuer Brainstem (rhai/md/...)
└── nerve-file          # Der Sensor (Binary/Script, beliebige Sprache)
```

**Nerve-Protokoll:** Registrieren auf `aiux/nerve/register`, dann Events auf eigenen Channels publizieren. Schema-Validierung kommt mit dem Brainstem (D.2).

---

## Brainstem

Sandbox im Core-Prozess. Keine eigene Logik — fuehrt aus was Nerves mitliefern.

```mermaid
flowchart LR
  MQTT["Nerve-Event\n(MQTT)"] --> BS["Brainstem"]
  BS -->|"sucht"| I["interpret.*\ndes Nerve"]
  I --> R["rhai / LLM / API"]
  R -->|"Weiterleitungsregeln\ndes Nerve"| Out["MQTT / Cortex"]
```

| Aufgabe | Beschreibung |
|---------|-------------|
| Verarbeitung | interpret.* aus Nerve-Verzeichnis ausfuehren |
| Registry | Welche Nerves aktiv, welche Channels |
| Discovery | Neues Nerve-Verzeichnis → scannen, laden, starten |
| Heartbeat | Watchdog, Rhythmen (Puls/Atem), Reminder |

Discovery: Boot-Scan von `nerves/*/manifest.toml`, danach uebernimmt file-watcher.

---

## Verzeichnisstruktur

```
aiux/
├── core/src/
│   ├── main.rs              # Verdrahtung
│   ├── config.rs            # Config aus .system/config.toml
│   ├── history.rs           # Conversation-Persistenz
│   ├── home.rs              # home/-Verzeichnis finden
│   ├── repl.rs              # Kommandozeile
│   ├── mqtt.rs              # MQTT-Bridge
│   ├── agent/{cortex,hippocampus}.rs
│   ├── bus/{mod,events}.rs
│   └── tools/{soul,user,memory}.rs
├── nerve/                   # Nerve-Binaries (Workspace-Crate)
├── home/
│   ├── .system/             # Config + System-Prompts
│   ├── memory/              # soul.md, user.md, shortterm.md, conversations/
│   ├── nerves/              # Nerve-Verzeichnisse
│   ├── skills/              # Platzhalter
│   └── tools/               # Platzhalter
└── docs/
```

Zielsystem (Raspi): `/home/claude/` mit gleicher Struktur.

---

## Tech-Stack

| Crate | Zweck |
|-------|-------|
| **rig-core** | LLM (Multi-Provider, Streaming, Tool-Use) |
| **tokio** | Async Runtime |
| **rumqttc** | MQTT Client |
| **serde** / **serde_json** | Serialisierung |
| **schemars** | JSON Schema (Tool-Definitionen) |
| **chrono** | Datum (History-Rotation) |
| **thiserror** / **anyhow** | Error-Handling |
| **futures** | Stream-Verarbeitung |
| **dotenvy** | .env laden |
| **toml** | Config parsen |

Geplant: **notify** (Filesystem-Watcher), **rhai** (Brainstem-Sandbox),
**rig-sqlite** (RAG), **tokio-cron-scheduler** (Rhythmen), **tract-onnx** (lokale Inference).

---

## Offene Fragen

- Weiterleitungsregeln: Wie definiert ein Nerve wohin Ergebnisse gehen?
- Brainstem-LLM: Welches kleine Modell, wie angebunden?
- Dynamische Tools: Nerves liefern dem Cortex Tools (rig-core `ToolDyn`)?
- Heartbeat-Details: Intervalle, Reminder-API, Watchdog-Timeouts

---

*Letzte Aktualisierung: 2026-03-02*
