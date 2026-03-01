# AIUX - Abweichungen: Plan vs. Realitaet

> Stand: 2026-03-01, nach Phase 4.3
> Zweck: Klarheit schaffen, Docs an Realitaet anpassen oder bewusst offen lassen.

---

## 1. Input ist kein Nerve

**Plan (PRD):** "Alles was den Core erreicht, kommt ueber einen Nerve - auch Texteingaben."
`nerve-input` ist explizit als Sinn definiert.

**Realitaet:** REPL liest direkt von stdin in main.rs. Kein Nerve, kein Bus, kein Event.

**Handlung:** Docs anpassen. Die REPL ist bewusst direkt - das ist Phase 3.
Nerve-Input kommt in Phase 6/7 (Gateway). In der Architektur klarstellen:
"Aktuell direkt, spaeter ueber nerve-input."

---

## 2. Boot-Sequence: Journal fehlt

**Plan (ARCHITECTURE):** `soul.md -> user.md -> journal/heute -> journal/gestern`

**Realitaet:** `soul.md -> user.md -> context/*.md` - Journal wird nicht geladen.

**Handlung:** Architektur anpassen. Journal-Loading ist Phase 9 (Lebendigkeit/Rhythmen).
Boot-Sequence dokumentieren wie sie JETZT ist, mit Hinweis auf geplante Erweiterung.

---

## 3. Conversation-JSON ist ungeplante Kategorie

**Plan (PRD):** Kurzzeit = Markdown (context/), Langzeit = SQLite + RAG.
Konversations-History wird nicht als eigene Kategorie beschrieben.

**Realitaet:** `conversation-YYYY-MM-DD.json` - eine dritte Speicherform die in keinem
Dokument vorkommt.

**Handlung:** PRD und Architektur ergaenzen. Memory-Modell hat jetzt drei Teile:
- Kurzzeit: context/*.md (Agent-Notizen)
- Konversation: conversation-*.json (Tages-History)
- Langzeit: SQLite + RAG (noch nicht gebaut)

---

## 4. Tools sind hardcoded, nicht Plugin-basiert

**Plan (PRD):** `home/tools/` enthaelt Tool-Definitionen. Tools als Plugins.

**Realitaet:** MemoryTool ist in `core/src/memory.rs` hardcoded, direkt im Agent-Builder
registriert. `home/tools/` ist leer und wird nicht gelesen.

**Handlung:** Docs anpassen. Aktuell sind Tools Rust-Code im Core. Plugin-System ist
Fernziel. `home/tools/` aus der Verzeichnisstruktur entfernen oder als "geplant" markieren.

---

## 5. Skills-Verzeichnis ungenutzt

**Plan:** Skills als Markdown, Skill-Loader beim Start, in LLM-Calls eingebunden.

**Realitaet:** `home/skills/` ist leer, wird nirgends geladen. Phase 8.

**Handlung:** In Verzeichnisstruktur als "geplant (Phase 8)" markieren. Kein Code-Aenderung.

---

## 6. Session-Modell vereinfacht

**Plan (ARCHITECTURE):** Alter Text vermischte Session-Konzept mit Heartbeat.

**Realitaet:** Es gibt keine Sessions. Es gibt eine REPL mit History.
Heartbeat existiert nicht (Phase 5.3).

**Handlung:** Session-Modell Abschnitt aus Architektur entfernt. Erledigt.

---

## 7. Tech-Stack Abweichungen

| Crate | Architektur-Dok | Cargo.toml | Status |
|-------|----------------|------------|--------|
| rig-core | ja | ja | eingebaut |
| chrono | nein | ja | eingebaut, nicht dokumentiert |
| schemars | nein | ja | eingebaut (Memory-Tool Schema) |
| thiserror | nein | ja | eingebaut (Memory-Tool Errors) |
| rig-sqlite | ja | nein | geplant (Phase 4.4) |
| rumqttc | ja | nein | geplant (Phase 6) |
| tract-onnx | ja | nein | geplant (Fernziel) |
| tokio-cron-scheduler | ja | nein | geplant (Phase 5.3) |
| pulldown-cmark | ja | nein | unklar ob noch gebraucht |

**Handlung:** Tech-Stack Tabelle in Architektur zweiteilen:
"Aktuell eingebaut" vs. "Geplant". pulldown-cmark pruefen ob noetig.

---

## 8. Embodiment fehlt als Leitidee

**Plan (PRD):** "Embodied AI" steht im Untertitel.

**Realitaet:** Die PRD behandelte es als Tagline, nicht als Leitidee.
Die Konzepte (Nerves, Tools, Memory) wurden flach aufgelistet statt
als Teile eines Koerpers erklaert. Selbstlernen fehlte komplett.
AIUX war de facto ein LLM-Agent als App auf Linux, nicht ein AI-OS.

**Handlung:** PRD komplett umgebaut. Embodiment ist jetzt der rote Faden:
- Vision erklaert was "Embodied" konkret bedeutet
- Konzepte sind als "Der Koerper" organisiert (Sein, Denken, Spueren, Erinnern, Wissen, Handeln)
- Neuer Abschnitt "Selbstlernen" (Signale bewerten, Reflexion, Praeferenzen)
- Lebendigkeit als Konsequenz des Embodiment, nicht als eigenes Kapitel
- CLAUDE.md ergaenzt mit Embodiment-Kontext

---

## Zusammenfassung: Was tun?

Alle Abweichungen sind erwartbar - wir haben Phase 3+4 gebaut, die Docs beschreiben
das Zielbild bis Phase 9. Das Problem ist nicht der Code, sondern dass die Docs nicht
klar machen was JETZT ist vs. was SPAETER kommt.

### Aenderungen gemacht in:

| Dokument | Was geaendert |
|----------|--------------|
| **ARCHITECTURE.md** | Boot-Sequence, Session-Modell entfernt, Tech-Stack zweigeteilt, Verzeichnisstruktur, Aktueller Stand |
| **PRD.md** | Komplett umgebaut: Embodiment als Leitidee, Koerper-Metapher, Selbstlernen |
| **ROADMAP.md** | Phase 4.1-4.3 als erledigt markiert |
| **CLAUDE.md** | Tech-Stack vervollstaendigt, Embodiment-Kontext, aktueller Stand |
