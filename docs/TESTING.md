# AIUX - Testing-Strategie

> Jede Phase wird abschliessend getestet bevor sie gemergt wird.

---

## Grundsaetze

- **Unit-Tests fuer jede Phase.** Neue/geaenderte Funktionen werden getestet.
- **Edge-Cases an Eingaben und Antworten.** Leere Strings, sehr lange Texte,
  Unicode, Sonderzeichen, abgebrochene Streams, fehlende Daten.
- **LLM-API wird gemockt.** Kein Test darf echte API-Calls machen.
- **Filesystem mit tempdir.** Tests die Dateien brauchen arbeiten auf temporaeren Verzeichnissen.
- **Deterministische Tests.** Jeder Test muss ohne Netzwerk, ohne API-Key, ohne Seiteneffekte laufen.

---

## Wie mocken wir das LLM?

rig-core hat kein exportiertes Mock-Framework, aber der `CompletionModel` Trait
laesst sich direkt implementieren. Wir bauen ein eigenes `MockModel` das:

- Deterministische Antworten liefert (konfigurierbar pro Test)
- Fehler simulieren kann (API-Error, leere Antwort, abgebrochener Stream)
- Usage-Daten kontrollierbar zurueckgibt (fuer Kompaktifizierungs-Tests)

rig nutzt intern dasselbe Pattern (`MultiTurnMockModel` in den eigenen Tests).
Kein externes Mocking-Crate noetig.

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
