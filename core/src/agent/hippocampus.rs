// Hippocampus: Automatische Gedaechtnisbildung.
//
// Wird vom Core aufgerufen bei:
// - Kompaktifizierung (Token-Schwellwert erreicht)
// - Memory-Flush (/clear, /quit)
//
// Kein Sub-Agent im rig-Sinne (nicht per .tool() eingehaengt),
// sondern ein eigenstaendiger LLM-Call den der Core steuert.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Chat;
use rig::providers::{anthropic, mistral, ollama};

use crate::config::Config;
use crate::tools::memory::MemoryTool;
use crate::tools::soul::SoulTool;
use crate::tools::user::UserTool;

/// Hippocampus-Call: Destilliert Wissen aus der Konversation in Memory-Dateien.
/// Nutzt einen Agent mit Tools (soul, user, memory) um Wichtiges zu speichern.
/// Gibt die Zusammenfassung als Text zurueck.
pub async fn hippocampus_call(
    home: &Path,
    config: &Config,
    preamble_dirty: &Arc<AtomicBool>,
    prompt: &str,
) -> Result<String, anyhow::Error> {
    let preamble = "Du bist der Hippocampus - das Gedaechtnis des Agents.\n\
        Deine Aufgabe: Wichtiges aus der Konversation destillieren und speichern.\n\n\
        1. Lies die Konversation durch\n\
        2. Schreibe Wichtiges in die passenden Memory-Dateien:\n\
           - Neues ueber dich selbst: soul Tool\n\
           - Neues ueber den User: user Tool\n\
           - Entscheidungen, Gelerntes, Projekte: memory Tool\n\
        3. Fasse den Rest der Konversation kurz zusammen\n\
        4. Die Zusammenfassung ist deine Antwort\n\n\
        Regeln:\n\
        - Nur schreiben was NEU ist (nicht was schon in den Dateien steht)\n\
        - Lies zuerst die aktuelle Datei (read) bevor du schreibst\n\
        - Konkrete Details behalten: Dateinamen, Entscheidungen, Code\n\
        - Offene Punkte markieren\n\
        - Halte die Zusammenfassung kompakt aber vollstaendig";

    let soul_tool = SoulTool::new(home, Arc::clone(preamble_dirty));
    let user_tool = UserTool::new(home, Arc::clone(preamble_dirty));
    let memory_tool = MemoryTool::new(home, Arc::clone(preamble_dirty));

    let provider = config.hippocampus_provider();
    let model = config.hippocampus_model();

    match provider {
        "anthropic" => {
            let client = anthropic::Client::from_env();
            let agent = client
                .agent(model)
                .preamble(&preamble)
                .temperature(0.3)
                .tool(soul_tool)
                .tool(user_tool)
                .tool(memory_tool)
                .build();
            Ok(agent.chat(prompt, vec![]).await?)
        }
        "mistral" => {
            let client = mistral::Client::from_env();
            let agent = client
                .agent(model)
                .preamble(&preamble)
                .temperature(0.3)
                .tool(soul_tool)
                .tool(user_tool)
                .tool(memory_tool)
                .build();
            Ok(agent.chat(prompt, vec![]).await?)
        }
        "ollama" => {
            let client: ollama::Client = ollama::Client::new(rig::client::Nothing)
                .map_err(|e| anyhow::anyhow!("Ollama-Client: {}", e))?;
            let agent = client
                .agent(model)
                .preamble(&preamble)
                .temperature(0.3)
                .tool(soul_tool)
                .tool(user_tool)
                .tool(memory_tool)
                .build();
            Ok(agent.chat(prompt, vec![]).await?)
        }
        other => anyhow::bail!("Unbekannter Provider fuer Hippocampus: '{}'", other),
    }
}
