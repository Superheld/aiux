# AIUX - Product Requirements Document

> Ein neuronales, LLM-gesteuertes Betriebssystem.
> Embodied AI - ein OS das wahrnimmt, versteht und handelt.

---

## Vision

AIUX ist ein minimales Linux-System, in dem KI kein Werkzeug ist, sondern eine
**Schicht des Betriebssystems**. Mensch und KI arbeiten kooperativ - das System
denkt mit, nimmt wahr und handelt.

Nicht ein Chatbot auf einem OS. Ein OS das lebt.

---

## Kernprinzipien

- **Minimal** - Nur was gebraucht wird, nichts mehr
- **Kooperativ** - Mensch und LLM arbeiten zusammen, nicht gegeneinander
- **Geschichtet** - Klare Trennung: Kernel → Nerven → Agent → Mensch
- **Sicher** - LLM hat keinen Root-Zugang, klares Privilege-Modell
- **Erweiterbar** - Neue "Nerven" und Fähigkeiten hinzufügbar

---

## Architektur

```
┌─────────────────────────────────────────────┐
│              Mensch-Interface                │
│    Terminal → TUI → Web → Touch-GUI         │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│           aiux-agent (LLM-Client)           │
│     Denkt, plant, kommuniziert, handelt     │
│        API: Anthropic / Mistral             │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│              aiux-hub                        │
│     Verbindet Agent ↔ Nerven ↔ System       │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│          aiux-nerve (Daemons)               │
│   Kleine neuronale Netze, lokal, schnell    │
│                                             │
│   log-brain   net-brain   res-brain         │
│   sec-brain   health-brain                  │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│           Alpine Linux (Basis-OS)           │
│    Kernel · busybox · musl · Netzwerk       │
└─────────────────────────────────────────────┘
```

### Schicht 1: Basis-OS (Alpine Linux)

Minimales Linux auf ARM (Raspberry Pi 4, 8GB):

- Linux Kernel (aarch64)
- Alpine Linux als Basis (~5 MB)
- busybox, musl, Shell
- Netzwerk (WiFi/Ethernet)
- Kein Desktop, keine GUI (vorerst)

### Schicht 2: Neuronales System (aiux-nerve)

Kleine, spezialisierte neuronale Netze die lokal laufen:

- **log-brain** - Anomalie-Erkennung in System-Logs
- **net-brain** - Netzwerk-Überwachung, ungewöhnlicher Traffic
- **res-brain** - Ressourcen-Optimierung, predictive Scheduling
- **sec-brain** - Security-Monitoring, Intrusion Detection
- **health-brain** - Hardware-Gesundheit (Temperatur, Disk, RAM)

Technologie: ONNX Runtime auf ARM, Modelle je ~5 MB.
Laufen als Daemons, melden Auffälligkeiten an den Hub.

### Schicht 3: LLM-Agent (aiux-agent)

Der Denkende. Kommuniziert mit externen LLMs (Anthropic, Mistral):

- Nimmt Eingaben vom Mensch entgegen
- Empfängt Meldungen von den Nerven
- Führt Aktionen aus (Shell, MCP, Tools)
- Erklärt, schlägt vor, fragt nach

Technologie: Shell + curl (Phase 1), Rust-Binary (Phase 2).

### Schicht 4: Hub (aiux-hub)

Verbindet alles:

- Nerve → Agent: "Anomalie erkannt"
- Agent → Mensch: "Jemand versucht SSH-Bruteforce"
- Mensch → Agent → System: "Blockiere die IP"
- Nerve → Nerve: Korrelation zwischen Events

---

## Privilege-Modell

Das LLM läuft als eigener User, NICHT als root.

| Stufe | Aktion | Bestätigung |
|-------|--------|-------------|
| **Frei** | Lesen, suchen, analysieren | Nein |
| **Normal** | Dateien ändern, Apps starten | Konfigurierbar |
| **Kritisch** | Pakete, Services, Netzwerk, System | Immer |

Kritische Aktionen erfordern immer menschliche Bestätigung.
Berechtigungen sind konfigurierbar (ähnlich sudoers, aber für das LLM).

---

## LLM-Anbindung

- **Primär**: API-basiert (Anthropic Claude, Mistral)
- **Lokal (experimentell)**: Kleine Modelle auf dem Raspi (Phi-3, Llama 3.2)
- **MCP**: Als Interface-Layer zwischen LLM und System-Tools
- **Tool-Use**: LLM kann definierte Funktionen aufrufen (Dateien, Services, Netzwerk)

---

## Mensch-Interface (Evolutionsstufen)

1. **Terminal** - Shell, direkte Interaktion
2. **TUI** - Terminal UI mit Panels (Logs, Chat, Status)
3. **Web** - Remote-Zugang von anderen Geräten
4. **Touch-GUI** - Für das Display am Raspi

---

## Hardware (Phase 1)

- Raspberry Pi 4, 8GB RAM
- SD-Karte (Boot + System)
- Ethernet / WiFi für Netzwerk und API-Calls
- HDMI an Monitor (Entwicklung)
- Touch-Display (Drittanbieter, später)

---

## Nicht-Ziele (vorerst)

- Kein Desktop-Replacement
- Kein Multi-User-System
- Keine eigene Paketverwaltung
- Kein lokales LLM-Training
- Keine GUI in Phase 1

---

## Tech-Stack

| Schicht | Entscheidung | Begründung |
|---------|-------------|------------|
| Basis-OS | Alpine Linux (musl) | ~5 MB, minimal, ARM-Support |
| Init-System | OpenRC | Alpine-Default, einfach |
| Shell | ash (busybox) + bash | ash dabei, bash für Komfort |
| LLM-Client Phase 1 | curl + jq | Kein Overhead, sofort nutzbar |
| LLM-Client Phase 2+ | Rust | Memory-safe, kein GC, kleine Binaries |
| Lokale Inference | ONNX Runtime (Rust-Bindings) | Leichtgewichtig, breiter Modell-Support |
| IPC / Hub | Unix Sockets | Einfach, schnell, kein Overhead |
| TUI | ratatui (Rust) | Passt zum Rust-Stack |
| Web/Remote | htmx + Rust (axum/actix) | Leichtgewichtig, auch am Handy nutzbar |

## Offene Fragen

- [ ] Genaues Display-Modell identifizieren (Drittanbieter, Touch)
- [ ] MCP-Server-Architektur im Detail
- [ ] Wie lernen die Nerve-Modelle? Vortrainiert vs. on-device?
- [ ] Remote-Zugang: VPN, Tailscale, Cloudflare Tunnel?
- [ ] Sync-Strategie: Was bleibt zentral, was wird synchronisiert?

---

*Letzte Aktualisierung: 2026-02-28*
