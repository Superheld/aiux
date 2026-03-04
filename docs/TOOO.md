# TODO

## Offen

- REPL: SystemMessages und Heartbeats unterbrechen den User-Input (Timing-Problem)
- Cron/Heartbeat: Scheduler-Eintraege sind nur im RAM, ueberleben keinen Neustart. Braucht persistierte Config (z.B. `[heartbeat]` in config.toml) die beim Boot automatisch geladen wird

## Erledigt

- ~~Config: Hippocampus-Model separat konfigurierbar (hippocampus_provider, hippocampus_model)~~
- ~~Architecture: Hippocampus-Model im Agent-Diagramm ergaenzt~~
- ~~Boot-Info: zeigt Model, Provider, MQTT-Status~~
- ~~MemoryTool: shortterm.md → notes.md, Beschreibungen angeglichen~~
- ~~tool-*.md Dateien entfernt (Beschreibungen fest im Code)~~
- ~~ToolArgs.key entfernt (nie benutzt)~~
