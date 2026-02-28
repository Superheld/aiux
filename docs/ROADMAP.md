# AIUX - Roadmap

> Vom ersten Wort bis zum lebendigen System.
> Jede Phase baut auf der vorherigen auf. Jede Phase endet mit etwas das funktioniert.

---

## Legende

- [x] Erledigt
- [ ] Offen
- **Fett** = Meilenstein (Phase ist "fertig" wenn dieser Punkt steht)

---

## Phase 1: Boden unter den Füßen

> Alpine läuft, SSH geht, Raspi ist erreichbar.

- [x] Alpine Linux 3.23.3 auf SD-Karte (aarch64)
- [x] Erster Boot, setup-alpine durchlaufen
- [x] SSH mit Key-Auth (root + user claude)
- [x] Firewall (iptables, nur SSH offen)
- [x] bash, curl, jq installiert
- [x] Configs persistiert (lbu commit)
- [x] **aichat v0.30.0 installiert**

Status: **Abgeschlossen.**

---

## Phase 2: Die erste Stimme

> Der Agent kann sprechen. Noch nicht autonom, noch kein Bus - aber er redet.

- [ ] aichat konfigurieren (API-Key Anthropic)
- [ ] soul.md schreiben (erste Version der Persönlichkeit)
- [ ] soul.md als aichat System-Prompt laden
- [ ] Memory-Verzeichnisstruktur anlegen (/home/claude/memory/)
- [ ] Erster LLM-Call auf dem Raspi (aichat interaktiv)
- [ ] **Gespräch mit dem Agent führen - er kennt seinen Namen und seine Rolle**

Ergebnis: Man kann sich per SSH einloggen und mit dem Agent reden.
Er hat eine Persönlichkeit, aber noch kein Gedächtnis über Sessions hinweg.

---

## Phase 3: Gedächtnis

> Der Agent erinnert sich. Kurzzeit per Dateien, Langzeit per RAG.

- [ ] Kurzzeit-Memory: Markdown-Dateien in /home/claude/memory/context/
- [ ] aichat RAG einrichten (Dokumente indexieren)
- [ ] Memory-Tool: Agent kann selbst in sein Gedächtnis schreiben
- [ ] Lerntagebuch-Struktur: /home/claude/memory/journal/
- [ ] **Agent erinnert sich an das Gespräch von gestern**

Ergebnis: Konversationen überleben Sessions. Der Agent lernt über Zeit.

---

## Phase 4: Der Bus

> Events können fließen. Grundlage für alles was danach kommt.

