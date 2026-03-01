// Core: Das Gehirn des Agents.
//
// Kapselt die Preamble, History und Tools.
// Hoert auf UserInput Events und antwortet mit ResponseToken/ResponseComplete.
// Der LLM-Client wird per Config gesteuert (Provider, Modell, Temperature).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::{Chat, Usage};
use rig::message::Message;
use rig::providers::{anthropic, mistral, ollama};
use rig::streaming::{StreamedAssistantContent, StreamingChat};

use crate::bus::Bus;
use crate::config::Config;
use crate::events::Event;
use crate::memory::MemoryTool;

/// Macro: Streamt die Agent-Antwort und sammelt den Text.
/// Wird pro Provider-Arm genutzt, weil jeder einen eigenen Typ erzeugt.
/// Gibt (String, Option<Usage>) zurueck.
macro_rules! stream_agent {
    ($agent:expr, $input:expr, $history:expr, $bus:expr) => {{
        let mut stream = $agent.stream_chat($input, $history).await;
        let mut response_text = String::new();
        let mut usage: Option<Usage> = None;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Text(text),
                )) => {
                    $bus.publish(Event::ResponseToken {
                        text: text.text.clone(),
                    });
                    response_text.push_str(&text.text);
                }
                Ok(MultiTurnStreamItem::FinalResponse(final_resp)) => {
                    usage = Some(final_resp.usage());
                }
                Ok(_) => {
                    // ToolCall, etc.
                }
                Err(e) => {
                    $bus.publish(Event::SystemMessage {
                        text: format!("Fehler: {}", e),
                    });
                    break;
                }
            }
        }

        (response_text, usage)
    }};
}

/// Core haelt alles was der Agent braucht.
pub struct Core {
    bus: Arc<Bus>,
    home: PathBuf,
    history: Vec<Message>,
    config: Config,
    preamble: String,
    preamble_dirty: Arc<AtomicBool>,
}

/// Boot-Info fuer die Anzeige beim Start.
pub struct BootInfo {
    pub has_soul: bool,
    pub has_user: bool,
    pub context_count: usize,
    pub history_count: usize,
}

