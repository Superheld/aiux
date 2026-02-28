# AIUX

> Ein neuronales, LLM-gesteuertes Betriebssystem.
> Embodied AI - ein OS das wahrnimmt, versteht und handelt.

## Was ist AIUX?

AIUX ist ein minimales System, in dem KI keine App ist, sondern eine
Schicht des Betriebssystems. Mensch und LLM arbeiten kooperativ - das
System denkt mit, nimmt wahr und handelt.

## Dokumentation

- [docs/PRD.md](docs/PRD.md) - Was ist AIUX? (Vision, Konzepte)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - Wie ist AIUX gebaut? (Tech-Stack, Plattformen)
- [docs/ROADMAP.md](docs/ROADMAP.md) - Wann wird was gebaut? (Phasen)

## Projektstruktur

```
aiux/
├── core/            # aiux-core (Rust) - das Gehirn
├── nerve/           # aiux-nerve (Rust) - die Sinne
├── home/            # Agent-Home (wird auf Zielsystem deployed)
│   ├── memory/
│   │   ├── soul.md  # Persoenlichkeit (= System-Prompt)
│   │   └── user.md  # Wissen ueber den Menschen
│   ├── skills/      # Expertise als Markdown
│   └── tools/       # Werkzeuge
├── build/           # System-Build Config
├── scripts/         # Deploy-Scripts
└── docs/            # Dokumentation
```

## Plattformen

| Plattform | Status |
|-----------|--------|
| Linux x86_64 | Unterstuetzt |
| Linux aarch64 (Raspi) | Unterstuetzt (Primaer-Ziel) |
| macOS (Intel + Apple Silicon) | Unterstuetzt |
| Windows x86_64 | Unterstuetzt |

## Schnellstart

```bash
git clone https://github.com/Superheld/aiux.git
cd aiux
cargo build --release
```

## Lizenz

MIT - siehe [LICENSE](LICENSE)
