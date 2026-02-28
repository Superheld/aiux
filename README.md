# AIUX

> Ein neuronales, LLM-gesteuertes Betriebssystem.
> Embodied AI - ein OS das wahrnimmt, versteht und handelt.

## Was ist AIUX?

AIUX ist ein minimales Linux-System auf Basis von Alpine Linux, in dem KI keine
App ist, sondern eine Schicht des Betriebssystems. Mensch und LLM arbeiten
kooperativ - das System denkt mit, nimmt wahr und handelt.

Nicht ein Chatbot auf einem OS. Ein OS das lebt.

Siehe [docs/PRD.md](docs/PRD.md) fuer die vollstaendige Vision und
[docs/ROADMAP.md](docs/ROADMAP.md) fuer den aktuellen Stand.

## Schnellstart (lokal testen)

```bash
# Repo klonen
git clone https://github.com/Superheld/aiux.git
cd aiux

# aichat bauen
cargo build --release -p aichat

# Config anlegen
mkdir -p ~/.config/aichat
cp system/aichat/config.example.yaml ~/.config/aichat/config.yaml
# API-Key eintragen!

# Mit dem Agent sprechen
cat home/memory/soul.md | ./target/release/aichat -S - "Wer bist du?"
```

## Projektstruktur

```
aiux/
├── aichat/          # LLM-Engine (Subtree von sigoden/aichat)
├── core/            # aiux-core (Rust) - Bus, Scheduler, Autonomie
├── nerve/           # aiux-nerve (Rust) - Sinnesorgane
├── home/            # Agent-Home (wird auf /home/claude/ deployed)
│   ├── memory/
│   │   ├── soul.md  # Persoenlichkeit
│   │   └── user.md  # Wissen ueber den Menschen
│   ├── skills/      # Expertise (aichat Agents)
│   └── tools/       # Werkzeuge (MCP, llm-functions)
├── system/          # System-Configs (aichat, Mosquitto, OpenRC)
├── build/           # Image-Build-System
├── scripts/         # Deploy und Hilfsskripte
└── docs/            # PRD, Roadmap
```

## Tech-Stack

| Schicht | Technologie |
|---------|-------------|
| OS | Alpine Linux (musl, busybox, aarch64) |
| LLM-Engine | [aichat](https://github.com/sigoden/aichat) (im Repo als Subtree) |
| Core | Rust Daemon (wraps aichat, MQTT-Bus, Scheduler) |
| Bus | Mosquitto (MQTT) |
| Memory | Markdown (Kurzzeit) + SQLite/RAG (Langzeit) |
| Tools | MCP-Server + aichat llm-functions |
| Lokale Inference | ONNX Runtime (fuer Nerves) |

## Hardware

- Raspberry Pi 4 (8 GB RAM)
- microSD-Karte (mind. 8 GB)
- Ethernet oder WiFi

## Deploy auf den Raspi

```bash
# Dateien syncen
./scripts/deploy.sh root@<raspi-ip>

# Auf dem Raspi: Configs sichern
ssh root@<raspi-ip> "lbu commit -d"
```

Siehe [docs/PRD.md](docs/PRD.md) fuer die Raspi-Einrichtung im Detail.

## Lizenz

MIT - siehe [LICENSE](LICENSE)
