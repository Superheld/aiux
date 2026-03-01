# Event-Bus Architektur

> Alles kommt als Event rein, alles geht als Event raus.

## Bus

- Interner `tokio::sync::broadcast` Channel (kein Netzwerk)
- Ein Sender, beliebig viele Empfaenger
- Jedes Modul holt sich mit `bus.subscribe()` seinen eigenen Receiver
- Kapazitaet: 64 Events (konfiguriert in `main.rs`)

## Events

Definiert in `core/src/events.rs`:

| Event | Payload | Bedeutung |
|-------|---------|-----------|
| `UserInput` | `text: String` | User hat etwas eingegeben |
| `ResponseToken` | `text: String` | Ein Token aus der LLM-Antwort (Streaming) |
| `ResponseComplete` | `full_text: String` | Antwort fertig, ganzer Text |
| `ClearHistory` | - | History loeschen |
| `Shutdown` | - | Sauber herunterfahren |

## Teilnehmer

### Core (`core.rs`)

Subscribt auf: `UserInput`, `ClearHistory`, `Shutdown`
Publiziert: `ResponseToken`, `ResponseComplete`

Das Gehirn. Empfaengt Eingaben, fragt den LLM-Agent, streamt die Antwort
als Token-Events zurueck.

### REPL Input (`repl.rs` - run_input)

Subscribt auf: nichts (liest von stdin)
Publiziert: `UserInput`, `ClearHistory`, `Shutdown`

Laeuft in einem eigenen Thread (`spawn_blocking`) weil stdin blockiert.

### REPL Output (`repl.rs` - run_output)

Subscribt auf: `UserInput`, `ResponseToken`, `ResponseComplete`, `Shutdown`
Publiziert: nichts (schreibt auf stdout)

Gibt Tokens live auf stdout aus. Reagiert auf `UserInput` um das
"AIUX: " Label zu drucken.

## Event-Fluss

```
REPL Input          Core                REPL Output
(stdin)              |                   (stdout)
   |                 |                      |
   |-- UserInput --->|                      |
   |-- UserInput ---|---------------------->|  "AIUX: "
   |                 |                      |
   |                 |-- ResponseToken ---->|  Token drucken
   |                 |-- ResponseToken ---->|  Token drucken
   |                 |-- ResponseComplete ->|  Zeilenumbruch
   |                 |                      |
```

## Regeln

1. **Jedes Modul hoert nur auf Events die es interessieren.** Unbekannte Events ignorieren.
2. **Publisher wissen nicht wer zuhoert.** Das ist Absicht - lose Kopplung.
3. **Kein Request-Response.** Events sind fire-and-forget. Antworten kommen als eigene Events.
4. **Reihenfolge:** Token-Events kommen in Reihenfolge, aber zwischen verschiedenen
   Event-Typen gibt es keine Garantie.

## Agent-Factory und der Bus

Der Bus loest das Provider-Problem: verschiedene LLM-Provider (Anthropic, Mistral,
Ollama) erzeugen verschiedene Rust-Typen. Aber jeder Agent lebt in seinem eigenen
Task hinter dem Bus. Nach aussen gibt es nur Events - der Provider-Typ ist intern.

```
Config: provider=anthropic     Config: provider=mistral
        |                              |
        v                              v
   Agent-Factory                  Agent-Factory
        |                              |
        v                              v
  Agent<Anthropic>               Agent<OpenAI>
  (lebt in Task)                 (lebt in Task)
        |                              |
        +---------> Bus <--------------+
                     |
              Nur Events hier
              (Typ ist weg)
```

Die Factory liest die Config und baut den richtigen Agent. Der Agent-Code
(Preamble, Tools, Chat, Streaming) ist bei allen Providern identisch -
nur die Client-Erstellung unterscheidet sich. Siehe ARCHITECTURE.md fuer Details.

## Spaeter

Neue Module schliessen sich einfach an:

- **Nerve** (Sensor): publiziert z.B. `FileChanged`, `CronTick`
- **Gateway** (HTTP/WebSocket): ersetzt REPL, publiziert `UserInput`, hoert auf Response-Events
- **Scheduler**: publiziert zeitgesteuerte Events
- **Sub-Agents**: eigener Agent pro Config-Eintrag, eigener Provider moeglich
