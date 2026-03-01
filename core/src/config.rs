// Config: Agent-Konfiguration aus TOML laden.
//
// Steuert Provider, Modell und Temperature ohne neu zu kompilieren.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// Konfiguration fuer einen einzelnen Agent.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub provider: String,
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    /// Env-Variable fuer den API-Key. Falls nicht gesetzt, wird der Default pro Provider genutzt.
    pub api_key_env: Option<String>,
}

fn default_temperature() -> f64 {
    0.7
}

impl AgentConfig {
    /// Gibt den Namen der Env-Variable fuer den API-Key zurueck.
    /// Entweder explizit konfiguriert oder der Default pro Provider.
    pub fn api_key_env(&self) -> &str {
        if let Some(ref env) = self.api_key_env {
            return env;
        }
        match self.provider.as_str() {
            "anthropic" => "ANTHROPIC_API_KEY",
            "mistral" => "MISTRAL_API_KEY",
            "ollama" => "", // Ollama braucht keinen API-Key
            other => {
                eprintln!("Unbekannter Provider '{}', kein Default-API-Key bekannt.", other);
                ""
            }
        }
    }
}

/// Top-Level Config mit benannten Agents.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub agents: HashMap<String, AgentConfig>,
}

impl Config {
    /// Laedt die Config aus home/config.toml.
    pub fn load(home: &Path) -> Result<Self, anyhow::Error> {
        let path = home.join("config.toml");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Config nicht gefunden ({}): {}", path.display(), e))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Config-Fehler in {}: {}", path.display(), e))?;
        Ok(config)
    }

    /// Gibt die Config fuer den "main" Agent zurueck.
    pub fn main_agent(&self) -> Result<&AgentConfig, anyhow::Error> {
        self.agents
            .get("main")
            .ok_or_else(|| anyhow::anyhow!("Kein [agents.main] in config.toml gefunden"))
    }
}
