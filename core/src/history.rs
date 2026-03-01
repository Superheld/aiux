// History: Conversation-Persistenz und Kompaktifizierungs-Logik.
//
// Tages-History als JSON in memory/conversations/.
// Kompaktifizierung prueft ob das Token-Budget erreicht ist.

use std::fs;
use std::path::{Path, PathBuf};

use rig::message::Message;

/// Gibt den Dateinamen fuer die heutige Konversation zurueck.
pub fn conversation_path(home: &Path) -> PathBuf {
    let today = chrono::Local::now().format("%Y-%m-%d");
    home.join(format!("memory/conversations/conversation-{}.json", today))
}

/// Laedt die gespeicherte Konversations-History fuer heute.
pub fn load_history(home: &Path) -> Vec<Message> {
    // Verzeichnis erstellen falls nicht vorhanden
    let conv_dir = home.join("memory/conversations");
    fs::create_dir_all(&conv_dir).ok();

    let path = conversation_path(home);
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => vec![],
    }
}

/// Speichert die aktuelle History als JSON.
pub fn save_history(home: &Path, history: &[Message]) {
    let path = conversation_path(home);
    if let Ok(data) = serde_json::to_string_pretty(history) {
        fs::write(&path, data).ok();
    }
}

/// Prueft ob die Input-Token-Nutzung den Schwellwert erreicht hat.
pub fn should_compact(input_tokens: u64, context_window: u64, threshold_percent: u64) -> bool {
    if context_window == 0 {
        return false;
    }
    input_tokens * 100 / context_window >= threshold_percent
}

/// Schaetzt die Context-Window-Groesse anhand des Modellnamens.
/// Config-Override hat Vorrang (z.B. fuer Ollama-Modelle).
pub fn context_window_size(model: &str, config_override: Option<u64>) -> u64 {
    if let Some(v) = config_override {
        return v;
    }
    if model.starts_with("claude") {
        200_000
    } else if model.starts_with("mistral-large") {
        128_000
    } else if model.starts_with("mistral-small") {
        32_000
    } else {
        128_000 // Konservativer Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_home() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory/conversations")).unwrap();
        (tmp, home)
    }

    // ==========================================================
    // should_compact()
    // ==========================================================

    #[test]
    fn compact_schwelle_genau_erreicht() {
        assert!(should_compact(160_000, 200_000, 80));
    }

    #[test]
    fn compact_knapp_unter_schwelle() {
        assert!(!should_compact(159_999, 200_000, 80));
    }

    #[test]
    fn compact_ueber_schwelle() {
        assert!(should_compact(180_000, 200_000, 80));
    }

    #[test]
    fn compact_context_window_null() {
        assert!(!should_compact(100, 0, 80));
    }

    #[test]
    fn compact_null_prozent_schwelle() {
        assert!(should_compact(1, 200_000, 0));
    }

    #[test]
    fn compact_hundert_prozent() {
        assert!(should_compact(200_000, 200_000, 100));
        assert!(!should_compact(199_999, 200_000, 100));
    }

    #[test]
    fn compact_null_tokens() {
        assert!(!should_compact(0, 200_000, 80));
    }

    // ==========================================================
    // context_window_size()
    // ==========================================================

    #[test]
    fn window_claude_modelle() {
        assert_eq!(context_window_size("claude-sonnet-4-5-20250929", None), 200_000);
        assert_eq!(context_window_size("claude-3-haiku", None), 200_000);
    }

    #[test]
    fn window_mistral_modelle() {
        assert_eq!(context_window_size("mistral-large-latest", None), 128_000);
        assert_eq!(context_window_size("mistral-small-2402", None), 32_000);
    }

    #[test]
    fn window_unbekanntes_modell() {
        assert_eq!(context_window_size("gpt-4o", None), 128_000);
        assert_eq!(context_window_size("", None), 128_000);
    }

    #[test]
    fn window_config_override() {
        assert_eq!(context_window_size("claude-sonnet-4-5-20250929", Some(50_000)), 50_000);
        assert_eq!(context_window_size("unbekannt", Some(8_000)), 8_000);
    }

    // ==========================================================
    // save_history() / load_history()
    // ==========================================================

    #[test]
    fn history_save_and_load_roundtrip() {
        let (_tmp, home) = test_home();
        let history = vec![
            Message::user("Hallo"),
            Message::assistant("Hi!"),
        ];
        save_history(&home, &history);

        let loaded = load_history(&home);
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn history_load_fehlende_datei() {
        let (_tmp, home) = test_home();
        let loaded = load_history(&home);
        assert!(loaded.is_empty());
    }

    #[test]
    fn history_load_kaputtes_json() {
        let (_tmp, home) = test_home();
        let path = conversation_path(&home);
        fs::write(&path, "das ist kein json {{{").unwrap();

        let loaded = load_history(&home);
        assert!(loaded.is_empty());
    }

    #[test]
    fn history_leere_liste() {
        let (_tmp, home) = test_home();
        save_history(&home, &[]);

        let loaded = load_history(&home);
        assert!(loaded.is_empty());
    }

    // ==========================================================
    // conversation_path()
    // ==========================================================

    #[test]
    fn conversation_path_in_conversations_subdir() {
        let (_tmp, home) = test_home();
        let path = conversation_path(&home);
        assert!(path.to_string_lossy().contains("memory/conversations/conversation-"));
        assert!(path.to_string_lossy().ends_with(".json"));
    }

    #[test]
    fn conversations_dir_wird_automatisch_erstellt() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();

        let _loaded = load_history(&home);
        assert!(home.join("memory/conversations").is_dir());
    }
}
