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
use rig::message::Message;
use rig::providers::{anthropic, mistral, ollama};
use rig::streaming::{StreamedAssistantContent, StreamingChat};

use crate::bus::Bus;
use crate::config::AgentConfig;
use crate::events::Event;
use crate::memory::MemoryTool;

/// Macro: Streamt die Agent-Antwort und sammelt den Text.
/// Wird pro Provider-Arm genutzt, weil jeder einen eigenen Typ erzeugt.
macro_rules! stream_agent {
    ($agent:expr, $input:expr, $history:expr, $bus:expr) => {{
        let mut stream = $agent.stream_chat($input, $history).await;
        let mut response_text = String::new();

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
                Ok(_) => {
                    // ToolCall, Final, etc.
                }
                Err(e) => {
                    eprintln!("\nFehler: {}", e);
                    break;
                }
            }
        }

        response_text
    }};
}

/// Core haelt alles was der Agent braucht.
pub struct Core {
    bus: Arc<Bus>,
    home: PathBuf,
    history: Vec<Message>,
    config: AgentConfig,
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
    pub fn new(bus: Arc<Bus>, home: PathBuf, config: AgentConfig) -> Self {
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
        let response_text = match self.config.provider.as_str() {
            "anthropic" => {
                let client = anthropic::Client::from_env();
                let agent = client
                    .agent(&self.config.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.temperature)
                    .tool(memory_tool)
                    .build();
                stream_agent!(agent, input, self.history.clone(), self.bus)
            }
            "mistral" => {
                let client = mistral::Client::from_env();
                let agent = client
                    .agent(&self.config.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.temperature)
                    .tool(memory_tool)
                    .build();
                stream_agent!(agent, input, self.history.clone(), self.bus)
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
                stream_agent!(agent, input, self.history.clone(), self.bus)
            }
            other => {
                anyhow::bail!("Unbekannter Provider: '{}'", other);
            }
        };

        self.bus.publish(Event::ResponseComplete {
            full_text: response_text.clone(),
        });

        // History aktualisieren und persistieren (nur wenn Antwort nicht leer)
        if !response_text.is_empty() {
            self.history.push(Message::user(input));
            self.history.push(Message::assistant(&response_text));
            save_history(&self.home, &self.history);
        }

        Ok(())
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
    home.join(format!("memory/conversation-{}.json", today))
}

/// Laedt die gespeicherte Konversations-History fuer heute.
fn load_history(home: &PathBuf) -> Vec<Message> {
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
