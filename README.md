# AIUX

> Embodied AI - ein OS in dem eine KI lebt, nicht eine App auf einem OS.

## Was ist AIUX?

AIUX ist ein minimales System, in dem KI keine App ist, sondern eine
Schicht des Betriebssystems. Das Linux-System ist der Koerper, der Agent
lernt ihn zu spueren und zu nutzen.

Aktuell: REPL mit Streaming, Memory, Hippocampus, MQTT-Bridge.
Siehe [docs/PRD.md](docs/PRD.md) fuer die Vision.

## Voraussetzungen

| Was | Minimum | Hinweis |
|-----|---------|---------|
| **Rust** | 1.75+ | `rustup` empfohlen |
| **Cargo** | (kommt mit Rust) | Build-System |
| **LLM API Key** | Anthropic, Mistral oder Ollama | Provider per `config.toml` waehlbar |
| **Mosquitto** (optional) | 2.x | Nur fuer MQTT/Nervensystem. Ohne laeuft AIUX als reiner Chat. |

### Plattformen

| Plattform | Status |
|-----------|--------|
| Linux (x86_64) | Entwicklung, getestet |
| Linux (aarch64 / Raspi) | Zielsystem, Cross-Compilation |
| macOS | Sollte funktionieren (Pure Rust) |
| Windows | Ungetestet |

### Optionale Abhaengigkeiten

- **Mosquitto** - MQTT-Broker fuer das Nervensystem (Nerves, Brainstem)
  - Arch: `pacman -S mosquitto`
  - Alpine: `apk add mosquitto`
  - Debian/Ubuntu: `apt install mosquitto`

## Schnellstart (lokal)

```bash
git clone https://github.com/Superheld/aiux.git
cd aiux

# API Key setzen
cp .env.example .env
# .env editieren: ANTHROPIC_API_KEY=sk-ant-...

# Bauen und starten
cargo build --release
cargo run
```

### Mit MQTT (Nervensystem)

```bash
# Mosquitto starten
systemctl start mosquitto
# oder: mosquitto -d

# In home/.system/config.toml:
# mqtt_host = "localhost"
# mqtt_port = 1883

cargo run

# In einem anderen Terminal mitlesen:
mosquitto_sub -t 'aiux/#' -v

# Test-Signal senden:
mosquitto_pub -t aiux/nerve/test -m '{"source":"test","data":"hello"}'
```

## Tests

```bash
# Alle Tests ausfuehren
cargo test

# Einzelnen Test
cargo test test_name

# Nur Unit-Tests (ohne Integration)
cargo test --lib
```

Aktuell 77 Tests. Alle Tests laufen **ohne Netzwerk, ohne API-Key, ohne Mosquitto**.
LLM wird gemockt, Filesystem mit tempdir. Siehe [docs/TESTING.md](docs/TESTING.md).

## Auf dem Raspberry Pi

### Voraussetzungen

- Raspberry Pi 4 (aarch64) mit Linux (Alpine, Raspberry Pi OS, etc.)
- SSH-Zugang zum Raspi
- Rust auf dem Entwicklungsrechner
- Anthropic API Key

### Option A: Auf dem Raspi bauen

```bash
# Auf dem Raspi: Rust installieren
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Repo klonen und bauen
git clone https://github.com/Superheld/aiux.git
cd aiux
cargo build --release

# API Key setzen
cp .env.example .env
# .env editieren

# Starten
./target/release/aiux-core
```

### Option B: Cross-Compilation (schneller)

```bash
# Auf dem Entwicklungsrechner: Target hinzufuegen
rustup target add aarch64-unknown-linux-musl

# Bauen
cargo build --release --target aarch64-unknown-linux-musl

# Binary + home/ auf den Raspi kopieren
scp target/aarch64-unknown-linux-musl/release/aiux-core user@raspi:~/
scp -r home/ user@raspi:~/home/

# Auf dem Raspi: API Key setzen und starten
ssh user@raspi
echo "ANTHROPIC_API_KEY=sk-ant-..." > ~/home/.env
./aiux-core
```

> **Hinweis:** Cross-Compilation braucht ggf. einen Linker fuer aarch64.
> Auf Arch Linux: `sudo pacman -S aarch64-linux-gnu-gcc`
> und in `~/.cargo/config.toml`:
> ```toml
> [target.aarch64-unknown-linux-musl]
> linker = "aarch64-linux-gnu-gcc"
> ```

## Projektstruktur

```
aiux/
├── core/src/        # aiux-core (Rust) - das Gehirn
│   ├── agent/       # Cortex (LLM) + Hippocampus (Memory)
│   ├── bus/         # Interner Event-Bus (tokio::broadcast)
│   ├── mqtt.rs      # MQTT-Bridge (intern <-> extern)
│   └── tools/       # SoulTool, UserTool, MemoryTool
├── nerve/           # aiux-nerve (Workspace-Crate, Platzhalter)
├── home/            # Agent-Home
│   ├── .system/     # Config + System-Prompts
│   └── memory/      # Soul, User, Shortterm, Conversations
└── docs/            # PRD, Architektur, Roadmap, Testing
```

## Dokumentation

- [docs/PRD.md](docs/PRD.md) - Vision und Konzepte
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - Technische Architektur
- [docs/ROADMAP.md](docs/ROADMAP.md) - Phasen und Status
- [docs/TESTING.md](docs/TESTING.md) - Test-Strategie

## Lizenz

MIT - siehe [LICENSE](LICENSE)
