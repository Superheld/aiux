// aiux-core: LLM Agent mit Persoenlichkeit
//
// Boot-Sequence: soul.md -> user.md -> context/*.md
// Alles wird zum System-Prompt (Preamble) zusammengebaut.
// Startet eine REPL im Terminal mit Streaming-Ausgabe.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

mod memory;

use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, ProviderClient};
use rig::message::Message;
use rig::providers::anthropic;
use rig::streaming::{StreamedAssistantContent, StreamingChat};

use memory::MemoryTool;

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

/// Baut den System-Prompt zusammen (Boot-Sequence):
/// 1. soul.md - Wer bin ich?
/// 2. user.md - Mit wem rede ich?
/// 3. context/*.md - Was weiss ich noch?
fn load_preamble(home: &PathBuf) -> String {
    let mut parts: Vec<String> = Vec::new();

    // 1. Soul
    let soul = read_file(&home.join("memory/soul.md"));
    if !soul.is_empty() {
        parts.push(soul);
    }

    // 2. User
    let user = read_file(&home.join("memory/user.md"));
    if !user.is_empty() {
        parts.push(user);
    }

    // 3. Context-Dateien
    let context_files = load_context_files(&home.join("memory/context"));
    for (name, content) in &context_files {
        parts.push(format!("# Kontext: {}\n\n{}", name, content));
    }

    parts.join("\n\n---\n\n")
}

/// Findet das home/ Verzeichnis.
/// Sucht: ./home/ (Entwicklung) oder /home/claude/ (deployed)
fn find_home() -> PathBuf {
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

/// Laedt die gespeicherte Konversations-History (oder leeren Vec).
fn load_history(home: &PathBuf) -> Vec<Message> {
    let path = home.join("memory/conversation.json");
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => vec![],
    }
}

/// Speichert die aktuelle History als JSON.
fn save_history(home: &PathBuf, history: &[Message]) {
    let path = home.join("memory/conversation.json");
    if let Ok(data) = serde_json::to_string_pretty(history) {
        fs::write(&path, data).ok();
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // .env laden (sucht im aktuellen Verzeichnis)
    dotenvy::dotenv().ok();

    let home = find_home();

    // Boot-Sequence: was wird geladen?
    let has_soul = home.join("memory/soul.md").exists();
    let has_user = home.join("memory/user.md").exists();
    let context_files = load_context_files(&home.join("memory/context"));
    let context_count = context_files.len();

    let preamble = load_preamble(&home);

    if preamble.is_empty() {
        eprintln!("Warnung: Keine soul.md gefunden. Agent hat keine Persoenlichkeit.");
    }

    // Anthropic Client (ANTHROPIC_API_KEY aus .env oder Env)
    let client = anthropic::Client::from_env();

    // Memory-Tool: Agent kann in sein Gedaechtnis schreiben
    let memory_tool = MemoryTool::new(&home);

    // Agent: Preamble + Memory-Tool
    let agent = client
        .agent("claude-sonnet-4-5-20250929")
        .preamble(&preamble)
        .temperature(0.7)
        .tool(memory_tool)
        .build();

    let mut history = load_history(&home);
    let stdin = io::stdin();

    println!("AIUX v0.1.0");
    if has_soul { println!("  [+] soul.md"); }
    if has_user { println!("  [+] user.md"); }
    if context_count > 0 {
        println!("  [+] {} Context-Datei(en)", context_count);
    }
    println!("  [+] Memory-Tool (write/read/list)");
    if !history.is_empty() {
        println!("  [+] {} History-Nachrichten", history.len());
    }
    println!("Zum Beenden: quit | clear = History loeschen\n");

    loop {
        print!("Du: ");
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }
        if input == "quit" || input == "exit" {
            println!("\nTschuess.");
            break;
        }
        if input == "clear" {
            history.clear();
            let path = home.join("memory/conversation.json");
            fs::remove_file(&path).ok();
            println!("History geloescht.\n");
            continue;
        }

        // Antwort-Label mit Leerzeile davor
        print!("\nAIUX: ");
        io::stdout().flush()?;

        // stream_chat gibt direkt einen Stream zurueck (kein Result)
        let mut stream = agent.stream_chat(&input, history.clone()).await;
        let mut response_text = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Text(text),
                )) => {
                    print!("{}", text.text);
                    io::stdout().flush()?;
                    response_text.push_str(&text.text);
                }
                Ok(_) => {
                    // ToolCall, Final, etc. - ignorieren wir erstmal
                }
                Err(e) => {
                    eprintln!("\nFehler: {}", e);
                    break;
                }
            }
        }

        // Zwei Leerzeilen nach Antwort fuer Uebersicht
        println!("\n");

        // History aktualisieren und persistieren
        history.push(Message::user(&input));
        history.push(Message::assistant(&response_text));
        save_history(&home, &history);
    }

    Ok(())
}
