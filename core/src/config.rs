// Config: Agent-Konfiguration aus TOML laden.
//
// Strukturiert mit TOML Tables: [cortex], [hippocampus], [mqtt].
// Liegt in home/.system/config.toml.

use std::path::Path;

use serde::Deserialize;

/// Konfiguration fuer einen einzelnen Agent (Cortex oder Hippocampus).
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
    pub cortex: AgentConfig,
    /// Hippocampus-Config. Optional — Fallback auf Cortex-Werte.
    pub hippocampus: Option<AgentConfig>,
    /// MQTT-Config. Optional — wenn nicht vorhanden: MQTT deaktiviert.
    pub mqtt: Option<MqttConfig>,
}

fn default_temperature() -> f64 {
    0.7
}

fn default_mqtt_port() -> u16 {
    1883
}

impl Config {
    /// Provider fuer den Hippocampus (Fallback auf Cortex-Provider).
    pub fn hippocampus_provider(&self) -> &str {
        self.hippocampus
            .as_ref()
            .map(|h| h.provider.as_str())
            .unwrap_or(&self.cortex.provider)
    }

    /// Model fuer den Hippocampus (Fallback auf Cortex-Model).
    pub fn hippocampus_model(&self) -> &str {
        self.hippocampus
            .as_ref()
            .map(|h| h.model.as_str())
            .unwrap_or(&self.cortex.model)
    }

    /// Gibt den Namen der Env-Variable fuer den API-Key zurueck.
    /// Entweder explizit konfiguriert oder der Default pro Provider.
    pub fn api_key_env(&self) -> &str {
        if let Some(ref env) = self.cortex.api_key_env {
            return env;
        }
        match self.cortex.provider.as_str() {
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
    fn config_laden_nur_cortex() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[cortex]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
temperature = 0.5
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.cortex.provider, "anthropic");
        assert_eq!(config.cortex.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.cortex.temperature, 0.5);
        assert!(config.hippocampus.is_none());
        assert!(config.mqtt.is_none());
    }

    #[test]
    fn config_defaults_temperature() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[cortex]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.cortex.temperature, 0.7); // Default
    }

    #[test]
    fn config_defaults_optionale_felder() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[cortex]
provider = "anthropic"
model = "test-model"
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert!(config.cortex.api_key_env.is_none());
        assert!(config.cortex.context_window.is_none());
        assert!(config.cortex.compact_threshold.is_none());
    }

    #[test]
    fn config_alle_sections() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
[cortex]
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
        assert_eq!(config.cortex.provider, "mistral");
        assert_eq!(config.cortex.api_key_env.as_deref(), Some("MY_KEY"));
        assert_eq!(config.cortex.context_window, Some(50_000));
        assert_eq!(config.cortex.compact_threshold, Some(90));
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
[cortex]
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
[cortex]
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
            cortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "test".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
            },
            hippocampus: None,
            mqtt: None,
        };
        assert_eq!(config.api_key_env(), "ANTHROPIC_API_KEY");
    }

    #[test]
    fn api_key_env_custom() {
        let config = Config {
            cortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "test".to_string(),
                temperature: 0.7,
                api_key_env: Some("MY_CUSTOM_KEY".to_string()),
                context_window: None,
                compact_threshold: None,
            },
            hippocampus: None,
            mqtt: None,
        };
        assert_eq!(config.api_key_env(), "MY_CUSTOM_KEY");
    }

    // ==========================================================
    // hippocampus_provider() / hippocampus_model()
    // ==========================================================

    #[test]
    fn hippocampus_fallback_auf_cortex() {
        let config = Config {
            cortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
            },
            hippocampus: None,
            mqtt: None,
        };
        assert_eq!(config.hippocampus_provider(), "anthropic");
        assert_eq!(config.hippocampus_model(), "claude-sonnet");
    }

    #[test]
    fn hippocampus_eigenes_model() {
        let config = Config {
            cortex: AgentConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
            },
            hippocampus: Some(AgentConfig {
                provider: "ollama".to_string(),
                model: "llama3".to_string(),
                temperature: 0.7,
                api_key_env: None,
                context_window: None,
                compact_threshold: None,
            }),
            mqtt: None,
        };
        assert_eq!(config.hippocampus_provider(), "ollama");
        assert_eq!(config.hippocampus_model(), "llama3");
    }
}
