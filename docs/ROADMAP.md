# AIUX - Roadmap

> Koerper-Architektur. Von innen nach aussen bauen.

---

## Was steht

Gebaut und lauffaehig:

- [x] Alpine Linux auf Raspi, SSH, Firewall
- [x] Cargo Workspace (core/ + nerve/)
- [x] Interner Event-Bus (tokio::broadcast)
- [x] Core als Modul (Events rein/raus, Preamble, History)
- [x] REPL als eigenes Modul (stdin/stdout ueber Bus)
- [x] Agent-Factory (Anthropic, Mistral, Ollama per Config)
- [x] Boot-Sequence: soul.md -> user.md -> context/*.md
- [x] Memory-Tools: SoulTool, UserTool, MemoryTool (read/write/edit/append/list)
- [x] Conversation-Persistenz (taegliche JSON-Rotation)
- [x] Slash-Commands (/quit, /exit, /clear)
- [x] Kompaktifizierung (History-Zusammenfassung bei Token-Budget)
- [x] SystemMessage Event (System-Info ueber Bus statt stderr)

---

## Phase A: Stabilisierung

> Den direkten Draht zum Grosshirn sauber machen.

- [x] Preamble bei Aenderung neu laden (dirty flag mit Arc<AtomicBool>)
- [x] History-Limit: Kompaktifizierung bei Token-Budget (compact_threshold)
- [x] Fehlerbehandlung: API-Fehler als SystemMessage, nicht in History
- [x] REPL: Prompt nach Fehlern/Kompaktifizierung korrekt (Compacting/Compacted Events)
- [x] Unit-Tests fuer Phase A (43 Tests)

---

## Phase B: Verzeichnisstruktur umbauen

> Config und Memory fuer Rollen vorbereiten.

```
home/
├── .system/
│   ├── config.toml          # Flach: provider, model, temperature, ...
│   └── compact-preamble.md  # Kompaktifizierungs-Prompt
├── memory/
│   ├── soul.md              # Identitaet
│   ├── user.md              # Mensch
│   ├── context/             # Arbeits-Memory
│   └── conversations/       # Tages-History
├── skills/                  # Platzhalter
└── tools/                   # Platzhalter
```

- [x] Config nach .system/config.toml verschoben (flaches Format, kein [agents.main] mehr)
- [x] Conversations nach memory/conversations/ verschoben
- [x] Code angepasst: Config direkt statt AgentConfig + HashMap
- [x] .gitignore fuer **/conversations/
- [x] journal/ Platzhalter entfernt
- [x] .env.example ergaenzt
- [x] Unit-Tests fuer Phase B (11 Tests, 54 gesamt)

---

## Phase C: Hippocampus (Memory-System erweitern)

> Das Gehirn braucht spezialisiertes Gedaechtnis und bewusstes Aufschreiben.

Drei spezialisierte Tools statt einem generischen MemoryTool.
Hippocampus-Call bei Kompaktifizierung und Memory-Flush (/clear, /quit)
destilliert Wissen automatisch in die passenden Dateien.

- [x] Tools-Modul: SoulTool, UserTool, MemoryTool (read/write/edit/append)
- [x] Konfigurierbare Tool-Beschreibungen aus home/.system/tool-*.md
- [x] ToolCall Event: Tool-Aufrufe als SystemMessage sichtbar
- [x] Hippocampus-Call: Kompaktifizierung mit Tools (destilliert Wissen)
- [x] Memory-Flush bei /clear und /quit (sichert Wissen vor Loeschen)
- [x] History-Reduktion: nach Kompaktifizierung nur letzte 5 Messages behalten
- [x] Unit-Tests fuer Phase C (87 Tests gesamt)

---

## Phase D: Nervensystem

> MQTT, Brainstem, erste Nerves. Das System bekommt Sinne.

### D.1: MQTT-Grundlagen

Mosquitto als externer Bus. Bridge im Core.

- [x] Mosquitto lokal einrichten (Entwicklung)
- [x] MQTT-Bridge im Core: rumqttc Client, subscribe auf `aiux/nerve/#`
- [x] Bridge als optionaler Task in main.rs (nur wenn mqtt_host konfiguriert)
- [x] Config erweitern: mqtt_host, mqtt_port (optional)
- [x] NerveSignal Event-Typ im internen Bus
- [x] Bus-to-MQTT: ResponseComplete, SystemMessage, ToolCall nach aiux/cortex/*
- [x] Tests fuer Bridge (JSON-Parsing, Event-Mapping, Event-Filterung) - 5 Tests, 77 gesamt

### D.2: Brainstem (Sandbox + Heartbeat)

Laufzeitumgebung fuer Nerve-Verarbeitung. Heartbeat fuer Lebenszeichen
und Rhythmen. Kann vom Cortex als Reminder genutzt werden.

- [ ] MQTT Message-Schema definieren: Pflichtfelder, Validierung gegen channels.toml
- [ ] Brainstem-Modul im Core: Registry, Nerve-Discovery, interpret-Ausfuehrung
- [ ] Nerve-Verzeichnisse scannen: manifest.toml + channels.toml lesen
- [ ] Registry: welche Nerves sind aktiv, welche Channels existieren
- [ ] rhai-Engine einbetten (sandboxed)
- [ ] Boot-Scan: nerves/*/ beim Start laden
- [ ] Heartbeat: pruefen ob Nerves noch leben (Watchdog)
- [ ] Heartbeat: Cortex regelmaessig triggern (Puls, Atem, Tagesrueckblick)
- [ ] Heartbeat: Cortex kann Reminder setzen ("erinnere mich in 1h")
- [ ] Tests fuer Brainstem

