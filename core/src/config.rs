// Config: Agent-Konfiguration aus TOML laden.
//
// Flaches Format: provider, model, temperature direkt auf Top-Level.
// Liegt in home/.system/config.toml.

use std::path::Path;

use serde::Deserialize;

/// Konfiguration fuer den Agent.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
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
    /// MQTT-Host. None = MQTT deaktiviert.
    #[serde(default)]
    pub mqtt_host: Option<String>,
    /// MQTT-Port. Default: 1883.
    #[serde(default)]
    pub mqtt_port: Option<u16>,
}

fn default_temperature() -> f64 {
    0.7
}

impl Config {
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
    // Config::load() - flaches Format
    // ==========================================================

    #[test]
    fn config_laden_flaches_format() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
temperature = 0.5
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.temperature, 0.5);
    }

    #[test]
    fn config_defaults_temperature() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.temperature, 0.7); // Default
    }

    #[test]
    fn config_defaults_optionale_felder() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
provider = "anthropic"
model = "test-model"
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert!(config.api_key_env.is_none());
        assert!(config.context_window.is_none());
        assert!(config.compact_threshold.is_none());
    }

    #[test]
    fn config_alle_felder_gesetzt() {
        let (_tmp, home) = test_home();
        fs::write(home.join(".system/config.toml"), r#"
provider = "mistral"
model = "mistral-large-latest"
temperature = 0.3
api_key_env = "MY_KEY"
context_window = 50000
compact_threshold = 90
"#).unwrap();

        let config = Config::load(&home).unwrap();
        assert_eq!(config.provider, "mistral");
        assert_eq!(config.api_key_env.as_deref(), Some("MY_KEY"));
        assert_eq!(config.context_window, Some(50_000));
        assert_eq!(config.compact_threshold, Some(90));
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
            provider: "anthropic".to_string(),
            model: "test".to_string(),
            temperature: 0.7,
            api_key_env: None,
            context_window: None,
            compact_threshold: None,
            mqtt_host: None,
            mqtt_port: None,
        };
        assert_eq!(config.api_key_env(), "ANTHROPIC_API_KEY");
    }

    #[test]
    fn api_key_env_custom() {
        let config = Config {
            provider: "anthropic".to_string(),
            model: "test".to_string(),
            temperature: 0.7,
            api_key_env: Some("MY_CUSTOM_KEY".to_string()),
            context_window: None,
            compact_threshold: None,
            mqtt_host: None,
            mqtt_port: None,
        };
        assert_eq!(config.api_key_env(), "MY_CUSTOM_KEY");
    }
}
