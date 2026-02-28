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
- **Geschichtet** - Klare Trennung: Koerper, Nerven, Bus, Core, Mensch
- **Sicher** - LLM hat keinen Root-Zugang, klares Privilege-Modell
- **Erweiterbar** - Nerven, Skills und Tools als Plugins
- **Autonom** - System arbeitet eigenstaendig, meldet sich wenn noetig
- **Plattformunabhaengig** - Laeuft ueberall wo Rust kompiliert

---

## Grundgedanke

Der Agent lebt in seinem System wie ein Mensch an seinem Rechner. Er hat ein
OS, kann Software installieren, Programme nutzen, Dateien verwalten - alles
was ein User am Terminal tun kann.

Nicht alles braucht ein spezielles Framework. Manchmal installiert der Agent
einfach ein Programm und benutzt es. Die AIUX-Konzepte (Nerves, Tools, Skills,
Memory) sind das, was ihn **ueber einen normalen User hinaushebt**.

---

## Konzepte

### Core = Das Bewusstsein

Der Core ist der Agent selbst. Er denkt, entscheidet, handelt.
Ohne seine Nerven ist er blind und taub - **alles** kommt ueber Nerves rein,
auch Texteingaben und Nachrichten.

Der Core:
- Empfaengt Wahrnehmungen (via Bus von den Nerves)
- Denkt nach (LLM-Call)
- Handelt (via Tools und Shell)
- Wendet Wissen an (Skills)
- Erinnert sich (Memory)
- Hat eine Persoenlichkeit (Soul)
- Arbeitet autonom (Scheduler/Heartbeat)
- Kann nicht antworten? Merkt es sich und sendet spaeter.

### Nerves = Sinne (Wahrnehmung)

Nerven sind die **Sinnesorgane** des Systems. Sie nehmen die Umgebung wahr
und melden dem Core was gerade los ist. Ohne Nerves ist der Core blind und
taub. **Alles** was den Core erreicht, kommt ueber einen Nerve - auch
Texteingaben und eingehende Nachrichten.

Wie beim Menschen:

| Menschlicher Sinn | AIUX Nerve | Was nimmt es wahr? |
|-------------------|------------|-------------------|
| Sprache/Sehen | nerve-input | Direkte Interaktion (SSH, Web, App) |
| Briefe/Rufe | nerve-messages | Eingehende Nachrichten (Mail, Telegram, HA-Events, Webhooks) |
| Propriozeption | nerve-system | CPU, RAM, Disk, Prozesse - "wie geht es mir?" |
| Fuehlen | nerve-health | Temperatur, Hardware-Zustand |
| Gleichgewicht | nerve-net | Netzwerk - "bin ich verbunden? Ist was komisch?" |
| Umgebung | nerve-log | Syslog - "was passiert um mich herum?" |
| Tastsinn | nerve-file | Dateisystem-Events - "was veraendert sich?" |
| Hoeren | nerve-audio | Mikrofon, Audio-Streams |
| Sehen | nerve-vision | Kamera, Screenshots, Bilder |

Nerven **beobachten passiv und dauerhaft**. Sie filtern selbst und melden
nur Relevantes ueber den Bus an den Core. Ein Nerve tut nichts - er nimmt wahr.

Ein Nerve kann technisch alles sein: ein einfaches Skript, ein Daemon,
ein neuronales Netz, ein kleines Sprachmodell, ein Sensor.
Entscheidend ist: er beobachtet und meldet.

### Tools = Haende (Ausfuehrung)

Tools sind **Werkzeuge** die der Core nutzt um in der Welt zu handeln.
Das LLM entscheidet selbst wann es welches Tool nutzt (Tool-Use / Function
Calling). Das ist ein standardisiertes Protokoll: das LLM sieht welche
Tools verfuegbar sind (als JSON-Schema), gibt einen Tool-Call aus, das System
fuehrt ihn aus, das Ergebnis geht zurueck ans LLM.

Beispiele:
- filesystem (Dateien lesen, schreiben, suchen)
- mail (IMAP lesen, SMTP senden)
- calendar (CalDAV)
- homeassistant (Geraete steuern, Sensoren abfragen)
- web-search (im Internet suchen)
- shell (Befehle ausfuehren)
- memory (Gedaechtnis lesen/schreiben)

### Skills = Expertise (Wissen wie)

Skills sind **verpacktes Wissen** - Instruktionen, Vorlagen, Domaenenwissen.
Sie sagen dem LLM nicht WAS es tun soll, sondern WIE es vorgehen soll.
Skills bestimmen den Prozess, den Kontext und die Nuancen.

Ein Skill ist kein Code. Ein Skill ist Expertise als Text:
- Anleitungen und Best Practices
- Domaenenspezifisches Wissen
- Vorlagen und Muster
- Verhaltensregeln fuer bestimmte Situationen

Beispiele:
- "So sortierst und priorisierst du Mails nach Bruces Regeln"
- "So gehst du mit Security-Events um"
- "So machst du Code-Reviews"
- "So verwaltest du das Smart Home"

### Memory = Gedaechtnis

Zweiteilig, wie beim Menschen:

| | Kurzzeitgedaechtnis | Langzeitgedaechtnis |
|--|-------------------|-------------------|
| **Format** | Markdown-Dateien | SQLite + RAG |
| **Inhalt** | Aktuelle Konversation, Notizen, wartende Nachrichten | Tasks, Kalender, Erinnerungen, Wissen |
| **Zugriff** | LLM liest/schreibt direkt | Durchsuchbar per RAG |
| **Lebensdauer** | Session / Tage | Permanent |

