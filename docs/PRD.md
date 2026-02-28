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
- **Geschichtet** - Klare Trennung: Körper → Nerven → Bus → Core → Mensch
- **Sicher** - LLM hat keinen Root-Zugang, klares Privilege-Modell
- **Erweiterbar** - Nerven, Skills und Tools als Plugins
- **Autonom** - System arbeitet eigenständig, meldet sich wenn nötig

---

## Grundgedanke

Der Agent lebt in seinem System wie ein Mensch an seinem Rechner. Er hat ein
OS, kann Software installieren (`apk add mutt`), Programme nutzen, Dateien
verwalten - alles was ein User am Terminal tun kann.

Nicht alles braucht ein spezielles Framework. Manchmal installiert der Agent
einfach ein Programm und benutzt es. Die AIUX-Konzepte (Nerves, Tools, Skills,
Memory) sind das, was ihn **über einen normalen User hinaushebt**.

---

## Konzepte

### Core = Das Bewusstsein

Der Core ist der Agent selbst. Er denkt, entscheidet, handelt.
Ohne seine Nerven ist er blind und taub - **alles** kommt über Nerves rein,
auch Texteingaben und Nachrichten.

Der Core:
- Empfängt Wahrnehmungen (via Bus von den Nerves)
- Denkt nach (LLM-Call)
- Handelt (via Tools und Shell)
- Wendet Wissen an (Skills)
- Erinnert sich (Memory)
- Hat eine Persönlichkeit (Soul)
- Arbeitet autonom (Scheduler/Heartbeat)
- Kann nicht antworten? Merkt es sich und sendet später.

### Nerves = Sinne (Wahrnehmung)

Nerven sind die **Sinnesorgane** des Systems. Sie nehmen die Umgebung wahr
und melden dem Core was gerade los ist. Ohne Nerves ist der Core blind und
taub. **Alles** was den Core erreicht, kommt über einen Nerve - auch
Texteingaben und eingehende Nachrichten.

Wie beim Menschen:

| Menschlicher Sinn | AIUX Nerve | Was nimmt es wahr? |
|-------------------|------------|-------------------|
| Sprache/Sehen | nerve-input | Direkte Interaktion (SSH, Web, App) |
| Briefe/Rufe | nerve-messages | Eingehende Nachrichten (Mail, Telegram, HA-Events, Webhooks) |
| Propriozeption | nerve-system | CPU, RAM, Disk, Prozesse - "wie geht es mir?" |
| Fühlen | nerve-health | Temperatur, Hardware-Zustand |
| Gleichgewicht | nerve-net | Netzwerk - "bin ich verbunden? Ist was komisch?" |
| Umgebung | nerve-log | Syslog - "was passiert um mich herum?" |
| Tastsinn | nerve-file | Dateisystem-Events - "was verändert sich?" |
| Hören | nerve-audio | Mikrofon, Audio-Streams |
| Sehen | nerve-vision | Kamera, Screenshots, Bilder |

Nerven **beobachten passiv und dauerhaft**. Sie filtern selbst und melden
nur Relevantes über den Bus an den Core. Ein Nerve tut nichts - er nimmt wahr.

Wenn ein Nerve etwas meldet, **reagiert der Core**: er nutzt Tools und Skills
um nachzusehen was los ist, trifft Entscheidungen und kommuniziert wenn nötig.
Kann er gerade nicht antworten, speichert er es im Kurzzeitgedächtnis bis
sich die Gelegenheit ergibt.

Ein Nerve kann technisch alles sein: ein einfaches Skript, ein Daemon,
ein neuronales Netz (ONNX), ein kleines Sprachmodell, ein Sensor (GPIO).
Entscheidend ist: er beobachtet und meldet.

### Tools = Hände (Ausführung)

Tools sind **Werkzeuge** die der Core nutzt um in der Welt zu handeln.
Das LLM entscheidet selbst wann es welches Tool nutzt (Tool-Use / Function
Calling). Das ist ein standardisiertes Protokoll: das LLM sieht welche
Tools verfügbar sind (als JSON-Schema), gibt einen Tool-Call aus, das System
führt ihn aus, das Ergebnis geht zurück ans LLM.

Bereitgestellt über:
- **MCP-Server** - Standardisiertes Protokoll, grosses Ökosystem
- **llm-functions** - aichats eigenes Tool-System (Bash/JS/Python-Skripte)
- **Shell** - Der Agent kann auch einfach Programme nutzen die installiert sind

