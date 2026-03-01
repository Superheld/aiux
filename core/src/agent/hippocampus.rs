// Hippocampus: Automatische Gedaechtnisbildung.
//
// Separater Agent mit eigener Preamble (compact-preamble.md).
// Wird vom Core aufgerufen bei:
// - Kompaktifizierung (Token-Schwellwert erreicht)
// - Memory-Flush (/clear, /quit)
//
// Kein Sub-Agent im rig-Sinne (nicht per .tool() eingehaengt),
// sondern ein eigenstaendiger LLM-Call den der Core steuert.

use std::fs;
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
    let preamble = fs::read_to_string(home.join(".system/compact-preamble.md"))
        .unwrap_or_else(|_| {
            "Du bist der Hippocampus. Destilliere Wichtiges aus der Konversation \
             und speichere es ueber die Tools. Fasse den Rest zusammen."
                .to_string()
        });

    let soul_tool = SoulTool::new(home, Arc::clone(preamble_dirty));
    let user_tool = UserTool::new(home, Arc::clone(preamble_dirty));
    let memory_tool = MemoryTool::new(home, Arc::clone(preamble_dirty));

    match config.provider.as_str() {
        "anthropic" => {
            let client = anthropic::Client::from_env();
            let agent = client
                .agent(&config.model)
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
                .agent(&config.model)
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
                .agent(&config.model)
                .preamble(&preamble)
                .temperature(0.3)
                .tool(soul_tool)
                .tool(user_tool)
                .tool(memory_tool)
                .build();
            Ok(agent.chat(prompt, vec![]).await?)
        }
        other => anyhow::bail!("Unbekannter Provider: '{}'", other),
    }
}
