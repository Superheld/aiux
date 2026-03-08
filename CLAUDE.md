# AIUX — Claude Code Context

## What is AIUX?

Embodied AI — an OS where AI lives, not an app on an OS.
The Linux machine is the body; the agent learns to feel and use it.
See docs/PRD.md (vision), docs/ARCHITECTURE.md (technical), docs/ROADMAP.md (phases),
docs/TESTING.md (test strategy).

## Current State

Still a head without a body: REPL with memory, no daemon, no senses.
But: event bus works, code is modular, providers configurable.

- **core/src/main.rs** — Wiring (bus + core + REPL)
- **core/src/agent/** — Brain (neocortex.rs = LLM agent, hippocampus.rs = memory distillation)
- **core/src/bus/** — Internal event bus (tokio::sync::broadcast)
- **core/src/brainstem.rs** — Reflexes, nerve launcher, scheduler (rhai sandbox)
- **core/src/repl.rs** — Command line (stdin/stdout via bus)
- **core/src/config.rs** — Agent config from home/.system/config.toml
- **core/src/tools/** — Shell, Soul, User, Memory, Scheduler tools
- **home/.system/config.toml** — Provider, model, temperature, shell whitelist, etc.
- **home/memory/** — soul.md, user.md, notes.md, conversations/
- **nerve/** — Nerve processes (system-monitor, shared lib)

## Architecture Rules

- **Body architecture.** The system is built after a biological model:
  - **Neocortex** = Core/LLM. Thinks in language. Everything must arrive as text.
  - **Hippocampus** = Automatic memory. Listens along, stores unconsciously.
  - **Nerves** = Sensors. Own filter, preprocessing, report as text.
  - **Tools** = Hands. Conscious actions to the outside.
  - **Chat** = Direct access to the neocortex. No nerve, no filter.
- **Separation:** Core does not know nerves directly. Communication only via bus.
- **Roles:** Parallel agent instances with own config/memory. Main is the boss.
- **Tools are Rust code in core.** No plugin system. Comes later.
- **Preamble = soul.md + user.md + notes.md.** Order matters.
- **Events:** All communication via bus. SystemMessage for info to the user.
- **Compaction:** History is automatically summarized at token budget (compact_threshold).
- **simple_chat():** Internal LLM call without streaming/tools (for compaction etc.).
- **Testing:** LLM is mocked (custom MockModel), filesystem with tempdir. See docs/TESTING.md.

## Coding Rules

- **Language:** Rust
- **Error handling:** `anyhow` in main, `thiserror` for custom error types
- **Async:** tokio runtime, everything async
- **LLM:** rig-core 0.31 (Anthropic, streaming, tool use)
- **Serialization:** serde/serde_json, schemars for tool schemas
- **No over-engineering.** Only build what is needed now.
- **Simplest first.** Shell script before Rust daemon, if sufficient.
- **Code comments:** English

## Conventions

- Commits: `feat(<scope>):`, `fix(<scope>):`, `docs:`, `refactor:`, `test:`
- Merges: `--no-ff`
- Language: German for docs and communication, English for code and README

## Raspberry Pi (Target System)

- IP: 192.168.178.57, User: claude (uid 1000)
- SSH with key auth, firewall active
- Alpine Linux, save configs: `lbu commit -d`

## Deployment

- GitHub Actions builds release binaries on tags (`v*`)
- `install.sh` downloads latest release and installs to `~/bin/`
- First install creates home directory structure, updates only replace binaries
- `home/` on the target is never overwritten by deployment