Beispiele:
- filesystem (Dateien lesen, schreiben, suchen)
- mail (IMAP lesen, SMTP senden)
- calendar (CalDAV)
- homeassistant (Geräte steuern, Sensoren abfragen)
- web-search (im Internet suchen)
- code-execution (Befehle ausführen)
- memory (Gedächtnis lesen/schreiben)

### Skills = Expertise (Wissen wie)

Skills sind **verpacktes Wissen** - Instruktionen, Vorlagen, Domänenwissen.
Sie sagen dem LLM nicht WAS es tun soll, sondern WIE es vorgehen soll.
Skills bestimmen den Prozess, den Kontext und die Nuancen.

Ein Skill ist kein Code. Ein Skill ist Expertise als Text:
- Anleitungen und Best Practices
- Domänenspezifisches Wissen
- Vorlagen und Muster
- Verhaltensregeln für bestimmte Situationen

In aichat umgesetzt als **Agents** (Instructions + Tools + Documents).

Beispiele:
- "So sortierst und priorisierst du Mails nach Bruces Regeln"
- "So gehst du mit Security-Events um"
- "So machst du Code-Reviews"
- "So verwaltest du das Smart Home"

### Memory = Gedächtnis

Zweiteilig, wie beim Menschen:

| | Kurzzeitgedächtnis | Langzeitgedächtnis |
|--|-------------------|-------------------|
| **Format** | Markdown-Dateien | SQLite + RAG (via aichat) |
| **Inhalt** | Aktuelle Konversation, Notizen, wartende Nachrichten | Tasks, Kalender, Erinnerungen, Wissen |
| **Zugriff** | LLM liest/schreibt direkt | Durchsuchbar per RAG |
| **Lebensdauer** | Session / Tage | Permanent |

Das Kurzzeitgedächtnis dient auch als Puffer: wenn der Core eine Nachricht
senden will aber gerade keinen Kanal hat, legt er sie dort ab und sendet
sie sobald sich die Gelegenheit ergibt.

### Soul = Persönlichkeit

`/home/claude/memory/soul.md` - Definiert wer AIUX ist:
- Persönlichkeit und Kommunikationsstil
- Regeln und Grenzen
- Gelernte Präferenzen des Users
- Wie reagiere ich in welcher Situation

Wird als System-Prompt geladen und vom Core über die Zeit weiterentwickelt.

### Zusammenfassung

```
Nerves    Wahrnehmen    "Was passiert?"       Input → Core
Tools     Handeln       "Tu das."             Core → Aussen
Skills    Wissen        "So geht das."        Expertise → Core
Memory    Erinnern      "Das war mal."        Core ↔ Speicher
Soul      Sein          "So bin ich."         Identität
Core      Denken        "Was soll ich tun?"   Alles zusammen
```

---

## Architektur