impl Core {
    /// Neuen Core erstellen. Laedt Preamble und History.
    pub fn new(bus: Arc<Bus>, home: PathBuf, config: Config) -> Self {
        dotenvy::dotenv().ok();
        let preamble = load_preamble(&home);
        let history = load_history(&home);

        Self {
            bus,
            home,
            history,
            config,
            preamble,
            preamble_dirty: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Info ueber den Boot-Zustand (fuer Anzeige).
    pub fn boot_info(&self) -> BootInfo {
        let context_count = count_context_files(&self.home);
        BootInfo {
            has_soul: self.home.join("memory/soul.md").exists(),
            has_user: self.home.join("memory/user.md").exists(),
            context_count,
            history_count: self.history.len(),
        }
    }

    /// Hauptschleife: auf Events hoeren und reagieren.
    pub async fn run(mut self) -> Result<(), anyhow::Error> {
        let mut receiver = self.bus.subscribe();

        loop {
            match receiver.recv().await {
                Ok(Event::UserInput { text }) => {
                    self.handle_input(&text).await?;
                }
                Ok(Event::ClearHistory) => {
                    self.history.clear();
                    fs::remove_file(conversation_path(&self.home)).ok();
                }
                Ok(Event::Shutdown) => break,
                Ok(_) => {} // Eigene Events ignorieren
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("Bus: {} Events verpasst", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }

        Ok(())
    }

    /// Eingabe verarbeiten: Agent fragen, Tokens streamen, History updaten.
    async fn handle_input(&mut self, input: &str) -> Result<(), anyhow::Error> {
        // Preamble nur neu laden wenn sich context/ geaendert hat (dirty flag vom MemoryTool)
        if self.preamble_dirty.swap(false, Ordering::Relaxed) {
            self.preamble = load_preamble(&self.home);
        }

        let memory_tool = MemoryTool::new(&self.home, Arc::clone(&self.preamble_dirty));

        // Stream-Verarbeitung passiert im match-Block,
        // weil jeder Provider einen eigenen Rust-Typ erzeugt.
        let (response_text, usage) = match self.config.provider.as_str() {
            "anthropic" => {
                let client = anthropic::Client::from_env();
                let agent = client
                    .agent(&self.config.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.temperature)
                    .tool(memory_tool)
                    .build();
                stream_agent!(agent, input, self.history_for_agent(), self.bus)
            }
            "mistral" => {
                let client = mistral::Client::from_env();
                let agent = client
                    .agent(&self.config.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.temperature)
                    .tool(memory_tool)
                    .build();
                stream_agent!(agent, input, self.history_for_agent(), self.bus)
            }
            "ollama" => {
                let client: ollama::Client = ollama::Client::new(rig::client::Nothing).map_err(|e| {
                    anyhow::anyhow!("Ollama-Client konnte nicht erstellt werden: {}", e)
                })?;
                let agent = client
                    .agent(&self.config.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.temperature)
                    .tool(memory_tool)
                    .build();
                stream_agent!(agent, input, self.history_for_agent(), self.bus)
            }
            other => {
                anyhow::bail!("Unbekannter Provider: '{}'", other);
            }
        };

        self.bus.publish(Event::ResponseComplete {
            full_text: response_text.clone(),
        });

        if let Some(ref u) = usage {
            self.bus.publish(Event::SystemMessage {
                text: format!(
                    "usage: input={} output={} cached={}",
                    u.input_tokens, u.output_tokens, u.cached_input_tokens
                ),
            });
        }

        // History aktualisieren und persistieren (nur bei vollstaendiger Antwort)
        if usage.is_some() && !response_text.is_empty() {
            self.history.push(Message::user(input));
            self.history.push(Message::assistant(&response_text));
            save_history(&self.home, &self.history);
        }

        // Kompaktifizierung pruefen
        if let Some(ref u) = usage {
            let window = context_window_size(&self.config.model, self.config.context_window);
            let threshold = self.config.compact_threshold.unwrap_or(80);
            if threshold > 0 && should_compact(u.input_tokens, window, threshold) {
                self.bus.publish(Event::Compacting);
                match self.compact_history().await {
                    Ok(summary) => {
                        self.history.push(Message::user("[KOMPAKTIFIZIERUNG]"));
                        self.history.push(Message::assistant(&summary));
                        save_history(&self.home, &self.history);
                        self.bus.publish(Event::Compacted);
                    }
                    Err(e) => {
                        self.bus.publish(Event::SystemMessage {
                            text: format!("Kompaktifizierung fehlgeschlagen: {}", e),
                        });
                        self.bus.publish(Event::Compacted); // Prompt wiederherstellen
                    }
                }
            }
        }

        Ok(())
    }

    /// Gibt den relevanten Teil der History zurueck (ab dem letzten Kompaktifizierungs-Marker).
    fn history_for_agent(&self) -> Vec<Message> {
        // Letzten Kompaktifizierungs-Marker suchen
        let last_compact = self.history.iter().rposition(|msg| {
            matches!(msg, Message::User { content } if content.iter().any(|part| {
                matches!(part, rig::message::UserContent::Text(rig::message::Text { text, .. }) if text == "[KOMPAKTIFIZIERUNG]")
            }))
        });

        match last_compact {
            Some(idx) => self.history[idx..].to_vec(),
            None => self.history.clone(),
        }
    }

    /// Non-streaming, tool-freier LLM-Call fuer interne Aufgaben (z.B. Kompaktifizierung).
    async fn simple_chat(&self, preamble: &str, prompt: &str) -> Result<String, anyhow::Error> {
        match self.config.provider.as_str() {
            "anthropic" => {
                let client = anthropic::Client::from_env();
                let agent = client
                    .agent(&self.config.model)
                    .preamble(preamble)
                    .temperature(0.3)
                    .build();
                Ok(agent.chat(prompt, vec![]).await?)
            }
            "mistral" => {
                let client = mistral::Client::from_env();
                let agent = client
                    .agent(&self.config.model)
                    .preamble(preamble)
                    .temperature(0.3)
                    .build();
                Ok(agent.chat(prompt, vec![]).await?)
            }
            "ollama" => {
                let client: ollama::Client = ollama::Client::new(rig::client::Nothing)
                    .map_err(|e| anyhow::anyhow!("Ollama-Client: {}", e))?;
                let agent = client
                    .agent(&self.config.model)
                    .preamble(preamble)
                    .temperature(0.3)
                    .build();
                Ok(agent.chat(prompt, vec![]).await?)
            }
            other => anyhow::bail!("Unbekannter Provider: '{}'", other),
        }
    }

    /// Baut die History als lesbaren Text zusammen.
    fn history_as_text(&self) -> String {
        let mut text = String::from("Hier ist die bisherige Konversation:\n\n");
        for msg in &self.history {
            match msg {
                Message::User { content } => {
                    text.push_str("User: ");
                    for part in content.iter() {
                        if let rig::message::UserContent::Text(t) = part {
                            text.push_str(&t.text);
                        }
                    }
                    text.push('\n');
                }
                Message::Assistant { content, .. } => {
                    text.push_str("Assistant: ");
                    for part in content.iter() {
                        if let rig::message::AssistantContent::Text(t) = part {
                            text.push_str(&t.text);
                        }
                    }
                    text.push('\n');
                }
            }
        }
        text.push_str("\nFasse diese Konversation zusammen.");
        text
    }

    /// Fuehrt einen Kompaktifizierungs-Call durch.
    async fn compact_history(&self) -> Result<String, anyhow::Error> {
        let preamble = fs::read_to_string(self.home.join(".system/compact-preamble.md"))
            .unwrap_or_else(|_| "Fasse die Konversation zusammen.".to_string());
        let prompt = self.history_as_text();
        self.simple_chat(&preamble, &prompt).await
    }
}

/// Prueft ob die Input-Token-Nutzung den Schwellwert erreicht hat.
fn should_compact(input_tokens: u64, context_window: u64, threshold_percent: u64) -> bool {
    if context_window == 0 {
        return false;
    }
    input_tokens * 100 / context_window >= threshold_percent
}

/// Schaetzt die Context-Window-Groesse anhand des Modellnamens.
/// Config-Override hat Vorrang (z.B. fuer Ollama-Modelle).
fn context_window_size(model: &str, config_override: Option<u64>) -> u64 {
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

// --- Hilfsfunktionen (aus dem alten main.rs) ---

/// Laedt eine Datei oder gibt einen leeren String zurueck.
fn read_file(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Laedt alle .md Dateien aus einem Verzeichnis, alphabetisch sortiert.
fn load_context_files(dir: &PathBuf) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "md"))
            .collect();

        paths.sort();

        for path in paths {
            let name = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let content = read_file(&path);
            if !content.is_empty() {
                files.push((name, content));
            }
        }
    }

    files
}

/// Zaehlt die Context-Dateien (fuer Boot-Info).
fn count_context_files(home: &PathBuf) -> usize {
    load_context_files(&home.join("memory/context")).len()
}

/// Baut den System-Prompt zusammen (Boot-Sequence):
/// 1. soul.md - Wer bin ich?
/// 2. user.md - Mit wem rede ich?
/// 3. context/*.md - Was weiss ich noch?
fn load_preamble(home: &PathBuf) -> String {
    let mut parts: Vec<String> = Vec::new();

    let soul = read_file(&home.join("memory/soul.md"));
    if !soul.is_empty() {
        parts.push(soul);
    }

    let user = read_file(&home.join("memory/user.md"));
    if !user.is_empty() {
        parts.push(user);
    }

    let context_files = load_context_files(&home.join("memory/context"));
    for (name, content) in &context_files {
        parts.push(format!("# Kontext: {}\n\n{}", name, content));
    }

    parts.join("\n\n---\n\n")
}

/// Findet das home/ Verzeichnis.
pub fn find_home() -> PathBuf {
    let local = PathBuf::from("home");
    if local.join("memory/soul.md").exists() {
        return local;
    }

    let deployed = PathBuf::from("/home/claude");
    if deployed.join("memory/soul.md").exists() {
        return deployed;
    }

    local
}

/// Gibt den Dateinamen fuer die heutige Konversation zurueck.
fn conversation_path(home: &PathBuf) -> PathBuf {
    let today = chrono::Local::now().format("%Y-%m-%d");
    home.join(format!("memory/conversations/conversation-{}.json", today))
}

/// Laedt die gespeicherte Konversations-History fuer heute.
fn load_history(home: &PathBuf) -> Vec<Message> {
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
fn save_history(home: &PathBuf, history: &[Message]) {
    let path = conversation_path(home);
    if let Ok(data) = serde_json::to_string_pretty(history) {
        fs::write(&path, data).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- Helper ---

    /// Erstellt ein temporaeres home/ mit der noetigsten Struktur.
    fn test_home() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory/context")).unwrap();
        fs::create_dir_all(home.join("memory/conversations")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        (tmp, home)
    }

    fn test_config() -> Config {
        Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5-20250929".to_string(),
            temperature: 0.7,
            api_key_env: None,
            context_window: None,
            compact_threshold: None,
        }
    }

    fn test_core(home: PathBuf) -> Core {
        let bus = Arc::new(Bus::new(16));
        Core::new(bus, home, test_config())
    }

    // ==========================================================
    // should_compact()
    // ==========================================================

    #[test]
    fn compact_schwelle_genau_erreicht() {
        // 80% von 200_000 = 160_000
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
        // Division durch 0 abgefangen
        assert!(!should_compact(100, 0, 80));
    }

    #[test]
    fn compact_null_prozent_schwelle() {
        // 0% = immer kompaktifizieren (wenn tokens > 0)
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
        // Override hat Vorrang, egal welches Modell
        assert_eq!(context_window_size("claude-sonnet-4-5-20250929", Some(50_000)), 50_000);
        assert_eq!(context_window_size("unbekannt", Some(8_000)), 8_000);
    }

    // ==========================================================
    // load_preamble() / load_context_files()
    // ==========================================================

    #[test]
    fn preamble_mit_soul_und_user() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "Ich bin AIUX.").unwrap();
        fs::write(home.join("memory/user.md"), "Bruce ist cool.").unwrap();

        let preamble = load_preamble(&home);
        assert!(preamble.contains("Ich bin AIUX."));
        assert!(preamble.contains("Bruce ist cool."));
        assert!(preamble.contains("---")); // Trenner
    }

    #[test]
    fn preamble_nur_soul() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "Ich bin AIUX.").unwrap();

