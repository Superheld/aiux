// Neocortex: Das Grosshirn.
//
// Hoert auf UserInput Events, fragt das LLM, streamt die Antwort.
// Einziger Agent der am Bus haengt und die History verwaltet.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Usage;
use rig::message::Message;
use rig::providers::{anthropic, mistral, ollama};
use rig::streaming::{StreamedAssistantContent, StreamingChat};

use crate::bus::Bus;
use crate::bus::events::Event;
use crate::config::Config;
use crate::history;
use super::hippocampus;
use crate::brainstem::SharedScheduler;
use crate::tools::memory::MemoryTool;
use crate::tools::scheduler::SchedulerTool;
use crate::tools::shell::ShellTool;
use crate::tools::soul::SoulTool;
use crate::tools::user::UserTool;

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
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::ToolCall { tool_call, .. },
                )) => {
                    $bus.publish(Event::ToolCall {
                        name: tool_call.function.name.clone(),
                    });
                }
                Ok(_) => {}
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

/// Baut den System-Prompt aus Sektionen zusammen (wie OpenClaw Bootstrap).
/// Reihenfolge: Identity → Koerper → User → Notizen
fn load_preamble(home: &std::path::Path, config: &Config) -> String {
    let mut sections = Vec::new();

    // 1. Identitaet (soul.md)
    let soul = fs::read_to_string(home.join("memory/soul.md")).unwrap_or_default();
    if !soul.is_empty() {
        sections.push(soul);
    }

    // 2. Koerper (dynamisch aus Config generiert)
    sections.push(build_body_section(config));

    // 3. User-Profil (user.md)
    let user = fs::read_to_string(home.join("memory/user.md")).unwrap_or_default();
    if !user.is_empty() {
        sections.push(user);
    }

    // 4. Notizen (notes.md)
    let notes = fs::read_to_string(home.join("memory/notes.md")).unwrap_or_default();
    if !notes.is_empty() {
        sections.push(notes);
    }

    sections.join("\n\n---\n\n")
}

/// Generiert die Koerper-Sektion: Was bin ich, was habe ich?
/// Folgt den 6 Schichten aus der PRD:
/// Sein (Soul) | Denken (Core) | Spueren (Nerves) | Erinnern (Memory) | Wissen (Skills) | Handeln (Tools)
fn build_body_section(config: &Config) -> String {
    let mut s = String::from("# Mein Koerper\n\n");

    // --- Denken (Core) ---
    s.push_str("## Denken — Core\n\n");
    s.push_str(&format!(
        "Mein Neocortex laeuft auf {} ({}).\n",
        config.neocortex.model, config.neocortex.provider
    ));
    if let Some(ref hc) = config.hippocampus {
        s.push_str(&format!(
            "Mein Hippocampus (unbewusstes Gedaechtnis) laeuft auf {} ({}).\n\
             Er destilliert Wissen wenn mein Kontext voll wird oder die Session endet.\n",
            hc.model, hc.provider
        ));
    }

    // --- Erinnern (Memory) ---
    s.push_str("\n## Erinnern — Memory\n\n");
    s.push_str(
        "- **soul.md** — Meine Identitaet. Wer ich bin, wie ich spreche.\n\
         - **user.md** — Mein Wissen ueber Bruce.\n\
         - **notes.md** — Mein Notizbuch. Hier schreibe ich auf was ich lerne.\n\
         - **conversations/** — Meine Gespraeche (pro Tag eine Datei, automatisch).\n"
    );

    // --- Wissen (Skills) ---
    s.push_str("\n## Wissen — Skills\n\n");
    s.push_str("Noch keine Skills geladen. Skills sind Expertise als Text — \
         Anleitungen, Domaenenwissen, Vorlagen. Kein Code, sondern Wissen.\n");

    // --- Handeln (Tools) ---
    s.push_str("\n## Handeln — Tools\n\n");
    s.push_str(
        "- **SoulTool** — soul.md lesen/schreiben\n\
         - **UserTool** — user.md lesen/schreiben\n\
         - **MemoryTool** — notes.md lesen/schreiben\n\
         - **SchedulerTool** — Reminder und Heartbeats planen\n"
    );
    if let Some(ref shell) = config.shell {
        if !shell.whitelist.is_empty() {
            s.push_str(&format!(
                "- **ShellTool** — Shell-Befehle ausfuehren (Whitelist: {})\n",
                shell.whitelist.join(", ")
            ));
        }
    }

    // --- Spueren (Nerves) ---
    s.push_str("\n## Spueren — Nerves\n\n");
    if let Some(ref mqtt) = config.mqtt {
        s.push_str(&format!(
            "MQTT-Bus aktiv ({}:{}). Nerves melden sich per MQTT.\n\
             Ich bekomme Nerve-Signale als Text — ich weiss nicht dass MQTT dahinter steckt.\n",
            mqtt.host, mqtt.port
        ));
    } else {
        s.push_str("Kein MQTT konfiguriert. Ich habe noch keine Sinne — nur Chat.\n");
    }

    s
}