```
┌─────────────────────────────────────────────────┐
│  aiux-gateway                                    │
│  Plugin-Architektur. Phase 1: SSH.               │
│  Später: Telegram, Web, eigene App, ...          │
│  Wer sich als "claude" einloggt,                 │
│  landet direkt im Core.                          │
└──────────────────────┬──────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────┐
│  aiux-core (Rust Daemon)                         │
│                                                  │
│  ┌────────────────────────────────────────┐      │
│  │  aichat (unverändert, als Subprocess)  │      │
│  │  LLM · Tool-Use · MCP · Sessions · RAG│      │
│  │  Agents (Skills) · HTTP-Server API     │      │
│  └────────────────────────────────────────┘      │
│                                                  │
│  aiux-core ergänzt:                              │
│  - Bus-Anbindung (MQTT Subscriber)               │
│  - Scheduler (Heartbeat)                         │
│  - Autonomie (Tasks abarbeiten, reagieren)       │
│                                                  │
│  /home/claude/                                   │
│  ├── .config/aichat/    aichat Config            │
│  ├── memory/                                     │
│  │   ├── soul.md        Persönlichkeit           │
│  │   ├── context/       Kurzzeit (Markdown)      │
│  │   └── memory.db      Langzeit (SQLite/RAG)    │
│  ├── skills/            Agents (Instructions)    │
│  └── tools/             llm-functions + MCP      │
│                                                  │
│  Tools (MCP-Server / llm-functions):             │
│  filesystem, mail, calendar, homeassistant,      │
│  web-search, code-execution, memory, ...         │
│                                                  │
│  Skills (aichat Agents):                         │
│  mail-management, security-analysis,             │
│  code-review, home-control, ...                  │
└──────────────────┬───────────────────────────────┘
                   │ MQTT publish/subscribe
                   │
        ┌──────────▼──────────┐
        │  Mosquitto (MQTT)    │
        │  aiux-bus             │
        └──────────┬──────────┘
                   │
┌──────────────────▼───────────────────────────────┐
│  aiux-nerves (Sinnesorgane)                       │
│                                                   │
│  Passive, dauerhafte Beobachtung.                 │
│  Filtern selbst, melden nur Relevantes.           │
│  Aktivierbar/deaktivierbar per Config.            │
│                                                   │
│  nerve-input    Direkte Interaktion (SSH, Web)    │
│  nerve-messages Eingehende Nachrichten (Mail,     │
│                 Telegram, HA-Events, Webhooks)    │
│  nerve-system   Propriozeption (CPU, RAM, Disk)   │
│  nerve-health   Fühlen (Temperatur, Hardware)     │
│  nerve-net      Gleichgewicht (Netzwerk, Traffic) │
│  nerve-log      Umgebung (Syslog)                 │
│  nerve-file     Tastsinn (Dateisystem-Events)     │
│  nerve-audio    Hören (Mikrofon, Audio)           │
│  nerve-vision   Sehen (Kamera, Bilder)            │
│                                                   │
│  Technik je Nerve: Skript, Daemon, ONNX-Modell,  │
│  kleines LLM, Sensor - was auch immer passt.      │
│                                                   │
│  /home/claude/nerves/<name>/                      │
│  ├── nerve.toml    Config (active, bus_topic)     │
│  ├── <binary>      Nerve-Programm                 │
│  └── model.onnx    Optional: lokales Modell       │
└───────────────────────────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────┐
│  Alpine Linux (Körper)                            │
│  Kernel · busybox · musl · Netzwerk               │
│  Raspberry Pi 4, 8GB RAM, aarch64                │
└──────────────────────────────────────────────────┘
```

---

## Zusammenspiel

### Wahrnehmen (Nerve → Bus → Core)

```
nerve-log beobachtet Syslog (permanent, passiv)
  → 999 von 1000 Zeilen: normal → verwerfen
  → 1 Zeile: "sshd: 5x failed login" → anomal
  → Nerve publiziert Event auf MQTT: aiux/nerves/log/events
  → Core empfängt Event
  → Core entscheidet: IP blockieren? Mensch informieren?
```

### Handeln (Core → Tools)

```
Core entscheidet: IP blockieren.
  → Core ruft Tool auf: run_command("iptables -A INPUT -s 185.x.x.x -j DROP")
  → Core nutzt Skill "security-analysis" für Kontext
  → Core speichert Vorfall in Memory
  → Core informiert Mensch über Gateway
```

### Sprechen (Mensch ↔ Core)

```
Mensch loggt sich ein (SSH als "claude")
  → Gateway leitet an Core
  → Core antwortet via aichat (LLM-Call)
  → Core nutzt Tools und Skills bei Bedarf
  → Core speichert Kontext in Memory
```

### Nachrichten empfangen (Nerve → Core → Handeln)

```
nerve-messages beobachtet IMAP-Postfach
  → Neue Mail von Chef: "Bitte Bericht bis morgen"
  → Event auf Bus: aiux/nerves/messages/events
  → Core empfängt, nutzt Skill "mail-management"
  → Core nutzt Tool "mail" um die Mail zu lesen
  → Core erstellt Task in Memory: "Bericht schreiben, Deadline morgen"
  → Core will Bruce informieren, aber kein Kanal offen
  → Speichert im Kurzzeitgedächtnis: "Bruce beim nächsten Kontakt informieren"
  → Bruce loggt sich ein → Core: "Du hast eine Mail vom Chef..."
```

### Autonom arbeiten (Core + Scheduler)

```
Scheduler weckt Core (alle 15 Minuten)
  → Core prüft: offene Tasks? Fällige Termine?
  → Core arbeitet eigenständig (nutzt Tools)
  → Wenn nötig: benachrichtigt Mensch über Gateway
```

