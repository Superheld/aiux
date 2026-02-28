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

> Der Agent erinnert sich. Schritt fuer Schritt, jedes Inkrement einzeln testen.

### 4.1 Memory-Dateien + Boot-Sequence

> context/*.md Dateien werden beim Start geladen. Der Agent hat Kontext.

- [ ] Boot-Sequence: soul.md -> user.md -> context/*.md als Preamble
- [ ] load_context_files() laedt alle .md aus memory/context/
- [ ] Startup zeigt was geladen wurde
- [ ] **Test: context/test.md anlegen, Agent weiss beim Start davon**

### 4.2 Memory-Tool (Tool-Use)

> Der Agent kann selbst in sein Gedaechtnis schreiben, lesen, auflisten.

- [ ] MemoryTool implementieren (rig-core Tool trait)
- [ ] Aktionen: write, read, list auf memory/context/
- [ ] Sicherheit: kein Path-Traversal, nur einfache Dateinamen
- [ ] **Test: Agent bitten sich etwas zu merken, neu starten, fragen ob er es weiss**

### 4.3 Conversation-Persistenz

> Was passiert mit der Chat-History? Geht sie verloren beim Beenden?

- [ ] History in Datei speichern (memory/conversation.json oder aehnlich)
- [ ] Beim Start letzte N Nachrichten laden
- [ ] Entscheidung: wie viel History? Komplett? Zusammenfassung?
- [ ] **Test: Gespraech fuehren, beenden, neu starten - Agent kennt den Kontext**

### 4.4 RAG (Vektor-Suche)

> Statt alles in den Preamble zu stopfen: relevantes per Embedding finden.

- [ ] rig-sqlite als Dependency (SQLite + sqlite-vec)
- [ ] Embedding-Modell konfigurieren
- [ ] Memory-Dateien indexieren
- [ ] Bei jeder Frage: relevante Erinnerungen per Vektor-Suche finden
- [ ] **Test: Viele Notizen anlegen, Agent findet die relevante ohne alles zu laden**

---

## Phase 5: Umgebung + Autonomie

> Der Agent kennt sein Zuhause und kann selbstaendig handeln.

### 5.1 Umgebungs-Bewusstsein

> Der Agent weiss wo er laeuft, was er sehen und anfassen kann.

- [ ] environment.md: OS, Hostname, IP, Pfade, verfuegbare Tools
- [ ] Automatisch generieren beim Start (oder als Boot-Kontext)
- [ ] In Preamble einbinden
- [ ] **Test: Agent fragen "wo bin ich?" - er antwortet korrekt**

### 5.2 Shell-Tool

> Der Agent kann Befehle ausfuehren. Erste Haende.

- [ ] ShellTool implementieren (rig-core Tool trait)
- [ ] Privilege-Check: Whitelist erlaubter Befehle? Oder Trust-Level?
- [ ] Timeout + Output-Limit
- [ ] **Test: Agent bitten "wie voll ist die Disk?" - er fuehrt df aus**

### 5.3 Heartbeat + Scheduler

> Der Agent meldet sich regelmaessig. Erster Puls.

- [ ] tokio-cron-scheduler als Dependency
- [ ] Heartbeat: alle N Minuten ein LLM-Call ("Was steht an?")
- [ ] Agent kann eigene Cronjobs anlegen/aendern
- [ ] **Test: Agent konfiguriert selbst einen taeglichen Check**

### 5.4 Daemon-Modus

> Von REPL zu Hintergrund-Dienst. Laeuft dauerhaft.

- [ ] Core als Tokio-Daemon (neben REPL, oder Umschaltung)
- [ ] OpenRC Service-File
- [ ] install.sh: Binary + Service registrieren
- [ ] **Test: Agent laeuft als Service, Heartbeat tickt, Logs zeigen Aktivitaet**

---

## Phase 6: Der Bus + Sinne

> Events fliessen. Das System nimmt seine Umgebung wahr.

### 6.1 MQTT-Bus

> Mosquitto als Event-Backbone. Core hoert zu.

- [ ] Mosquitto auf Zielsystem installieren
- [ ] rumqttc als Dependency in core/
- [ ] Topic-Struktur: aiux/nerves/*, aiux/core/*
- [ ] Event-Format: JSON (source, type, priority, data, timestamp)
- [ ] **Test: Event per mosquitto_pub senden, Core empfaengt und loggt**

### 6.2 Erster Nerve: System

> Das System spuert sich selbst. CPU, RAM, Disk, Temperatur.

- [ ] nerve-system bauen (Rust Binary)
- [ ] Publiziert auf aiux/nerves/system/events
- [ ] Filtert selbst: meldet nur Anomalien (Schwellwerte)
- [ ] nerve.toml Config-Format
- [ ] **Test: nerve-system meldet "Disk 90% voll", Core reagiert darauf**

### 6.3 Weitere Nerves

> Mehr Sinne, je nach Bedarf.

- [ ] nerve-log (Syslog beobachten)
- [ ] nerve-net (Netzwerk-Monitoring)
- [ ] nerve-messages (Mail/Telegram Eingang)
- [ ] **Test: Ein neuer Nerve wird angeschlossen, Core verarbeitet seine Events**

---

## Phase 7: Gateway + Zugang

> Verschiedene Wege zum Agent.

### 7.1 SSH-Gateway

> Login als "claude" -> direkt im Agent.

- [ ] SSH-Login fuer claude User -> REPL startet
- [ ] **Test: ssh claude@raspi landet im Gespraech**

### 7.2 Weitere Gateways

> Nicht nur SSH. Je nach Bedarf.

- [ ] Gateway Plugin-Architektur entwerfen
- [ ] nerve-input formalisieren (Text-Eingabe als Nerve)
- [ ] Telegram? Web? API?
- [ ] **Test: Nachricht ueber alternativen Kanal kommt beim Agent an**

---

## Phase 8: Skills + Kompetenz

> Der Agent wird gut in bestimmten Bereichen.

### 8.1 Skill-Format

> Wie beschreibt man eine Faehigkeit?

- [ ] Skill-Format definieren (Markdown + Metadaten)
- [ ] Skill-Loader: verfuegbare Skills beim Start erkennen
- [ ] Skills als Kontext in LLM-Calls einbinden
- [ ] **Test: Skill wird geladen und beeinflusst das Verhalten**

### 8.2 Erste Skills

> Konkrete Faehigkeiten.

- [ ] system-admin (Logs lesen, Services pruefen, Probleme loesen)
- [ ] Weitere nach Bedarf
- [ ] **Test: Agent nutzt Skill um ein Problem eigenstaendig zu loesen**

---

## Phase 9: Lebendigkeit

> Der Agent wird proaktiv. Er wartet nicht nur - er lebt.

### 9.1 Rhythmen

> Puls, Tagesrhythmus, Reflexion.

- [ ] Tages-/Wochenrueckblick (automatischer Journal-Eintrag)
- [ ] Reflexion: Nach Aufgaben ueber eigenes Handeln nachdenken
- [ ] **Test: Agent schreibt abends eine Zusammenfassung des Tages**

### 9.2 Initiative

> Der Agent handelt von sich aus.

- [ ] Neugier: Im ruhigen Moment die Umgebung erkunden
- [ ] Muster erkennen und Vorschlaege machen
- [ ] Wachsendes Vertrauen (Trust-Level in Soul)
- [ ] **Test: Agent schlaegt von sich aus eine Verbesserung vor**

---

## Fernziel (ungeplant)

- Eigene App (PWA oder Native)
- Touch-Display Interface
- Spracheingabe (nerve-audio)
- Kamera (nerve-vision)
- Workspaces + Sub-Agents (rig-core Agent-als-Tool, verschiedene Modelle pro Aufgabe)
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
