# AIUX - Roadmap

> Architektur zuerst. Fundament vor Features.

---

## Was steht (Phase 1-4.3)

Gebaut und lauffaehig:

- Alpine Linux auf Raspi, SSH, Firewall
- Cargo Workspace (core/ + nerve/)
- REPL mit Streaming-Ausgabe
- Boot-Sequence: soul.md -> user.md -> context/*.md
- MemoryTool (write/read/list auf context/)
- Conversation-Persistenz (taegliche JSON-Rotation)

---

## Code-Refinement: Basis-Architektur

> Der Code muss die Architektur widerspiegeln.
> Aktuell ist alles ein Skript in main.rs. Das wird zur
> Event-Driven Architecture umgebaut - dem Leitprinzip aus ARCHITECTURE.md.

### Schicht 1: Interner Event-Bus

Das Nervensystem. Ohne das geht nichts.

- [x] Event-Typen definieren (UserInput, ResponseToken, ResponseComplete, ClearHistory, Shutdown)
- [x] Pub/Sub Mechanismus (tokio::sync::broadcast)
- [x] Module koennen Events publizieren und subscriben

### Schicht 2: Core als Modul

Der Core wird ein Struct das auf Events hoert und Events produziert.
Die REPL wird ein eigenes Modul das InputEvents auf den Bus legt.

- [x] Core Struct (haelt Agent-State: Config, Preamble, History, Tools)
- [x] Core subscribt auf UserInput, produziert ResponseToken/ResponseComplete
- [x] REPL als eigenstaendiges Modul (stdin -> UserInput, ResponseToken -> stdout)
- [x] Boot-Sequence in den Core
- [x] Agent-Factory: Provider per Config (Anthropic, Mistral, Ollama)

### Danach: offen

Memory-Anbindung, Tools, Nerves, Scheduler - wird diskutiert wenn
Schicht 1 und 2 stehen. Architektur-Entscheidungen die noch offen sind:

- Gehoert Memory an den Bus oder ist es Teil des Core? (Kopf vs. Koerper)
- Tool-Calls sind LLM-intern, nicht event-basiert - Konsequenzen?
- Wie werden Nerves angebunden? (interner Bus -> MQTT Bridge -> externer Bus?)
- Scheduler: eigenes Modul oder Teil des Core?

---

## Hinweise

- Die alte Roadmap (Phase 1-9) wurde entfernt. Sie war feature-getrieben,
  nicht architektur-getrieben. Features kommen wenn die Basis steht.
- Tool-Calls (LLM Tool-Use) sind ein internes Protokoll des Agenten.
  Sie laufen nicht ueber den Event-Bus. Das widerspricht dem Embodied-Prinzip,
  ist aber eine technische Realitaet der LLM-Architektur.

---

*Letzte Aktualisierung: 2026-03-01*