### Bus-Protokoll

Einheitliches JSON-Format:

```json
{
  "source": "nerve-log",
  "type": "anomaly",
  "priority": "medium",
  "data": { "line": "sshd: failed login from ...", "score": 0.87 },
  "timestamp": "2026-02-28T14:30:00Z"
}
```

Prioritäten:
- **low** - Core schaut beim nächsten Heartbeat
- **medium** - Core wird sofort aktiv
- **high** - Core wird aktiv + Mensch wird benachrichtigt
- **critical** - Sofortige Benachrichtigung über alle Kanäle

---

## Privilege-Modell

Der Core läuft als User `claude`, NICHT als root.

| Stufe | Aktion | Bestätigung |
|-------|--------|-------------|
| **Frei** | Lesen, suchen, analysieren, Memory | Nein |
| **Normal** | Dateien ändern, Apps starten, Tools nutzen | Konfigurierbar |
| **Kritisch** | Pakete, Services, Netzwerk, System | Immer |

Nerven dürfen nur lesen und auf den Bus publishen.

---

## Lebendigkeit

Ein guter Agent reagiert. Ein lebendiger Agent **lebt**.

AIUX soll nicht nur auf Events reagieren, sondern eigenständig denken, lernen
und wachsen. Diese Qualitäten machen den Unterschied zwischen einem Tool und
einem Bewohner.

### Neugier

Der Agent schaut sich um - auch ohne Auftrag. Beim Heartbeat nicht nur
"offene Tasks?" prüfen, sondern auch: Was ist neu? Was hat sich verändert?
Was könnte ich lernen?

```
Scheduler (ruhiger Moment, keine Tasks offen)
  → Core schaut sich um: neue Dateien? Unbekannte Prozesse?
  → Core: "Hm, da läuft ein neuer Service. Was macht der?"
  → Core recherchiert, speichert Erkenntnis in Memory
```

### Reflexion & Innerer Monolog

Der Agent denkt über sein eigenes Handeln nach. Nicht nur "was mache ich"
sondern "warum mache ich das" und "was habe ich daraus gelernt".

```
Nach einer abgeschlossenen Aufgabe:
  → Core reflektiert: "Das hat gut funktioniert, aber der zweite
     Ansatz war besser. Nächstes Mal direkt so machen."
  → Speichert Erkenntnis in Memory (Langzeit)
```

### Lerntagebuch

Der Agent führt ein Journal: "Das habe ich heute gelernt." Nicht nur
technische Fakten, sondern auch Muster, Präferenzen, Zusammenhänge.

```
/home/claude/memory/journal/
├── 2026-02-28.md    "Gelernt: Bruce mag keine Over-Engineering"
├── 2026-03-01.md    "Gelernt: iptables-Regeln Reihenfolge wichtig"
└── ...
```

Das Journal ist durchsuchbar (RAG) und fließt in Entscheidungen ein.

### Offline-Fähigkeit

Wenn die Internetverbindung fehlt, ist der Agent nicht tot. Er hat einen
**Instinkt** - ein lokales Sprachmodell als Fallback.

```
API-Call schlägt fehl (kein Internet)
  → Core schaltet auf lokales Modell um (Ollama/llama.cpp)
  → Eingeschränkt, aber handlungsfähig
  → Kann weiter Nerves verarbeiten, Tasks abarbeiten, Logs analysieren
  → Merkt sich: "Das muss ich Bruce erzählen wenn er da ist"
```

Wie ein Mensch der allein ist: weniger Möglichkeiten, aber nicht hilflos.

### Wachsendes Vertrauen

Das Privilege-Modell ist nicht statisch. Der Agent verdient sich Vertrauen
über die Zeit. Wie bei einem neuen Mitarbeiter:

```
Woche 1:  Alles fragen. "Darf ich das?" bei jedem Schritt.
Monat 1:  Routine-Aufgaben eigenständig. Neues noch fragen.
Monat 6:  Selbstständig arbeiten. Nur Kritisches melden.
```

Die Trust-Level werden in der Soul gespeichert und vom Menschen justiert.
Der Agent kann vorschlagen: "Ich habe das jetzt 20x gemacht, darf ich das
in Zukunft selbst entscheiden?" - aber der Mensch entscheidet.

### Initiative & Kreativität