- [ ] Mosquitto installieren (apk add mosquitto)
- [ ] Mosquitto konfigurieren (lokaler Broker, kein Auth nötig)
- [ ] Topic-Struktur festlegen (aiux/nerves/*, aiux/core/*, ...)
- [ ] Bus-Protokoll definieren (JSON-Format, Prioritäten)
- [ ] CLI-Test: mosquitto_pub/sub funktioniert
- [ ] **Event auf Bus publishen und in einem Subscriber empfangen**

Ergebnis: Die Infrastruktur für Nerves → Core Kommunikation steht.

---

## Phase 5: Erster Nerve

> Das System nimmt etwas wahr. Der einfachste Nerve zuerst.

- [ ] nerve-system bauen (Shell-Skript oder Rust)
  - CPU-Last, RAM, Disk, Temperatur
  - Publiziert auf aiux/nerves/system/events
  - Meldet nur Anomalien (Schwellwerte)
- [ ] nerve.toml Config-Format definieren
- [ ] Nerve als Service (OpenRC oder einfacher Loop)
- [ ] **nerve-system meldet "Disk 90% voll" auf dem Bus**

Ergebnis: Das System hat seinen ersten Sinn - es spürt sich selbst.

---

## Phase 6: aiux-core (Prototyp)

> Der Rust-Daemon der alles verbindet. Hört auf den Bus, denkt, handelt.

- [ ] Rust-Projekt aufsetzen (Cargo, Dependencies)
- [ ] MQTT-Client: Events vom Bus empfangen
- [ ] aichat als Subprocess starten und steuern
- [ ] Event empfangen → LLM-Call → Entscheidung
- [ ] Einfacher Scheduler (Heartbeat alle 5 Min)
- [ ] **Core empfängt nerve-system Event und reagiert sinnvoll**

Ergebnis: Der Agent reagiert automatisch auf Events. Erster Funke Autonomie.

---

## Phase 7: Tools

> Der Agent kann handeln - nicht nur denken.

- [ ] Erstes llm-function Tool (z.B. filesystem)
- [ ] Shell-Execution Tool (mit Privilege-Check)
- [ ] Memory-Tool (Gedächtnis lesen/schreiben via Tool-Use)
- [ ] MCP-Server Evaluation: welche gibt es, was passt?
- [ ] **Agent führt eigenständig einen Befehl aus (z.B. Disk aufräumen)**

Ergebnis: Der Agent hat Hände. Er kann seine Umgebung verändern.

---

## Phase 8: Gateway

> Verschiedene Wege zum Agent. Nicht nur SSH.

- [ ] SSH-Gateway: Login als "claude" → direkt im Agent
- [ ] Gateway Plugin-Architektur entwerfen
- [ ] nerve-input formalisieren (Text-Eingabe als Nerve)
- [ ] **Mensch loggt sich ein und landet direkt im Gespräch mit dem Agent**

Ergebnis: Der Zugang zum Agent ist sauber getrennt vom System-SSH.

---

## Phase 9: Skills

> Der Agent wird kompetent in bestimmten Bereichen.

- [ ] Skill-Format definieren (Markdown + Config)
- [ ] Erster Skill: system-admin (Logs lesen, Services prüfen, Probleme lösen)
- [ ] aichat Agent-Integration (Skill = aichat Agent)
- [ ] Skill-Loader: verfügbare Skills beim Start erkennen
- [ ] **Agent nutzt Skill "system-admin" um ein Problem eigenständig zu lösen**

Ergebnis: Der Agent hat Expertise, nicht nur Intelligenz.

---

## Phase 10: Lebendigkeit

> Der Agent wird proaktiv. Er wartet nicht nur - er lebt.

- [ ] Rhythmen implementieren (Puls/Atem/Tages-/Wochenrückblick)
- [ ] Neugier: Im ruhigen Moment die Umgebung erkunden
- [ ] Reflexion: Nach Aufgaben über eigenes Handeln nachdenken
- [ ] Lerntagebuch: Automatische Journal-Einträge
- [ ] Initiative: Muster erkennen und Vorschläge machen
- [ ] **Agent schlägt von sich aus eine Verbesserung vor**

Ergebnis: Der Agent ist kein Tool mehr. Er ist ein Bewohner.

---

## Phase 11: Wachstum

> Mehr Sinne, mehr Fähigkeiten, mehr Vertrauen.

- [ ] nerve-log (Syslog beobachten)
- [ ] nerve-net (Netzwerk-Monitoring)
- [ ] nerve-messages (Mail/Telegram Eingang)
- [ ] Wachsendes Vertrauen (Trust-Level in Soul)
- [ ] Offline-Fähigkeit (lokales Modell als Fallback)
- [ ] Weitere Skills: mail-management, security-analysis
- [ ] Weitere Tools: mail, calendar, homeassistant (MCP)
- [ ] **Agent arbeitet einen Tag lang eigenständig und berichtet abends**

Ergebnis: Ein funktionierendes, autonomes System.

---

## Fernziel (ungeplant)

- Eigene App (PWA oder Native)
- Touch-Display Interface
- Spracheingabe (nerve-audio)
- Kamera (nerve-vision)
- Multi-Agent (Sub-Agents für komplexe Aufgaben)
- Remote-Zugang (VPN/Tailscale/Cloudflare Tunnel)
- Image-Build automatisieren (reproduzierbares System)
- Zweiter Raspi / Cluster

---

## Prinzipien

- **Jede Phase ist nutzbar.** Kein "erst in Phase 8 kann man was damit anfangen."
- **Einfachstes zuerst.** Shell-Skript vor Rust-Daemon, wenn es reicht.
- **Iterativ.** Lieber 10x klein verbessern als 1x perfekt planen.
- **Kein Over-Engineering.** Nur bauen was jetzt gebraucht wird.
- **Testen am echten System.** Der Raspi ist die Wahrheit, nicht der Laptop.

---

*Letzte Aktualisierung: 2026-02-28*