Das Kurzzeitgedaechtnis dient auch als Puffer: wenn der Core eine Nachricht
senden will aber gerade keinen Kanal hat, legt er sie dort ab und sendet
sie sobald sich die Gelegenheit ergibt.

### Soul = Persoenlichkeit

`soul.md` - Definiert wer AIUX ist:
- Persoenlichkeit und Kommunikationsstil
- Regeln und Grenzen
- Gelernte Praeferenzen des Users
- Wie reagiere ich in welcher Situation

Wird als System-Prompt geladen und vom Core ueber die Zeit weiterentwickelt.

### Zusammenfassung

```
Nerves    Wahrnehmen    "Was passiert?"       Input -> Core
Tools     Handeln       "Tu das."             Core -> Aussen
Skills    Wissen        "So geht das."        Expertise -> Core
Memory    Erinnern      "Das war mal."        Core <-> Speicher
Soul      Sein          "So bin ich."         Identitaet
Core      Denken        "Was soll ich tun?"   Alles zusammen
```

---

## Zusammenspiel

### Wahrnehmen (Nerve -> Bus -> Core)

```
nerve-log beobachtet Syslog (permanent, passiv)
  -> 999 von 1000 Zeilen: normal -> verwerfen
  -> 1 Zeile: "sshd: 5x failed login" -> anomal
  -> Nerve publiziert Event auf Bus
  -> Core empfaengt Event
  -> Core entscheidet: IP blockieren? Mensch informieren?
```

### Handeln (Core -> Tools)

```
Core entscheidet: IP blockieren.
  -> Core ruft Tool auf: shell("iptables -A INPUT -s 185.x.x.x -j DROP")
  -> Core nutzt Skill "security-analysis" fuer Kontext
  -> Core speichert Vorfall in Memory
  -> Core informiert Mensch ueber Gateway
```

### Sprechen (Mensch <-> Core)

```
Mensch verbindet sich (SSH, Telegram, Web, App)
  -> Gateway leitet an Core
  -> Core antwortet (LLM-Call)
  -> Core nutzt Tools und Skills bei Bedarf
  -> Core speichert Kontext in Memory
```

### Nachrichten empfangen (Nerve -> Core -> Handeln)

```
nerve-messages beobachtet IMAP-Postfach
  -> Neue Mail von Chef: "Bitte Bericht bis morgen"
  -> Event auf Bus
  -> Core empfaengt, nutzt Skill "mail-management"
  -> Core erstellt Task in Memory: "Bericht schreiben, Deadline morgen"
  -> Bruce nicht erreichbar -> Kurzzeitgedaechtnis
  -> Bruce loggt sich ein -> Core: "Du hast eine Mail vom Chef..."
```

---

## Privilege-Modell

Der Core laeuft als unprivilegierter User, NICHT als root.

| Stufe | Aktion | Bestaetigung |
|-------|--------|-------------|
| **Frei** | Lesen, suchen, analysieren, Memory | Nein |
| **Normal** | Dateien aendern, Apps starten, Tools nutzen | Konfigurierbar |
| **Kritisch** | Pakete, Services, Netzwerk, System | Immer |

Nerven duerfen nur lesen und auf den Bus publishen.

---

## Lebendigkeit

Ein guter Agent reagiert. Ein lebendiger Agent **lebt**.

AIUX soll nicht nur auf Events reagieren, sondern eigenstaendig denken, lernen
und wachsen. Diese Qualitaeten machen den Unterschied zwischen einem Tool und
einem Bewohner.

### Neugier

Der Agent schaut sich um - auch ohne Auftrag. Beim Heartbeat nicht nur
"offene Tasks?" pruefen, sondern auch: Was ist neu? Was hat sich veraendert?
Was koennte ich lernen?

### Reflexion & Innerer Monolog

Der Agent denkt ueber sein eigenes Handeln nach. Nicht nur "was mache ich"
sondern "warum mache ich das" und "was habe ich daraus gelernt".

### Lerntagebuch

Der Agent fuehrt ein Journal: "Das habe ich heute gelernt." Nicht nur
technische Fakten, sondern auch Muster, Praeferenzen, Zusammenhaenge.
Das Journal ist durchsuchbar (RAG) und fliesst in Entscheidungen ein.

### Offline-Faehigkeit

Wenn die Internetverbindung fehlt, ist der Agent nicht tot. Er hat einen
**Instinkt** - ein lokales Sprachmodell als Fallback. Eingeschraenkt,
aber handlungsfaehig.

### Wachsendes Vertrauen

Das Privilege-Modell ist nicht statisch. Der Agent verdient sich Vertrauen
ueber die Zeit. Die Trust-Level werden in der Soul gespeichert und vom
Menschen justiert.

### Initiative & Kreativitaet

Der Agent wartet nicht nur auf Aufgaben. Er beobachtet, kombiniert, schlaegt vor.
Nicht aufdringlich, nicht eigenmaechtig. Aber aufmerksam und hilfreich.

### Rhythmen

| Rhythmus | Frequenz | Was passiert |
|----------|----------|-------------|
| Puls | alle 5 Min | Quick-Check: Nerves, dringende Events |
| Atem | stuendlich | Review: offene Tasks, wartende Nachrichten |
| Tagesrueckblick | taeglich | Reflexion, Lerntagebuch, Aufraeumen |
| Wochenrueckblick | woechentlich | Muster erkennen, Vorschlaege machen |

---

## Offene Fragen

- Gateway Plugin-Architektur im Detail
- Nerve-Lifecycle: Wie startet/stoppt der Core Nerven?
- Remote-Zugang: VPN, Tailscale, Cloudflare Tunnel?
- Eigene App: PWA vs. Native?

---

*Letzte Aktualisierung: 2026-02-28*
