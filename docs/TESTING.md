# AIUX - Testing-Strategie

> Jede Phase wird abschliessend getestet bevor sie gemergt wird.

---

## Grundsaetze

- **Unit-Tests fuer jede Phase.** Neue/geaenderte Funktionen werden getestet.
- **Edge-Cases an Eingaben und Antworten.** Leere Strings, sehr lange Texte,
  Unicode, Sonderzeichen, abgebrochene Streams, fehlende Daten, Grenzwerte
  und ueberschreiten dieser, Fallbacks.
- **LLM-API wird gemockt.** Kein Test darf echte API-Calls machen.
- **Filesystem mit tempdir.** Tests die Dateien brauchen arbeiten auf temporaeren Verzeichnissen.
- **Deterministische Tests.** Jeder Test muss ohne Netzwerk, ohne API-Key, ohne Seiteneffekte laufen.

---

## Tooling

### Rust-Bordmittel

| Was | Wie |
|-----|-----|
| Unit-Tests | `#[test]` bzw. `#[tokio::test]` fuer async |
| Test-Module | `#[cfg(test)] mod tests` in jeder Datei (Zugriff auf private Funktionen) |
| Assertions | `assert!`, `assert_eq!`, `assert_ne!`, `assert!(result.is_err())` |
| Ausfuehrung | `cargo test` (alle), `cargo test test_name` (einzeln), `cargo test --lib` (nur Unit-Tests) |
| Integration-Tests | `tests/` Verzeichnis im Crate (spaeter, wenn Bus-Kommunikation getestet wird) |
| Panic-Tests | `#[should_panic]` fuer erwartete Panics |
| Ignored Tests | `#[ignore]` fuer langsame/optionale Tests, ausfuehren mit `cargo test -- --ignored` |

### Externe Crates

| Crate | Zweck |
|-------|-------|
| `tempfile` | Temporaere Verzeichnisse fuer Filesystem-Tests (TempDir, automatisches Aufraumen) |
| `async-stream` | Stream-Erzeugung fuer MockModel (spaeter, wenn Agent-Logik gemockt wird) |

### rig-core Testinfrastruktur

rig hat kein exportiertes Mock-Framework, aber bietet Bausteine die wir nutzen:

- **`CompletionModel` Trait** - direkt implementierbar fuer ein eigenes MockModel.
  Zwei Methoden: `completion()` (non-streaming) und `stream()` (streaming).
- **`RawStreamingChoice`** - Enum fuer Stream-Chunks: `Message(String)`,
  `ToolCall(...)`, `FinalResponse(...)`. Damit laesst sich ein kompletter
  Agent-Ablauf simulieren.
- **`MultiTurnMockModel`** - rig's eigenes Test-Pattern (nicht exportiert, aber
  als Vorlage nutzbar). Simuliert Multi-Turn mit Tool-Calls ueber `async_stream::stream!`.
- **`Nothing` Client-Typ** - Zero-sized Stub fuer Tests ohne echten API-Client.
- **`Usage`** - Token-Zaehler, kontrollierbar fuer Kompaktifizierungs-Tests.

### Tool-Call Tests (spaeter)

Agent + Tool-Use laesst sich komplett testen ohne echte API:
MockModel yielded `ToolCall` → Agent ruft echtes Tool → MockModel bekommt
das Result im naechsten Turn und yielded die finale Antwort.
Damit testbar: Tool-Parameter, Tool-Ergebnisse, Multi-Turn-Ablauf.

Voraussetzung: Core muss generisch ueber das Model sein (aktuell hardcoded
per Provider-Match). Refactoring wenn Tool-Tests gebraucht werden.

---

## Wie mocken wir das LLM?

Eigenes `MockModel` das `CompletionModel` implementiert:

- Deterministische Antworten (konfigurierbar pro Test)
- Fehler simulieren (API-Error, leere Antwort, abgebrochener Stream)
- Usage-Daten kontrollierbar (fuer Kompaktifizierungs-Tests)
- Tool-Calls simulieren (fuer spaetere Agent-Integration-Tests)

Kein externes Mocking-Crate noetig - rig's Trait-System reicht.

---

## Test-Kategorien

### 1. Reine Logik

Funktionen ohne externe Abhaengigkeiten. Nur Parameter rein, Ergebnis raus.
Brauchen weder Mock noch Filesystem.

### 2. Filesystem-Logik

Funktionen die Dateien lesen/schreiben (Preamble, History, Memory).
Getestet mit `tempfile::TempDir` - jeder Test bekommt sein eigenes Verzeichnis.

### 3. Agent-Logik

Funktionen die den LLM-Client nutzen. Getestet mit `MockModel` + tempdir.
Hier werden auch die Edge-Cases an Ein-/Ausgaben geprueft.

---

## Konventionen

- Tests leben als `#[cfg(test)] mod tests` in der jeweiligen Datei (Rust-Standard)
- Async Tests mit `#[tokio::test]`
- Hilfsfunktionen (test_home, test_bus, etc.) koennen in ein gemeinsames Modul wenn noetig
- `cargo test` muss ohne Konfiguration durchlaufen

---

*Letzte Aktualisierung: 2026-03-01*
