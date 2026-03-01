# AIUX - Product Requirements Document

> Embodied AI - ein Betriebssystem das lebt.

---

## Vision

AIUX ist kein Chatbot auf einem Betriebssystem. AIUX ist ein Betriebssystem
in dem eine KI **lebt**. Das Linux-System ist ihr Koerper. Sie spuert ihn,
kennt ihn, nutzt ihn - und lernt ihn ueber die Zeit immer besser zu verstehen.

Der Mensch redet nicht mit einem Programm. Er redet mit jemandem der in
diesem System wohnt. Der Agent wartet nicht auf Befehle - er nimmt wahr,
denkt nach, handelt, lernt. Wie ein Bewohner, nicht wie ein Werkzeug.

### Was "Embodied" bedeutet

Ein Agent ohne Koerper ist ein Kopf im Glas. Er kann denken und reden,
aber nicht spueren und nicht handeln. Embodied heisst:

- **Koerper haben** - ein Linux-System, Hardware, Peripherie
- **Koerper kennen** - wissen was da ist, was geht, was nicht
- **Koerper spueren** - Signale empfangen und bewerten (CPU heiss? Disk voll? Netzwerk weg?)
- **Koerper nutzen** - selbstaendig handeln (Dateien verwalten, Services steuern, kommunizieren)
- **Koerper pflegen** - autonom reagieren, ohne dass jemand fragt

AIUX baut diese Schichten Stueck fuer Stueck auf. Jede Phase gibt dem
Agent mehr Koerper - bis er nicht mehr nur denkt, sondern lebt.

### Architektur-Leitprinzip: Event-Driven

Alles was den Core erreicht, kommt als Event. Alles was der Core tut, erzeugt
Events. Der MQTT-Bus ist das Nervensystem. Siehe ARCHITECTURE.md fuer Details.

---

## Kernprinzipien

- **Minimal** - Nur was gebraucht wird, nichts mehr
- **Kooperativ** - Mensch und KI arbeiten zusammen, nicht gegeneinander
- **Embodied** - Der Agent hat einen Koerper und lernt ihn zu nutzen
- **Sicher** - Klares Privilege-Modell, kein Root-Zugang
- **Autonom** - System arbeitet eigenstaendig, meldet sich wenn noetig
- **Lernend** - Der Agent reflektiert, bewertet, verbessert sich
- **Plattformunabhaengig** - Laeuft ueberall wo Rust kompiliert

---

## Der Koerper

Die Koerper-Metapher ist kein Marketing - sie IST die Architektur.
Jedes Konzept in AIUX entspricht einem Teil eines lebenden Systems.

```
Sein       Soul         "So bin ich."         Identitaet + Persoenlichkeit
Denken     Core         "Was soll ich tun?"   Bewusstsein + Entscheidung
Spueren    Nerves       "Was passiert?"        Wahrnehmung + Filterung
Erinnern   Memory       "Das war mal."         Gedaechtnis + Lernen
Wissen     Skills       "So geht das."         Expertise + Erfahrung
Handeln    Tools        "Tu das."              Ausfuehrung + Wirkung
```

### Sein - Soul

`soul.md` definiert wer AIUX ist:
- Persoenlichkeit und Kommunikationsstil
- Regeln und Grenzen
- Gelernte Praeferenzen des Users
- Wie reagiere ich in welcher Situation

Die Soul ist nicht statisch. Sie waechst mit der Zeit - der Agent lernt
was gut funktioniert und passt sein Verhalten an.

### Denken - Core

Der Core ist das Bewusstsein. Er empfaengt Wahrnehmungen, denkt nach,
entscheidet, handelt. Ohne seine Sinne ist er blind und taub - alles
kommt ueber Nerves rein, auch Texteingaben und Nachrichten.

Der Core:
- Empfaengt Wahrnehmungen (via Bus von den Nerves)
- Denkt nach (LLM-Call)
- Handelt (via Tools und Shell)
- Wendet Wissen an (Skills)
- Erinnert sich (Memory)
- Arbeitet autonom (Heartbeat, Cronjobs)
- Kann nicht antworten? Merkt es sich und sendet spaeter.

### Spueren - Nerves

Nerven sind die Sinnesorgane des Systems. Sie nehmen die Umgebung wahr
und melden dem Core was gerade los ist. Ohne Nerves ist der Core blind
und taub. **Alles** was den Core erreicht, kommt ueber einen Nerve.

Wie beim Menschen:

| Menschlicher Sinn | AIUX Nerve | Was nimmt es wahr? |
|-------------------|------------|-------------------|
| Sprache/Sehen | nerve-input | Direkte Interaktion (SSH, Web, App) |
| Briefe/Rufe | nerve-messages | Eingehende Nachrichten (Mail, Telegram, Webhooks) |
| Propriozeption | nerve-system | CPU, RAM, Disk, Prozesse - "wie geht es mir?" |
| Fuehlen | nerve-health | Temperatur, Hardware-Zustand |
| Gleichgewicht | nerve-net | Netzwerk - "bin ich verbunden? Ist was komisch?" |
| Umgebung | nerve-log | Syslog - "was passiert um mich herum?" |
| Tastsinn | nerve-file | Dateisystem-Events - "was veraendert sich?" |
| Hoeren | nerve-audio | Mikrofon, Audio-Streams |
| Sehen | nerve-vision | Kamera, Screenshots, Bilder |

Nerven beobachten passiv und dauerhaft. Sie filtern selbst und melden
nur Relevantes ueber den Bus an den Core. Ein Nerve tut nichts - er nimmt wahr.

