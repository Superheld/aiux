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
- [x] MemoryTool (write/read/list auf context/)
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

---

## Phase B: Verzeichnisstruktur umbauen

> Config und Memory fuer Rollen vorbereiten.

```
home/
├── config.toml              # System: Provider, API-Keys
├── memory/
│   ├── soul.md              # Identitaet (geteilt)
│   ├── user.md              # Mensch (geteilt)
│   ├── context/             # Main's Arbeits-Memory
│   └── conversations/       # Main's Tages-History
└── roles/                   # Rollen (spaeter)
```

- [ ] Conversations nach memory/conversations/ verschieben
- [ ] Config aufteilen: System (Provider) vs. Main (Modell, Temperature)
- [ ] Code anpassen: Config und Pfade auf neue Struktur
- [ ] .gitignore fuer conversations/

---

## Phase C: Hippocampus (automatisches Memory)

> Das Gehirn soll sich Dinge merken ohne bewusst entscheiden zu muessen.

Ein kleiner Prozess der auf dem Bus mithoert und wichtige Dinge
automatisch speichert. Kein LLM-Call pro Nachricht noetig -
regelbasiert oder mit kleinem lokalem Modell.

- [ ] Hippocampus-Modul: hoert ResponseComplete Events mit
- [ ] Erkennung relevanter Inhalte (Keywords, Muster)
- [ ] Automatisches Schreiben in context/
- [ ] MemoryTool bleibt fuer bewusstes Aufschreiben

---

## Phase D: Erster Nerve

> Den ersten Fuehler zur Umwelt anschliessen.

`nerve-file` als einfachster Nerve: Beobachtet Dateiaenderungen
in home/ und meldet sie auf den Bus. Damit greift auch der
Preamble-Reload automatisch.

- [ ] nerve-file: inotify/notify auf home/memory/ und home/config.toml
- [ ] Event-Typ: FileChanged { path, change_type }
- [ ] Core reagiert auf Config-Aenderung: Agent neu bauen
- [ ] Core reagiert auf Preamble-Aenderung: Preamble neu laden
- [ ] MQTT-Bridge: interner Bus <-> Mosquitto (fuer externe Nerves)

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

---

## Phase F: Chat-Gateway

> Den direkten Zugang zum Grosshirn ueber richtige Kanaele.

Chat ist kein Nerve - es ist direktes Gespraech. Das Gateway
ersetzt die REPL fuer externe Kommunikation.

- [ ] Gateway-Trait: Nachricht empfangen, Antwort senden
- [ ] Telegram-Gateway (erstes echtes Gateway)
- [ ] Mehrzeilen-Input, Anhaenge (Bilder -> als Pfad/Beschreibung)
- [ ] MessageTool: Agent kann aktiv Nachrichten senden

---

## Phase G: Weitere Nerves

> Das System spueren und die Umwelt wahrnehmen.

Jeder Nerve hat eigene Vorverarbeitung (verteilter Thalamus).
Alles kommt als Text beim Core an.

- [ ] nerve-system: CPU, RAM, Disk, Temperatur
- [ ] nerve-log: Syslog beobachten, Anomalien erkennen
- [ ] nerve-net: Netzwerk-Status, Erreichbarkeit
- [ ] Nerves mit lokalem Modell fuer Vorverarbeitung (Ollama)

---

## Phase H: Hirnstamm (Scheduler)

> Rhythmen die ohne bewusstes Denken laufen.

- [ ] Puls (5 Min): Bin ich okay? Kurzer Selbst-Check
- [ ] Atem (1h): Was ist gerade los? Zusammenfassung
- [ ] Tagesrueckblick: Was habe ich heute gelernt?
- [ ] Events auf den Bus, Core entscheidet ob Aktion noetig

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
| Wie werden Nerves angebunden? | Eigene Prozesse, MQTT nach aussen, eigene Vorverarbeitung. |
| Scheduler: eigenes Modul? | Ja, Hirnstamm. Eigenes Modul, Events auf den Bus. |
| Chat = Nerve? | Nein. Chat ist direkter Zugang zum Grosshirn. Gateway, kein Nerve. |
| Filter/Thalamus zentral? | Nein. Verteilt, jeder Nerve filtert selbst. |
| Config wo? | System-Config in home/config.toml. Rollen-Config in roles/<name>/config.toml. |

---

*Letzte Aktualisierung: 2026-03-01*
