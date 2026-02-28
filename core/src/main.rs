// aiux-core: LLM Agent mit Persoenlichkeit
//
// Laedt soul.md + user.md als System-Prompt (Preamble),
// startet eine REPL im Terminal mit Streaming-Ausgabe.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, ProviderClient};
use rig::message::Message;
use rig::providers::anthropic;
use rig::streaming::{StreamedAssistantContent, StreamingChat};

/// Laedt eine Datei oder gibt einen leeren String zurueck.
fn read_file(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Baut den System-Prompt aus soul.md + user.md zusammen.
/// Soul = wer bin ich, User = mit wem rede ich.
fn load_preamble(home: &PathBuf) -> String {
    let soul = read_file(&home.join("memory/soul.md"));
    let user = read_file(&home.join("memory/user.md"));

    if user.is_empty() {
        soul
    } else {
        format!("{}\n\n---\n\n{}", soul, user)
    }
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // .env laden (sucht im aktuellen Verzeichnis)
    dotenvy::dotenv().ok();

    let home = find_home();
    let preamble = load_preamble(&home);

    if preamble.is_empty() {
        eprintln!("Warnung: Keine soul.md gefunden. Agent hat keine Persoenlichkeit.");
    }

    // Anthropic Client (ANTHROPIC_API_KEY aus .env oder Env)
    let client = anthropic::Client::from_env();

    // Agent: Preamble = soul.md + user.md
    let agent = client
        .agent("claude-opus-4-6")
        .preamble(&preamble)
        .temperature(0.7)
        .build();

    let mut history: Vec<Message> = vec![];
    let stdin = io::stdin();

    println!("AIUX v0.1.0");
    println!("Zum Beenden: quit\n");

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

        // History aktualisieren
        history.push(Message::user(&input));
        history.push(Message::assistant(&response_text));
    }

    Ok(())
}