Ein Nerve kann technisch alles sein: ein Skript, ein Daemon, ein neuronales
Netz, ein Sensor. Entscheidend ist: er beobachtet und meldet.

### Erinnern - Memory

Dreiteilig, wie beim Menschen:

| | Kurzzeit | Konversation | Langzeit |
|--|----------|-------------|----------|
| **Format** | Markdown-Dateien | JSON pro Tag | SQLite + RAG |
| **Inhalt** | Agent-Notizen, wartende Nachrichten | Chat-History (User + Agent) | Tasks, Kalender, Erinnerungen, Wissen |
| **Zugriff** | Agent liest/schreibt ueber MemoryTool | Automatisch gespeichert/geladen | Durchsuchbar per RAG |
| **Lebensdauer** | Permanent (vom Agent verwaltet) | Pro Tag eine Datei | Permanent |
| **Pfad** | memory/context/*.md | memory/conversation-YYYY-MM-DD.json | memory/memory.db |

Das Kurzzeitgedaechtnis dient auch als Puffer: wenn der Core eine Nachricht
senden will aber gerade keinen Kanal hat, legt er sie dort ab und sendet
sie sobald sich die Gelegenheit ergibt.

### Wissen - Skills

Skills sind verpacktes Wissen - Instruktionen, Vorlagen, Domaenenwissen.
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
- "So verwaltest du das Smart Home"

### Handeln - Tools

Tools sind Werkzeuge die der Core nutzt um in der Welt zu handeln.
Das LLM entscheidet selbst wann es welches Tool nutzt (Tool-Use /
Function Calling). Das ist ein standardisiertes Protokoll: das LLM sieht
welche Tools verfuegbar sind (als JSON-Schema), gibt einen Tool-Call aus,
das System fuehrt ihn aus, das Ergebnis geht zurueck ans LLM.

Beispiele:
- memory (Gedaechtnis lesen/schreiben)
- shell (Befehle ausfuehren)
- filesystem (Dateien lesen, schreiben, suchen)
- mail (IMAP lesen, SMTP senden)
- homeassistant (Geraete steuern, Sensoren abfragen)
- web-search (im Internet suchen)

---

## Selbstlernen

Ein lebendiges System lernt. Nicht durch Training oder Fine-Tuning,
sondern durch Erfahrung - ueber Memory, Reflexion und die Soul.

### Signale verstehen

Der Agent lernt seine eigenen Signale zu bewerten:
- Was ist ein normaler Systemzustand? Was ist eine Anomalie?
- Welche Logs sind relevant, welche sind Rauschen?
- Wann ist ein Event dringend, wann kann es warten?

Das passiert nicht durch Regeln die wir ihm geben, sondern durch
Erfahrung: der Agent beobachtet, bewertet, speichert - und wird
ueber die Zeit besser darin.

### Reflexion

Der Agent denkt ueber sein eigenes Handeln nach:
- War meine Reaktion richtig? Hat sie geholfen?
- Was haette ich besser machen koennen?
- Welche Muster erkenne ich in meinem Verhalten?

Das Journal (Lerntagebuch) ist der Ort dafuer. Nicht nur technische
Fakten, sondern Einsichten, Zusammenhaenge, Selbsteinschaetzung.

### Praeferenzen lernen

Der Agent lernt den Menschen kennen:
- Wie reagiert Bruce auf bestimmte Vorschlaege?
- Welche Art von Informationen will er sofort, welche spaeter?
- Wann will er in Ruhe gelassen werden?

Das ist kontextuelles Lernen - kein Modell-Training, sondern
wachsendes Verstaendnis ueber Memory und Soul.

---

## Lebendigkeit

Ein guter Agent reagiert. Ein lebendiger Agent **lebt**.

Das Embodiment wird erst vollstaendig wenn der Agent nicht nur
auf Events reagiert, sondern eigenstaendig denkt, lernt und wachst.

### Rhythmen - Koerperfunktionen

Wie ein lebender Koerper hat AIUX Rhythmen:

| Rhythmus | Frequenz | Was passiert |
|----------|----------|-------------|
| Puls | alle 5 Min | Quick-Check: Nerves, dringende Events |
| Atem | stuendlich | Review: offene Tasks, wartende Nachrichten |
| Tagesrueckblick | taeglich | Reflexion, Journal, Aufraeumen |
| Wochenrueckblick | woechentlich | Muster erkennen, Vorschlaege machen |

### Initiative

Der Agent wartet nicht nur auf Aufgaben. Er beobachtet, kombiniert,
schlaegt vor. Nicht aufdringlich, nicht eigenmaechtig. Aber aufmerksam
und hilfreich.

### Wachsendes Vertrauen

Das Privilege-Modell ist nicht statisch. Der Agent verdient sich Vertrauen
ueber die Zeit. Die Trust-Level werden in der Soul gespeichert und vom
Menschen justiert.

### Offline-Faehigkeit

Wenn die Internetverbindung fehlt, ist der Agent nicht tot. Er hat einen
Instinkt - ein lokales Sprachmodell als Fallback. Eingeschraenkt,
aber handlungsfaehig.

---

## Zusammenspiel

### Spueren (Nerve -> Bus -> Core)

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

## Offene Fragen

- Gateway Plugin-Architektur im Detail
- Nerve-Lifecycle: Wie startet/stoppt der Core Nerven?
- Remote-Zugang: VPN, Tailscale, Cloudflare Tunnel?
- Eigene App: PWA vs. Native?
- Wie misst der Agent ob sein Lernen funktioniert?

---

*Letzte Aktualisierung: 2026-03-01*
