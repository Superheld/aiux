# AIUX

> Embodied AI — an OS where AI lives, not an app on an OS.

## What is AIUX?

AIUX is a minimal system where AI is not an application but a layer of the operating system. The Linux machine is the body — the agent learns to feel and use it.

Currently: REPL with streaming, memory, hippocampus, MQTT bridge, shell access.
See [docs/PRD.md](docs/PRD.md) for the full vision.

## Quick Install

On the target machine (Raspberry Pi, any Linux box):

```bash
curl -fsSL https://raw.githubusercontent.com/Superheld/aiux/main/install.sh | sh
```

This downloads the latest release binaries and sets up the home directory on first run. On subsequent runs it only updates the binaries — config and memory are never touched.

You can also clone the repo and run it manually:

```bash
./install.sh
```

After installing, set your API key:

```bash
# Edit ~/.env
ANTHROPIC_API_KEY=sk-ant-...
```

Then start:

```bash
~/bin/aiux-core
```

## Prerequisites

| What | Minimum | Note |
|------|---------|------|
| **Linux** | aarch64 or x86_64 | Alpine, Debian, Raspberry Pi OS, ... |
| **LLM API Key** | Anthropic | Provider configurable in `config.toml` |
| **Mosquitto** (optional) | 2.x | Only for MQTT / nerve system. Without it AIUX runs as plain chat. |

## Building from Source

```bash
git clone https://github.com/Superheld/aiux.git
cd aiux

cp .env.example .env
# Edit .env: ANTHROPIC_API_KEY=sk-ant-...

cargo build --release
cargo run
```

### With MQTT (nerve system)

```bash
# Start Mosquitto
systemctl start mosquitto
# or: mosquitto -d

# In home/.system/config.toml:
# [mqtt]
# host = "localhost"
# port = 1883

cargo run

# Listen on another terminal:
mosquitto_sub -t 'aiux/#' -v

# Send a test signal:
mosquitto_pub -t aiux/nerve/test -m '{"source":"test","data":"hello"}'
```

## Tests

```bash
cargo test          # all tests
cargo test --lib    # unit tests only
```

All tests run **without network, API keys, or Mosquitto**. The LLM is mocked, the filesystem uses tempdir. See [docs/TESTING.md](docs/TESTING.md).

## Architecture

Built after a biological model:

| Component | Role |
|-----------|------|
| **Neocortex** | The brain — LLM agent with streaming and tools |
| **Hippocampus** | Automatic memory — listens, distills knowledge |
| **Nerves** | Sensors — own processes, communicate via MQTT |
| **Brainstem** | Reflexes — rhai sandbox, heartbeat, nerve launcher |
| **Tools** | Hands — shell commands, memory read/write |
| **Chat** | Direct access to the neocortex, no filtering |

Communication happens through an internal event bus (`tokio::broadcast`).
Nerves are external processes connected via MQTT.

```
aiux/
├── core/src/        # aiux-core — the brain
│   ├── agent/       # Neocortex (LLM) + Hippocampus (memory)
│   ├── bus/         # Internal event bus
│   ├── brainstem.rs # Reflexes, nerve launcher, scheduler
│   ├── mqtt.rs      # MQTT bridge (internal <-> external)
│   └── tools/       # Shell, Soul, User, Memory, Scheduler
├── nerve/           # Nerve processes
│   ├── shared/      # Common MQTT + registration code
│   └── system/      # System monitor (CPU, RAM, disk, temp)
├── home/            # Agent home (template for first install)
│   ├── .system/     # Config + system prompts
│   └── nerves/      # Nerve manifests + rhai scripts
└── docs/            # PRD, architecture, roadmap, testing
```

## Documentation

- [docs/PRD.md](docs/PRD.md) — Vision and concepts
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — Technical architecture
- [docs/ROADMAP.md](docs/ROADMAP.md) — Phases and status
- [docs/TESTING.md](docs/TESTING.md) — Test strategy

## License

MIT — see [LICENSE](LICENSE)
