// Config: Agent-Konfiguration aus TOML laden.
//
// Strukturiert mit TOML Tables: [neocortex], [hippocampus], [mqtt], [shell].
// Liegt in home/.system/config.toml.

use std::path::Path;

use serde::Deserialize;

/// Konfiguration fuer einen einzelnen Agent (Neocortex oder Hippocampus).
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub provider: String,
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    /// Env-Variable fuer den API-Key. Falls nicht gesetzt, wird der Default pro Provider genutzt.
    pub api_key_env: Option<String>,
    /// Context-Window Override (in Tokens). Wenn nicht gesetzt, wird anhand des Modells geschaetzt.
    #[serde(default)]
    pub context_window: Option<u64>,
    /// Kompaktifizierungs-Schwelle in Prozent (0 = aus). Default: 80.
    #[serde(default)]
    pub compact_threshold: Option<u64>,
    /// Max Tool-Use Turns pro Anfrage. Default: 10.
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,
}

/// MQTT-Konfiguration (Nervensystem).
#[derive(Debug, Clone, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
}

/// Gesamtkonfiguration mit TOML Tables.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub neocortex: AgentConfig,
    /// Hippocampus-Config. Optional — Fallback auf Neocortex-Werte.
    pub hippocampus: Option<AgentConfig>,
    /// MQTT-Config. Optional — wenn nicht vorhanden: MQTT deaktiviert.
    pub mqtt: Option<MqttConfig>,
    /// Shell-Config. Optional — ohne = ShellTool deaktiviert.
    pub shell: Option<ShellConfig>,
}

fn default_temperature() -> f64 {
    0.7
}

fn default_max_turns() -> usize {
    10
}

fn default_mqtt_port() -> u16 {
    1883
}

fn default_shell_timeout() -> u64 {
    30
}

/// Shell-Konfiguration (Tool). Eigene Section weil Shell kein Agent ist.
#[derive(Debug, Clone, Deserialize)]
pub struct ShellConfig {
    pub whitelist: Vec<String>,
    #[serde(default = "default_shell_timeout")]
    pub timeout: u64,
}

impl Config {
    /// Provider fuer den Hippocampus (Fallback auf Neocortex-Provider).
    pub fn hippocampus_provider(&self) -> &str {
        self.hippocampus
            .as_ref()
            .map(|h| h.provider.as_str())
            .unwrap_or(&self.neocortex.provider)
    }

    /// Model fuer den Hippocampus (Fallback auf Neocortex-Model).
    pub fn hippocampus_model(&self) -> &str {
        self.hippocampus
            .as_ref()
            .map(|h| h.model.as_str())
            .unwrap_or(&self.neocortex.model)
    }

    /// Gibt den Namen der Env-Variable fuer den API-Key zurueck.
    /// Entweder explizit konfiguriert oder der Default pro Provider.
    pub fn api_key_env(&self) -> &str {
        if let Some(ref env) = self.neocortex.api_key_env {
            return env;
        }
        match self.neocortex.provider.as_str() {
            "anthropic" => "ANTHROPIC_API_KEY",
            "mistral" => "MISTRAL_API_KEY",
            "ollama" => "", // Ollama braucht keinen API-Key
            other => {
                eprintln!("Unbekannter Provider '{}', kein Default-API-Key bekannt.", other);
                ""
            }
        }
    }

    /// Laedt die Config aus home/.system/config.toml.
    pub fn load(home: &Path) -> Result<Self, anyhow::Error> {
        let path = home.join(".system/config.toml");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Config nicht gefunden ({}): {}", path.display(), e))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Config-Fehler in {}: {}", path.display(), e))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn test_home() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join(".system")).unwrap();
        (tmp, home)
    }

    // ==========================================================
    // Config::load() - TOML Tables
    // ==========================================================

    #[test]
    fn config_laden_nur_neocortex() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
temperature = 0.5
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.neocortex.provider, "anthropic");
        assert_eq!(config.neocortex.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.neocortex.temperature, 0.5);
        assert!(config.hippocampus.is_none());
        assert!(config.mqtt.is_none());
        assert!(config.shell.is_none());
    }

    #[test]
    fn config_defaults_temperature() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.neocortex.temperature, 0.7); // Default
    }

    #[test]
    fn config_defaults_optionale_felder() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "anthropic"
model = "test-model"
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert!(config.neocortex.api_key_env.is_none());
        assert!(config.neocortex.context_window.is_none());
        assert!(config.neocortex.compact_threshold.is_none());
        assert_eq!(config.neocortex.max_turns, 10); // Default
    }

    #[test]
    fn config_alle_sections() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "mistral"
