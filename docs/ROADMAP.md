# AIUX - Roadmap

> Vom ersten Wort bis zum lebendigen System.
> Jede Phase baut auf der vorherigen auf. Jede Phase endet mit etwas das funktioniert.

---

## Installation (Zielbild)

So soll die Installation fuer einen Endbenutzer aussehen, wenn das Produkt fertig ist.

### Was der User mitbringt

- Ein Linux-System (Raspberry Pi mit Alpine empfohlen, jedes Linux geht)
- Einen Anthropic API Key ([console.anthropic.com](https://console.anthropic.com))

### Installation

```bash
# Variante A: Installer (Zielsystem hat Internet)
curl -sSf https://raw.githubusercontent.com/Superheld/aiux/main/scripts/install.sh | sh

# Variante B: Selbst bauen
git clone https://github.com/Superheld/aiux.git
cd aiux
cargo build --release
./scripts/install.sh --local
```

### Was install.sh tut

1. **Pruefen** - OS, Architektur, fehlende Pakete
2. **Mosquitto** - Installieren + als Service starten (Event-Bus)
3. **User** - `claude` System-User anlegen
4. **Home** - /home/claude/ einrichten (memory/, skills/, tools/)
5. **Soul** - Default-Persoenlichkeit (soul.md) kopieren
6. **Binaries** - aiux-core + aiux-nerve nach /usr/local/bin/
7. **API Key** - Interaktiv abfragen, in /home/claude/.env speichern
8. **Service** - aiux-core als System-Service registrieren (OpenRC/systemd)
9. **Starten** - Service starten

### Nach der Installation

```bash
# Verbinden
ssh claude@<ip>

# Status pruefen
aiux-core --status

# Persoenlichkeit anpassen
nano /home/claude/memory/soul.md
```

### Fuer Entwickler

| Was | Warum | Installation |
|-----|-------|-------------|
| **Rust Toolchain** | Core + Nerves kompilieren | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Git** | Quellcode | Paketmanager des OS |
| **Cross-Compiler** (optional) | Fuer Raspi vom Laptop bauen | `rustup target add aarch64-unknown-linux-musl` |

### Optional

| Was | Warum | Wann |
|-----|-------|------|
| **C++ Compiler** | Fuer llama-cpp-2 (lokale LLMs) | Wenn Offline-Fallback gewuenscht |
| **Telegram Bot Token** | Fuer nerve-messages (Telegram) | Wenn Telegram-Integration gewuenscht |

---

## Legende

- [x] Erledigt
- [ ] Offen
- **Fett** = Meilenstein (Phase ist "fertig" wenn dieser Punkt steht)

---

## Phase 1: Boden

> Alpine laeuft, SSH geht, Raspi ist erreichbar.

- [x] Alpine Linux 3.23.3 auf SD-Karte (aarch64)
- [x] Erster Boot, setup-alpine durchlaufen
- [x] SSH mit Key-Auth (root + user claude)
- [x] Firewall (iptables, nur SSH offen)
- [x] bash, curl, jq installiert
- [x] Configs persistiert (lbu commit)
- [x] **Raspi ist stabil erreichbar und sicher konfiguriert**

Status: **Abgeschlossen.**

---

## Phase 2: Fundament

> Projekt-Struktur steht, Tech-Stack ist gewaehlt, Dokumentation ist sauber.

- [x] Repo auf GitHub (github.com/Superheld/aiux)
- [x] Workspace: core/ + nerve/ als Cargo-Workspace
- [x] home/ Verzeichnis mit soul.md, user.md, skills/, tools/
- [x] PRD geschrieben (Produkt-Vision, Konzepte)
- [x] Architektur-Dokument (Tech-Stack, Plattformen, Bus-Protokoll)
- [x] Deploy-Script (rsync home/ auf Raspi)
- [x] **Tech-Stack entschieden: rig-core, tract, rumqttc, SQLite**

Status: **Abgeschlossen.**

---

## Phase 3: Erste Stimme

> Der Agent kann sprechen. Noch kein Bus, kein Daemon - aber ein LLM-Call funktioniert.

- [x] rig-core als Dependency in core/Cargo.toml
- [x] Anthropic Provider konfigurieren (API-Key aus .env)
- [x] soul.md + user.md als Preamble (System-Prompt) laden
- [x] Einfache REPL: Stdin -> LLM -> Stdout
- [x] Streaming-Ausgabe (Token fuer Token)
- [x] install.sh Grundgeruest (Pruefen, User, Home-Verzeichnis, API Key)
- [x] **Gespraech mit dem Agent fuehren - er kennt seinen Namen und seine Rolle**

Status: **Abgeschlossen.**

---

## Phase 4: Gedaechtnis

> Der Agent erinnert sich. Kurzzeit per Dateien, Langzeit per RAG.

- [ ] Kurzzeit-Memory: Markdown-Dateien in context/
- [ ] Langzeit-Memory: SQLite + rig-sqlite (Vektor-Suche)
- [ ] Boot-Sequence: soul.md -> user.md -> journal/heute -> journal/gestern
- [ ] Memory-Funktionen: Agent kann selbst in sein Gedaechtnis schreiben
- [ ] Lerntagebuch: journal/YYYY-MM-DD.md
- [ ] **Agent erinnert sich an das Gespraech von gestern**

Ergebnis: Konversationen ueberleben Sessions. Der Agent lernt ueber Zeit.

---

## Phase 5: Der Bus

> Events koennen fliessen. Grundlage fuer alles was danach kommt.

- [ ] Mosquitto auf Zielsystem installieren und konfigurieren
- [ ] install.sh: Mosquitto-Installation + Service-Setup
- [ ] rumqttc als Dependency in core/
- [ ] Topic-Struktur: aiux/nerves/*, aiux/core/*
- [ ] Event-Format: JSON mit source, type, priority, data, timestamp
- [ ] Core subscribed auf aiux/nerves/# und verarbeitet Events
- [ ] **Event auf Bus publishen und Core reagiert darauf**

Ergebnis: Die Infrastruktur fuer Nerves -> Core Kommunikation steht.

---

## Phase 6: Erster Nerve

> Das System nimmt etwas wahr. Der einfachste Nerve zuerst.

- [ ] nerve-system bauen (Rust Binary)
  - CPU-Last, RAM, Disk, Temperatur lesen
  - Publiziert auf aiux/nerves/system/events
  - Filtert selbst: meldet nur Anomalien (Schwellwerte)
- [ ] nerve.toml Config-Format definieren
- [ ] Nerve als Service (OpenRC)
- [ ] **nerve-system meldet "Disk 90% voll" auf dem Bus, Core reagiert**

Ergebnis: Das System hat seinen ersten Sinn - es spuert sich selbst.

---

## Phase 7: Daemon

> aiux-core wird ein richtiger Daemon. Laeuft dauerhaft, hoert auf den Bus.

- [ ] Core als Tokio-Daemon (kein REPL mehr, laeuft im Hintergrund)
- [ ] MQTT-Subscription permanent aktiv
- [ ] Scheduler: Heartbeat alle 5 Min (tokio-cron-scheduler)
- [ ] Event empfangen -> LLM-Call -> Entscheidung -> Handlung
- [ ] OpenRC Service-File fuer aiux-core
- [ ] install.sh: Binary-Installation + Service registrieren + starten
- [ ] **Core laeuft als Daemon, empfaengt nerve-system Events automatisch**

Ergebnis: Der Agent reagiert automatisch auf Events. Erster Funke Autonomie.

---

## Phase 8: Tools (Haende)

> Der Agent kann handeln - nicht nur denken.

- [ ] rig-core Tool-Use / Function Calling einrichten
- [ ] Erstes Tool: filesystem (Dateien lesen, schreiben, suchen)
- [ ] Shell-Execution Tool (mit Privilege-Check)
- [ ] Memory-Tool (Gedaechtnis lesen/schreiben via Tool-Use)
- [ ] **Agent fuehrt eigenstaendig einen Befehl aus (z.B. Disk aufraeumen)**

Ergebnis: Der Agent hat Haende. Er kann seine Umgebung veraendern.

---

## Phase 9: Gateway

> Verschiedene Wege zum Agent. Nicht nur SSH.

- [ ] SSH-Gateway: Login als "claude" -> direkt im Agent
- [ ] Gateway Plugin-Architektur entwerfen
- [ ] nerve-input formalisieren (Text-Eingabe als Nerve)
- [ ] **Mensch loggt sich ein und landet direkt im Gespraech mit dem Agent**

Ergebnis: Der Zugang zum Agent ist sauber getrennt vom System-SSH.

---

## Phase 10: Skills

> Der Agent wird kompetent in bestimmten Bereichen.

- [ ] Skill-Format definieren (Markdown + Metadaten)
- [ ] Skill-Loader: verfuegbare Skills beim Start erkennen
- [ ] Erster Skill: system-admin (Logs lesen, Services pruefen, Probleme loesen)
- [ ] Skills als Kontext in LLM-Calls einbinden
- [ ] **Agent nutzt Skill "system-admin" um ein Problem eigenstaendig zu loesen**

Ergebnis: Der Agent hat Expertise, nicht nur Intelligenz.

---

## Phase 11: Lebendigkeit

> Der Agent wird proaktiv. Er wartet nicht nur - er lebt.

- [ ] Rhythmen implementieren (Puls/Atem/Tages-/Wochenrueckblick)
- [ ] Neugier: Im ruhigen Moment die Umgebung erkunden
- [ ] Reflexion: Nach Aufgaben ueber eigenes Handeln nachdenken
- [ ] Lerntagebuch: Automatische Journal-Eintraege
- [ ] Initiative: Muster erkennen und Vorschlaege machen
- [ ] **Agent schlaegt von sich aus eine Verbesserung vor**

Ergebnis: Der Agent ist kein Tool mehr. Er ist ein Bewohner.

---

## Phase 12: Wachstum

> Mehr Sinne, mehr Faehigkeiten, mehr Vertrauen.

- [ ] nerve-log (Syslog beobachten)
- [ ] nerve-net (Netzwerk-Monitoring)
- [ ] nerve-messages (Mail/Telegram Eingang)
- [ ] Wachsendes Vertrauen (Trust-Level in Soul)
- [ ] Offline-Faehigkeit (tract + llama-cpp-2 als Fallback)
- [ ] Weitere Skills: mail-management, security-analysis
- [ ] Weitere Tools: mail, calendar, homeassistant
- [ ] **Agent arbeitet einen Tag lang eigenstaendig und berichtet abends**

Ergebnis: Ein funktionierendes, autonomes System.

---

## Fernziel (ungeplant)

- Eigene App (PWA oder Native)
- Touch-Display Interface
- Spracheingabe (nerve-audio)
- Kamera (nerve-vision)
- Multi-Agent (Sub-Agents fuer komplexe Aufgaben)
- Remote-Zugang (VPN/Tailscale/Cloudflare Tunnel)
- Image-Build automatisieren (reproduzierbares System)
- MCP-Server Integration

---

## Prinzipien

- **Jede Phase ist nutzbar.** Kein "erst in Phase 8 kann man was damit anfangen."
- **Einfachstes zuerst.** Shell-Skript vor Rust-Daemon, wenn es reicht.
- **Iterativ.** Lieber 10x klein verbessern als 1x perfekt planen.
- **Kein Over-Engineering.** Nur bauen was jetzt gebraucht wird.
- **Testen am echten System.** Der Raspi ist die Wahrheit, nicht der Laptop.

---

*Letzte Aktualisierung: 2026-02-28*