/// Core haelt alles was der Neocortex-Agent braucht.
pub struct Core {
    bus: Arc<Bus>,
    home: PathBuf,
    history: Vec<Message>,
    config: Config,
    preamble: String,
    preamble_dirty: Arc<AtomicBool>,
    scheduler: SharedScheduler,
}

/// Boot-Info fuer die Anzeige beim Start.
pub struct BootInfo {
    pub provider: String,
    pub model: String,
    pub hippocampus_provider: Option<String>,
    pub hippocampus_model: Option<String>,
    pub has_soul: bool,
    pub has_user: bool,
    pub has_notes: bool,
    pub mqtt_active: bool,
    pub shell_active: bool,
    pub history_count: usize,
}

impl Core {
    /// Neuen Core erstellen. Laedt Preamble und History.
    pub fn new(bus: Arc<Bus>, home: PathBuf, config: Config, scheduler: SharedScheduler) -> Self {
        dotenvy::dotenv().ok();
        let preamble_text = load_preamble(&home, &config);
        let hist = history::load_history(&home);

        Self {
            bus,
            home,
            history: hist,
            config,
            preamble: preamble_text,
            preamble_dirty: Arc::new(AtomicBool::new(false)),
            scheduler,
        }
    }

    /// Info ueber den Boot-Zustand (fuer Anzeige).
    pub fn boot_info(&self) -> BootInfo {
        BootInfo {
            provider: self.config.neocortex.provider.clone(),
            model: self.config.neocortex.model.clone(),
            hippocampus_provider: self.config.hippocampus.as_ref().map(|h| h.provider.clone()),
            hippocampus_model: self.config.hippocampus.as_ref().map(|h| h.model.clone()),
            has_soul: self.home.join("memory/soul.md").exists(),
            has_user: self.home.join("memory/user.md").exists(),
            has_notes: self.home.join("memory/notes.md").exists(),
            mqtt_active: self.config.mqtt.is_some(),
            shell_active: self.config.shell.is_some(),
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
                Ok(Event::HeartbeatTick { label }) => {
                    let prompt = format!("[HEARTBEAT: {}]", label);
                    self.handle_input(&prompt).await?;
                }
                Ok(Event::ClearHistory) => {
                    // Memory-Flush vor dem Loeschen
                    if !self.history.is_empty() {
                        self.bus.publish(Event::Compacting);
                        if let Err(e) = self.memory_flush().await {
                            self.bus.publish(Event::SystemMessage {
                                text: format!("Memory-Flush fehlgeschlagen: {}", e),
                            });
                        }
                        self.bus.publish(Event::Compacted);
                    }
                    self.history.clear();
                    fs::remove_file(history::conversation_path(&self.home)).ok();
                }
                Ok(Event::Shutdown) => {
                    // Memory-Flush vor dem Beenden
                    if !self.history.is_empty() {
                        self.bus.publish(Event::Compacting);
                        if let Err(e) = self.memory_flush().await {
                            self.bus.publish(Event::SystemMessage {
                                text: format!("Memory-Flush fehlgeschlagen: {}", e),
                            });
                        }
                        self.bus.publish(Event::Compacted);
                    }
                    break;
                }
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
            self.preamble = load_preamble(&self.home, &self.config);
        }

        let soul_tool = SoulTool::new(&self.home, Arc::clone(&self.preamble_dirty));
        let user_tool = UserTool::new(&self.home, Arc::clone(&self.preamble_dirty));
        let memory_tool = MemoryTool::new(&self.home, Arc::clone(&self.preamble_dirty));
        let scheduler_tool = SchedulerTool::new(self.scheduler.clone());

        // ShellTool: immer registriert, aber mit leerer Whitelist wenn [shell] fehlt.
        // (rig-core Builder aendert den Typ pro .tool() — bedingt registrieren geht nicht)
        let shell_config = self.config.shell.clone().unwrap_or(crate::config::ShellConfig {
            whitelist: vec![],
            timeout: 30,
        });
        let shell_tool = ShellTool::new(shell_config);

        // Stream-Verarbeitung passiert im match-Block,
        // weil jeder Provider einen eigenen Rust-Typ erzeugt.
        let (response_text, usage) = match self.config.neocortex.provider.as_str() {
            "anthropic" => {
                let client = anthropic::Client::from_env();
                let agent = client
                    .agent(&self.config.neocortex.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.neocortex.temperature)
                    .default_max_turns(self.config.neocortex.max_turns)
                    .tool(soul_tool)
                    .tool(user_tool)
                    .tool(memory_tool)
                    .tool(scheduler_tool)
                    .tool(shell_tool)
                    .build();
                stream_agent!(agent, input, self.history_for_agent(), self.bus)
            }
            "mistral" => {
                let client = mistral::Client::from_env();
                let agent = client
                    .agent(&self.config.neocortex.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.neocortex.temperature)
                    .default_max_turns(self.config.neocortex.max_turns)
                    .tool(soul_tool)
                    .tool(user_tool)
                    .tool(memory_tool)
                    .tool(scheduler_tool)
                    .tool(shell_tool)
                    .build();
                stream_agent!(agent, input, self.history_for_agent(), self.bus)
            }
            "ollama" => {
                let client: ollama::Client =
                    ollama::Client::new(rig::client::Nothing).map_err(|e| {
                        anyhow::anyhow!("Ollama-Client konnte nicht erstellt werden: {}", e)
                    })?;
                let agent = client
                    .agent(&self.config.neocortex.model)
                    .preamble(&self.preamble)
                    .temperature(self.config.neocortex.temperature)
                    .default_max_turns(self.config.neocortex.max_turns)
                    .tool(soul_tool)
                    .tool(user_tool)
                    .tool(memory_tool)
                    .tool(scheduler_tool)
                    .tool(shell_tool)
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
            history::save_history(&self.home, &self.history);
        }

        // Kompaktifizierung pruefen
        if let Some(ref u) = usage {
            let window = history::context_window_size(&self.config.neocortex.model, self.config.neocortex.context_window);
            let threshold = self.config.neocortex.compact_threshold.unwrap_or(80);
            if threshold > 0 && history::should_compact(u.input_tokens, window, threshold) {
                self.bus.publish(Event::Compacting);
                match self.compact_history().await {
                    Ok(summary) => {
                        self.history.push(Message::user("[KOMPAKTIFIZIERUNG]"));
                        self.history.push(Message::assistant(&summary));
                        history::save_history(&self.home, &self.history);
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
        text.push_str("\nDestilliere das Wichtige und fasse den Rest zusammen.");
        text
    }

    /// Hippocampus-Call: Delegiert an den Hippocampus-Agent.
    async fn hippocampus_call(&self, prompt: &str) -> Result<String, anyhow::Error> {
        hippocampus::hippocampus_call(&self.home, &self.config, &self.preamble_dirty, prompt).await
    }

    /// Fuehrt einen Kompaktifizierungs-Call durch.
    /// Destilliert Wissen via Tools und reduziert die History auf die letzten 5 Messages.
    async fn compact_history(&mut self) -> Result<String, anyhow::Error> {
        let prompt = self.history_as_text();
        let summary = self.hippocampus_call(&prompt).await?;

        // History auf die letzten 5 Messages reduzieren
        let keep_count = 5.min(self.history.len());
        let kept = self.history.split_off(self.history.len() - keep_count);
        self.history = kept;

        Ok(summary)
    }

    /// Memory-Flush: Hippocampus-Call ohne History-Reduktion.
    /// Wird bei /clear und /quit aufgerufen um Wissen zu sichern.
    async fn memory_flush(&self) -> Result<(), anyhow::Error> {
        let prompt = self.history_as_text();
        self.hippocampus_call(&prompt).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- Helper ---

    fn test_home() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join("memory/conversations")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        (tmp, home)
    }

    fn test_config() -> Config {
        Config {
            neocortex: crate::config::AgentConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-5-20250929".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
                max_turns: 10,
            },
            hippocampus: None,
            mqtt: None,
            shell: None,
        }
    }

    fn test_core(home: PathBuf) -> Core {
        let bus = Arc::new(Bus::new(16));
        let scheduler = Arc::new(std::sync::Mutex::new(Vec::new()));
        Core::new(bus, home, test_config(), scheduler)
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
        assert!(text.contains("Destilliere das Wichtige"));
    }

    #[test]
    fn history_text_leer() {
        let (_tmp, home) = test_home();
        let core = test_core(home);

        let text = core.history_as_text();
        assert!(text.contains("Hier ist die bisherige Konversation:"));
        assert!(text.contains("Destilliere das Wichtige"));
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
        assert_eq!(result.len(), 2);
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
        assert_eq!(result.len(), 4);
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
        assert_eq!(result.len(), 4);
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
        fs::write(home.join("memory/notes.md"), "Notizen").unwrap();

        let core = test_core(home);
        let info = core.boot_info();
        assert!(info.has_soul);
        assert!(info.has_user);
        assert!(info.has_notes);
    }

    // ==========================================================
    // build_body_section()
    // ==========================================================

    #[test]
    fn body_section_minimal() {
        let config = test_config();
        let body = build_body_section(&config);
        // 6 Schichten pruefen
        assert!(body.contains("Denken — Core"));
        assert!(body.contains("Erinnern — Memory"));
        assert!(body.contains("Wissen — Skills"));
        assert!(body.contains("Handeln — Tools"));
        assert!(body.contains("Spueren — Nerves"));
        // Inhalt
        assert!(body.contains("Neocortex"));
        assert!(body.contains("claude-sonnet-4-5-20250929"));
        assert!(body.contains("Kein MQTT"));
        assert!(body.contains("Noch keine Skills"));
        assert!(!body.contains("Hippocampus"));
        assert!(!body.contains("ShellTool"));
    }

    #[test]
    fn body_section_voll() {
        let mut config = test_config();
        config.hippocampus = Some(crate::config::AgentConfig {
            provider: "anthropic".to_string(),
            model: "claude-haiku-4-5-20251001".to_string(),
            temperature: 0.7,
            api_key_env: None,
            context_window: None,
            compact_threshold: None,
            max_turns: 10,
        });
        config.mqtt = Some(crate::config::MqttConfig {
            host: "localhost".to_string(),
            port: 1883,
        });
        config.shell = Some(crate::config::ShellConfig {
            whitelist: vec!["ls".to_string(), "cat".to_string()],
            timeout: 30,
        });

        let body = build_body_section(&config);
        assert!(body.contains("Hippocampus"));
        assert!(body.contains("claude-haiku"));
        assert!(body.contains("MQTT-Bus aktiv"));
        assert!(body.contains("localhost:1883"));
        assert!(body.contains("ShellTool"));
        assert!(body.contains("ls, cat"));
    }

    // ==========================================================
    // load_preamble()
    // ==========================================================

    #[test]
    fn preamble_enthaelt_alle_sektionen() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "# Soul\nIch bin AIUX.").unwrap();
        fs::write(home.join("memory/user.md"), "# Bruce\nMein Mensch.").unwrap();
        fs::write(home.join("memory/notes.md"), "# Notizen\nTest.").unwrap();

        let config = test_config();
        let preamble = load_preamble(&home, &config);
        assert!(preamble.contains("Ich bin AIUX"));
        assert!(preamble.contains("Mein Koerper"));
        assert!(preamble.contains("Mein Mensch"));
        assert!(preamble.contains("# Notizen"));
    }

    #[test]
    fn preamble_ohne_optionale_dateien() {
        let (_tmp, home) = test_home();
        let config = test_config();
        let preamble = load_preamble(&home, &config);
        // Koerper-Sektion ist immer da
        assert!(preamble.contains("Mein Koerper"));
        // Aber keine User/Notes-Sektion
        assert!(!preamble.contains("Mein Mensch"));
    }

    // ==========================================================
    // boot_info()
    // ==========================================================

    #[test]
    fn boot_info_nichts_vorhanden() {
        let (_tmp, home) = test_home();
        let core = test_core(home);
        let info = core.boot_info();
        assert!(!info.has_soul);
        assert!(!info.has_user);
        assert!(!info.has_notes);
        assert!(!info.mqtt_active);
        assert_eq!(info.provider, "anthropic");
        assert!(info.hippocampus_model.is_none());
        assert_eq!(info.history_count, 0);
    }
}