        let preamble = load_preamble(&home);
        assert_eq!(preamble, "Ich bin AIUX.");
    }

    #[test]
    fn preamble_ohne_dateien() {
        let (_tmp, home) = test_home();
        let preamble = load_preamble(&home);
        assert!(preamble.is_empty());
    }

    #[test]
    fn preamble_mit_context_dateien() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "Soul.").unwrap();
        fs::write(home.join("memory/context/a.md"), "AAA").unwrap();
        fs::write(home.join("memory/context/b.md"), "BBB").unwrap();

        let preamble = load_preamble(&home);
        assert!(preamble.contains("# Kontext: a"));
        assert!(preamble.contains("AAA"));
        assert!(preamble.contains("# Kontext: b"));
        assert!(preamble.contains("BBB"));
    }

    #[test]
    fn preamble_leere_dateien_werden_ignoriert() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "").unwrap();
        fs::write(home.join("memory/context/leer.md"), "").unwrap();

        let preamble = load_preamble(&home);
        assert!(preamble.is_empty());
    }

    #[test]
    fn context_files_nur_md() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/context/notiz.md"), "Inhalt").unwrap();
        fs::write(home.join("memory/context/bild.png"), "binary").unwrap();
        fs::write(home.join("memory/context/readme.txt"), "text").unwrap();

        let files = load_context_files(&home.join("memory/context"));
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "notiz");
    }

    #[test]
    fn context_files_sortiert() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/context/c.md"), "C").unwrap();
        fs::write(home.join("memory/context/a.md"), "A").unwrap();
        fs::write(home.join("memory/context/b.md"), "B").unwrap();

        let files = load_context_files(&home.join("memory/context"));
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn context_files_leeres_verzeichnis() {
        let (_tmp, home) = test_home();
        let files = load_context_files(&home.join("memory/context"));
        assert!(files.is_empty());
    }

    #[test]
    fn context_files_verzeichnis_existiert_nicht() {
        let (_tmp, home) = test_home();
        let files = load_context_files(&home.join("memory/gibts_nicht"));
        assert!(files.is_empty());
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
        assert!(loaded.is_empty()); // unwrap_or_default greift
    }

    #[test]
    fn history_leere_liste() {
        let (_tmp, home) = test_home();
        save_history(&home, &[]);

        let loaded = load_history(&home);
        assert!(loaded.is_empty());
    }

    // ==========================================================
    // history_as_text()
    // ==========================================================

    #[test]
    fn history_text_normal() {
        let (_tmp, home) = test_home();
        let mut core = test_core(home);
        core.history = vec![
            Message::user("Was ist Rust?"),
            Message::assistant("Eine Programmiersprache."),
        ];

        let text = core.history_as_text();
        assert!(text.contains("User: Was ist Rust?"));
        assert!(text.contains("Assistant: Eine Programmiersprache."));
        assert!(text.contains("Fasse diese Konversation zusammen."));
    }

    #[test]
    fn history_text_leer() {
        let (_tmp, home) = test_home();
        let core = test_core(home);

        let text = core.history_as_text();
        assert!(text.contains("Hier ist die bisherige Konversation:"));
        assert!(text.contains("Fasse diese Konversation zusammen."));
        assert!(!text.contains("User:"));
    }

    // ==========================================================
    // history_for_agent()
    // ==========================================================

    #[test]
    fn history_for_agent_ohne_marker() {
        let (_tmp, home) = test_home();
        let mut core = test_core(home);
        core.history = vec![
            Message::user("Eins"),
            Message::assistant("Zwei"),
        ];

        let result = core.history_for_agent();
        assert_eq!(result.len(), 2); // Alles
    }

    #[test]
    fn history_for_agent_mit_marker() {
        let (_tmp, home) = test_home();
        let mut core = test_core(home);
        core.history = vec![
            Message::user("Alt"),
            Message::assistant("Alte Antwort"),
            Message::user("[KOMPAKTIFIZIERUNG]"),
            Message::assistant("Zusammenfassung"),
            Message::user("Neu"),
            Message::assistant("Neue Antwort"),
        ];

        let result = core.history_for_agent();
        assert_eq!(result.len(), 4); // Ab Marker: Marker + Summary + Neu + Neue Antwort
    }

    #[test]
    fn history_for_agent_mehrere_marker() {
        let (_tmp, home) = test_home();
        let mut core = test_core(home);
        core.history = vec![
            Message::user("[KOMPAKTIFIZIERUNG]"),
            Message::assistant("Erste Zusammenfassung"),
            Message::user("Dazwischen"),
            Message::assistant("Antwort"),
            Message::user("[KOMPAKTIFIZIERUNG]"),
            Message::assistant("Zweite Zusammenfassung"),
            Message::user("Aktuell"),
            Message::assistant("Aktuelle Antwort"),
        ];

        let result = core.history_for_agent();
        assert_eq!(result.len(), 4); // Ab letztem Marker
    }

    #[test]
    fn history_for_agent_leere_history() {
        let (_tmp, home) = test_home();
        let core = test_core(home);

        let result = core.history_for_agent();
        assert!(result.is_empty());
    }

    // ==========================================================
    // boot_info()
    // ==========================================================

    #[test]
    fn boot_info_alles_vorhanden() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "Soul").unwrap();
        fs::write(home.join("memory/user.md"), "User").unwrap();
        fs::write(home.join("memory/context/a.md"), "A").unwrap();

        let core = test_core(home);
        let info = core.boot_info();
        assert!(info.has_soul);
        assert!(info.has_user);
        assert_eq!(info.context_count, 1);
    }

    #[test]
    fn boot_info_nichts_vorhanden() {
        let (_tmp, home) = test_home();
        let core = test_core(home);
        let info = core.boot_info();
        assert!(!info.has_soul);
        assert!(!info.has_user);
        assert_eq!(info.context_count, 0);
        assert_eq!(info.history_count, 0);
    }

    // ==========================================================
    // conversation_path() - neuer Pfad in conversations/
    // ==========================================================

    #[test]
    fn conversation_path_in_conversations_subdir() {
        let (_tmp, home) = test_home();
        let path = conversation_path(&home);
        // Pfad muss memory/conversations/conversation-YYYY-MM-DD.json sein
        assert!(path.to_string_lossy().contains("memory/conversations/conversation-"));
        assert!(path.to_string_lossy().ends_with(".json"));
    }

    #[test]
    fn conversations_dir_wird_automatisch_erstellt() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        // Nur memory/ erstellen, NICHT conversations/
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();

        // load_history erstellt conversations/ automatisch
        let _loaded = load_history(&home);
        assert!(home.join("memory/conversations").is_dir());
    }
}