model = "mistral-large-latest"
temperature = 0.3
api_key_env = "MY_KEY"
context_window = 50000
compact_threshold = 90

[hippocampus]
provider = "anthropic"
model = "claude-haiku-4-5-20251001"

[mqtt]
host = "localhost"
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.neocortex.provider, "mistral");
        assert_eq!(config.neocortex.api_key_env.as_deref(), Some("MY_KEY"));
        assert_eq!(config.neocortex.context_window, Some(50_000));
        assert_eq!(config.neocortex.compact_threshold, Some(90));
        assert_eq!(config.neocortex.max_turns, 10); // Default
        let hc = config.hippocampus.as_ref().unwrap();
        assert_eq!(hc.provider, "anthropic");
        assert_eq!(hc.model, "claude-haiku-4-5-20251001");
        let mqtt = config.mqtt.as_ref().unwrap();
        assert_eq!(mqtt.host, "localhost");
        assert_eq!(mqtt.port, 1883); // Default
    }

    #[test]
    fn config_mqtt_mit_port() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "anthropic"
model = "test"

[mqtt]
host = "192.168.1.1"
port = 9883
"#).unwrap();

        let config = Config::load(&home).unwrap();
        let mqtt = config.mqtt.as_ref().unwrap();
        assert_eq!(mqtt.host, "192.168.1.1");
        assert_eq!(mqtt.port, 9883);
    }

    #[test]
    fn config_fehlende_datei() {
        let (_tmp, home) = test_home();
        let result = Config::load(&home);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Config nicht gefunden"));
    }

    #[test]
    fn config_fehlende_pflichtfelder() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "anthropic"
"#).unwrap();

        let result = Config::load(&home);
        assert!(result.is_err()); // model fehlt
    }

    #[test]
    fn config_kaputtes_toml() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), "das ist {{kein toml").unwrap();

        let result = Config::load(&home);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Config-Fehler"));
    }

    // ==========================================================
    // api_key_env()
    // ==========================================================

    #[test]
    fn api_key_env_default_anthropic() {
        let config = Config {
            neocortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "test".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
                max_turns: 10,
            },
            hippocampus: None,
            mqtt: None,
            shell: None,
        };
        assert_eq!(config.api_key_env(), "ANTHROPIC_API_KEY");
    }

    #[test]
    fn api_key_env_custom() {
        let config = Config {
            neocortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "test".to_string(),
                temperature: 0.7,
                api_key_env: Some("MY_CUSTOM_KEY".to_string()),
                context_window: None,
                compact_threshold: None,
                max_turns: 10,
            },
            hippocampus: None,
            mqtt: None,
            shell: None,
        };
        assert_eq!(config.api_key_env(), "MY_CUSTOM_KEY");
    }

    // ==========================================================
    // hippocampus_provider() / hippocampus_model()
    // ==========================================================

    #[test]
    fn hippocampus_fallback_auf_neocortex() {
        let config = Config {
            neocortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
                max_turns: 10,
            },
            hippocampus: None,
            mqtt: None,
            shell: None,
        };
        assert_eq!(config.hippocampus_provider(), "anthropic");
        assert_eq!(config.hippocampus_model(), "claude-sonnet");
    }

    #[test]
    fn hippocampus_eigenes_model() {
        let config = Config {
            neocortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
                max_turns: 10,
            },
            hippocampus: Some(AgentConfig {
                provider: "ollama".to_string(),
                model: "llama3".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
                max_turns: 10,
            }),
            mqtt: None,
            shell: None,
        };
        assert_eq!(config.hippocampus_provider(), "ollama");
        assert_eq!(config.hippocampus_model(), "llama3");
    }

    // ==========================================================
    // ShellConfig
    // ==========================================================

    #[test]
    fn config_mit_shell() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "anthropic"
model = "test"

[shell]
whitelist = ["ls", "cat", "echo"]
timeout = 15
"#).unwrap();

        let config = Config::load(&home).unwrap();
        let shell = config.shell.as_ref().unwrap();
        assert_eq!(shell.whitelist, vec!["ls", "cat", "echo"]);
        assert_eq!(shell.timeout, 15);
    }

    #[test]
    fn config_shell_default_timeout() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[neocortex]
provider = "anthropic"
model = "test"

[shell]
whitelist = ["ls"]
"#).unwrap();

        let config = Config::load(&home).unwrap();
        let shell = config.shell.as_ref().unwrap();
        assert_eq!(shell.timeout, 30); // Default
    }
}