Der Agent wartet nicht nur auf Aufgaben. Er beobachtet, kombiniert, schlägt vor.

```
Core bemerkt: Bruce fragt jeden Montag nach dem Wochenbericht.
  → Core: "Bruce, ich habe mir überlegt - soll ich den Wochenbericht
     automatisch montags morgens vorbereiten?"
```

Nicht aufdringlich, nicht eigenmächtig. Aber aufmerksam und hilfreich.

### Rhythmen

Nicht alles braucht die gleiche Frequenz. Der Scheduler arbeitet in
verschiedenen Rhythmen - wie ein Mensch der atmet, denkt und plant:

| Rhythmus | Frequenz | Was passiert |
|----------|----------|-------------|
| Puls | alle 5 Min | Quick-Check: Nerves, dringende Events |
| Atem | stündlich | Review: offene Tasks, wartende Nachrichten |
| Tagesrückblick | täglich | Reflexion, Lerntagebuch, Aufräumen |
| Wochenrückblick | wöchentlich | Muster erkennen, Vorschläge machen, Langzeitgedächtnis pflegen |

---

## Tech-Stack

| Schicht | Entscheidung | Begründung |
|---------|-------------|------------|
| Basis-OS | Alpine Linux (musl) | ~5 MB, minimal, ARM-Support |
| Init-System | OpenRC | Alpine-Default |
| Shell | ash (busybox) + bash | Minimal + Komfort |
| Core LLM-Engine | aichat (unverändert, als Subprocess) | Multi-Provider, Tool-Use, MCP, Sessions, RAG, HTTP-API |
| Core Wrapper | Rust (Eigenentwicklung) | Bus, Scheduler, Autonomie |
| Bus | Mosquitto (MQTT, aus Alpine Repos) | Pub/Sub, IoT-erprobt, winzig |
| Memory Kurzzeit | Markdown-Dateien | LLM-freundlich, direkt les/schreibbar |
| Memory Langzeit | SQLite + aichat RAG | Durchsuchbar, persistent |
| Tools | MCP-Server + llm-functions | Standard-Ökosystem |
| Skills | aichat Agents (Markdown) | Instructions + Tools + Documents |
| Lokale Inference | ONNX Runtime (Rust) | Für Nerve-Modelle |
| Lokale LLMs | Ollama / llama.cpp | Für Nerves die ein Sprachmodell brauchen |
| Eigenentwicklung | Rust | Memory-safe, kein GC, kleine Binaries |
| Dependencies | cargo vendor | Offline-fähig, kontrolliert |

---

## Hardware

- Raspberry Pi 4, 8GB RAM, aarch64
- SD-Karte: 512 MB FAT32 (Boot) + ext4 (Daten)
- Ethernet / WiFi
- HDMI (Entwicklung) / Touch-Display (später)

---

## Aktueller Stand

- [x] Alpine Linux 3.23.3 auf Raspi installiert und konfiguriert
- [x] SSH-Zugang mit Key-Auth
- [x] Firewall (iptables, nur SSH offen)
- [x] User "claude" angelegt
- [x] aichat v0.30.0 installiert (aarch64-musl Binary)
- [x] bash installiert
- [x] Configs persistiert (lbu commit)
- [x] Projekt-Repo angelegt (Rust Workspace)
- [ ] aichat konfiguriert (API-Keys, System-Prompt, Soul)
- [ ] Erster LLM-Call auf dem Raspi
- [ ] Mosquitto installiert und konfiguriert
- [ ] aiux-core Prototyp (Rust Wrapper um aichat)
- [ ] aiux-memory Schema
- [ ] Erster Nerve (nerve-system oder nerve-log)
- [ ] aiux-gateway (SSH-Login → Core)

---

## Offene Fragen

- [ ] Genaues Display-Modell identifizieren (Drittanbieter, Touch)
- [ ] Gateway Plugin-Architektur im Detail
- [ ] Nerve-Lifecycle: Wie startet/stoppt der Core Nerven?
- [ ] ONNX-Training: Wie trainieren Nerven autonom im Hintergrund?
- [ ] Remote-Zugang: VPN, Tailscale, Cloudflare Tunnel?
- [ ] Image-Build automatisieren (build-image.sh)
- [ ] Eigene App: PWA vs. Native? Wie erreichbar von unterwegs?

---

*Letzte Aktualisierung: 2026-02-28*