### D.3: nerve-file

Erster Nerve. Beobachtet Dateiaenderungen in home/.
Ist gleichzeitig das Discovery-System fuer neue Nerves.

- [ ] nerve-file Binary (notify crate, inotify auf Linux)
- [ ] MQTT: publish auf aiux/nerve/file/changed
- [ ] Nerve-Verzeichnis: manifest.toml, channels.toml, interpret.*
- [ ] Brainstem-Verarbeitung: Config-Reload, Preamble-Reload, Nerve-Discovery
- [ ] Tests fuer nerve-file

### D.4: nerve-system

Zweiter Nerve. Ueberwacht CPU, RAM, Disk, Temperatur.
Zeigt das Thalamus-Pattern: Nerve filtert selbst, meldet nur Anomalien.

- [ ] nerve-system Binary (oder Telegraf mit MQTT-Output)
- [ ] MQTT: publish auf aiux/nerve/system/*
- [ ] Nerve-Verzeichnis: manifest.toml, channels.toml, interpret.*
- [ ] Brainstem-Verarbeitung per rhai (Schwellwerte, Trends)
- [ ] Tests fuer nerve-system

---

## Phase E: Rollen

> Parallele Agent-Instanzen mit eigener Config und eigenem Memory.

- [ ] Rollen-Verzeichnisstruktur (roles/<name>/)
- [ ] Rollen-Config laden (role.md, config.toml, eigener Memory)
- [ ] Preamble pro Rolle: soul + user + role + role-memory
- [ ] Mehrere Core-Instanzen parallel auf dem Bus
- [ ] REPL: /role zum Wechseln, /roles zum Auflisten
- [ ] Prompt zeigt aktive Rolle: main>, assistent>, etc.
- [ ] Kommunikation zwischen Rollen ueber Bus
- [ ] Unit-Tests fuer Phase E

---

## Phase F: Chat-Gateway

> Den direkten Zugang zum Grosshirn ueber richtige Kanaele.

Chat ist kein Nerve - es ist direktes Gespraech. Das Gateway
ersetzt die REPL fuer externe Kommunikation.

- [ ] Gateway-Trait: Nachricht empfangen, Antwort senden
- [ ] Telegram-Gateway (erstes echtes Gateway)
- [ ] Mehrzeilen-Input, Anhaenge (Bilder -> als Pfad/Beschreibung)
- [ ] MessageTool: Agent kann aktiv Nachrichten senden
- [ ] Unit-Tests fuer Phase F

---

## Phase G: Weitere Nerves

> Das System spueren und die Umwelt wahrnehmen.

- [ ] nerve-log: Syslog beobachten, Anomalien erkennen
- [ ] nerve-net: Netzwerk-Status, Erreichbarkeit
- [ ] Brainstem-LLM: kleines Modell fuer sprachliche Interpretation
- [ ] Externe Modelle/APIs fuer Nerves (Ollama, ONNX)
- [ ] Unit-Tests fuer Phase G

---

## Fernziele

Kein Zeitplan, keine Reihenfolge. Ideen fuer spaeter:

- Langzeit-Memory: SQLite + RAG (rig-sqlite, semantische Suche)
- Skills als Markdown (Expertise die geladen wird)
- Shell-Tool (Agent kann Befehle ausfuehren)
- Lokale Inference auf Raspi (tract/ONNX)
- Vision-Nerve (Kamera + lokales Vision-Modell)
- Audio-Nerve (Mikrofon + Speech-to-Text)
- Journal (Lerntagebuch, Reflexion)
- Web-Gateway

---

## Entschiedene Architektur-Fragen

Frueher offen, jetzt beantwortet:

| Frage | Antwort |
|-------|---------|
| Memory am Bus oder im Core? | Hippocampus hoert auf dem Bus mit (Phase C). MemoryTool bleibt im Core. |
| Wie werden Nerves angebunden? | Eigene Prozesse, MQTT nach aussen. Nerve liefert Was/Wie/Wohin. |
| Brainstem = Scheduler? | Brainstem = Sandbox + Heartbeat. Nerve-Verarbeitung (rhai, LLM, APIs) UND Taktgeber (Watchdog, Rhythmen, Reminder). |
| Nerve-Discovery? | file-watcher Nerve beobachtet nerves/. Bootstrap beim Boot durch Brainstem-Scan. |
| Chat = Nerve? | Nein. Chat ist direkter Zugang zum Cortex. Gateway, kein Nerve. |
| Cortex bekommt Nerve-Events? | Nicht automatisch. Cortex ist Superuser, kann auf MQTT mitlesen wenn er will. |
| Nerve-Format? | Verzeichnis unter nerves/ mit manifest.toml, channels.toml, interpret.*, binary. |
| Scriptsprache? | rhai (sandboxed, eingebettet, fertig). Keine eigene Scriptsprache. |
| Config wo? | System-Config in home/.system/config.toml. Rollen-Config spaeter in roles/<name>/config.toml. |

---

*Letzte Aktualisierung: 2026-03-02 (Phase D neu geschnitten: Nervensystem)*
