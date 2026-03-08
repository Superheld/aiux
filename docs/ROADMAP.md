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
- [x] Tool-Beschreibungen fest im Code (nicht konfigurierbar)
- [x] ToolCall Event: Tool-Aufrufe als SystemMessage sichtbar
- [x] Hippocampus-Call: Kompaktifizierung mit Tools (destilliert Wissen)
- [x] Memory-Flush bei /clear und /quit (sichert Wissen vor Loeschen)
- [x] History-Reduktion: nach Kompaktifizierung nur letzte 5 Messages behalten
- [x] Unit-Tests fuer Phase C (87 Tests gesamt)

---

## Phase D: Nervensystem ✓

> MQTT, Brainstem, Nerves. Das System bekommt Sinne.

### D.1: MQTT-Grundlagen ✓

- [x] Mosquitto lokal einrichten (Entwicklung)
- [x] MQTT-Bridge im Core: rumqttc Client, subscribe auf `aiux/nerve/#`
- [x] Bridge als optionaler Task in main.rs (nur wenn mqtt_host konfiguriert)
- [x] Config erweitern: mqtt_host, mqtt_port (optional)
- [x] NerveSignal Event-Typ im internen Bus
- [x] Bus-to-MQTT: ResponseComplete, SystemMessage, ToolCall nach aiux/cortex/*
- [x] Tests (5 Tests)

### D.2: Brainstem ✓

- [x] MQTT Message-Schema definieren: Pflichtfelder, Validierung
- [x] Brainstem-Modul: Registry, Self-Registration, interpret-Ausfuehrung
- [x] rhai-Engine einbetten (sandboxed, parse_json Hilfsfunktion)
- [x] Heartbeat: Scheduler mit Cron-Jobs und Einmal-Timern
- [x] SchedulerTool: Neocortex kann Reminder setzen (set, cron, cancel, list)
- [x] Tests (38 Tests)

### D.3: nerve-system ✓

- [x] Self-Registration: Nerves melden sich per MQTT beim Brainstem an
- [x] Registration-Schema standardisiert (name, version, description, channels, home)
- [x] nerve/shared: Gemeinsamer Code (MQTT, Registration)
- [x] nerve/system: System-Monitor (CPU, RAM, Disk, Temperatur)
- [x] interpret.rhai: Schwellwert-Filterung (CPU >80%, RAM >90%, Temp >70°C)
- [x] Nerve-Launcher: Brainstem startet Binaries aus manifest.toml beim Boot
- [x] Shutdown: Brainstem beendet alle Child-Prozesse sauber
- [x] Tests (3 Tests nerve-shared, 1 Test nerve-system, 6 Launcher-Tests)

**Gesamt nach Phase D: 125 Tests (core) + 3 (nerve-shared) + 1 (nerve-system) = 129**

---

## Phase E: Haende (Shell-Tool) ✓

> Der Agent bekommt Zugriff auf sein System. Unboxed.

Der Neocortex kann Shell-Befehle ausfuehren um seinen Koerper kennenzulernen,
zu pflegen und Probleme selbst zu loesen.

- [x] ShellTool: Neocortex kann Shell-Befehle ausfuehren
- [x] Sicherheit: Whitelist in `[shell]`-Config, Segment-fuer-Segment-Pruefung
- [x] Ausgabe als Text an den Neocortex zurueck (stdout/stderr, 4000 Zeichen Limit)
- [x] Timeout fuer lang laufende Befehle (konfigurierbar, Default 30s)
- [x] Tests fuer ShellTool (22 Tests)
- [x] Rename: cortex → neocortex (Code, Config, MQTT-Topics, Docs)
- [x] Shell-Config als eigene TOML-Section `[shell]` (Tool, kein Agent)
- [x] Tool-Description mit Handlungsanweisungen und Triggern

**Gesamt nach Phase E: 147 Tests (core) + 2 (nerve-shared) + 1 (nerve-system) = 150**

---

## Phase E.5: Deploy-Pipeline ✓

> Jede Iteration auf den Raspi bringen. Ohne Friction.

Branching: `dev` (Arbeit, Tests) → `main` (Release).
GitHub Actions baut bei jedem Push auf main automatisch einen Rolling-Release (`latest`).
Install-Skript holt den neuesten Stand von GitHub.

- [x] GitHub Actions: Tests auf `dev` + `main`
- [x] GitHub Actions: Cross-Build (aarch64-musl + x86_64-musl) bei Push auf `main`
- [x] Auto-Release: Rolling `latest` Release mit Binaries
- [x] `install.sh`: Download + Install von GitHub Releases
- [x] Erstinstallation: Home-Verzeichnis anlegen (config, memory, nerves)
- [x] Update: Nur Binaries ersetzen, home/ nie ueberschreiben
- [x] README.md + CLAUDE.md auf Englisch
- [ ] Erster erfolgreicher Deploy auf den Raspi

---

## Phase F: Rollen

> Parallele Agent-Instanzen mit eigener Config und eigenem Memory.

- [ ] Rollen-Verzeichnisstruktur (roles/<name>/)
- [ ] Rollen-Config laden (role.md, config.toml, eigener Memory)
- [ ] Preamble pro Rolle: soul + user + role + role-memory
- [ ] Mehrere Core-Instanzen parallel auf dem Bus
- [ ] REPL: /role zum Wechseln, /roles zum Auflisten
- [ ] Prompt zeigt aktive Rolle: main>, assistent>, etc.
- [ ] Kommunikation zwischen Rollen ueber Bus
- [ ] Tests

---

## Phase G: Chat-Gateway

> Den direkten Zugang zum Grosshirn ueber richtige Kanaele.

Chat ist kein Nerve - es ist direktes Gespraech. Das Gateway
ersetzt die REPL fuer externe Kommunikation.

- [ ] Gateway-Trait: Nachricht empfangen, Antwort senden
- [ ] Telegram-Gateway (erstes echtes Gateway)
- [ ] Mehrzeilen-Input, Anhaenge (Bilder -> als Pfad/Beschreibung)
- [ ] MessageTool: Agent kann aktiv Nachrichten senden
- [ ] Tests

---

## Phase H: Weitere Nerves

> Das System spueren und die Umwelt wahrnehmen.

- [ ] nerve-log: Syslog beobachten, Anomalien erkennen
- [ ] nerve-net: Netzwerk-Status, Erreichbarkeit
- [ ] nerve-file: Dateiaenderungen beobachten (notify/inotify)
- [ ] Brainstem-LLM: kleines Modell fuer sprachliche Interpretation
- [ ] Tests

---

## Fernziele

Kein Zeitplan, keine Reihenfolge. Ideen fuer spaeter:

- Langzeit-Memory: SQLite + RAG (rig-sqlite, semantische Suche)
- Skills als Markdown (Expertise die geladen wird)
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
| Wie werden Nerves angebunden? | Eigene Prozesse, MQTT nach aussen. Self-Registration beim Start. |
| Brainstem = Scheduler? | Brainstem = Sandbox + Heartbeat + Nerve-Launcher. |
| Nerve-Discovery? | Self-Registration per MQTT. Brainstem startet Binaries aus manifest.toml. |
| Chat = Nerve? | Nein. Chat ist direkter Zugang zum Neocortex. Gateway, kein Nerve. |
| Neocortex bekommt Nerve-Events? | Nicht automatisch. interpret.rhai entscheidet ob/was weitergeleitet wird. |
| Nerve-Format? | Verzeichnis unter nerves/ mit manifest.toml (binary) + interpret.rhai (optional). Alles andere per Self-Registration. |
| Scriptsprache? | rhai (sandboxed, eingebettet, fertig). Keine eigene Scriptsprache. |
| Config wo? | System-Config in home/.system/config.toml. Rollen-Config spaeter in roles/<name>/config.toml. |

---

*Letzte Aktualisierung: 2026-03-04 (Phase E abgeschlossen, Rename cortex → neocortex)*
