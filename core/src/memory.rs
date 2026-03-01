// Memory-Tool: Erlaubt dem Agent in sein Gedaechtnis zu schreiben.
//
// Schreibt Markdown-Dateien in memory/context/.
// Der Agent kann Notizen ablegen die beim naechsten Start
// automatisch in den Preamble geladen werden.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Das Memory-Tool. Haelt den Pfad zum context/ Verzeichnis.
pub struct MemoryTool {
    context_dir: PathBuf,
    preamble_dirty: Arc<AtomicBool>,
}

impl MemoryTool {
    pub fn new(home: &PathBuf, preamble_dirty: Arc<AtomicBool>) -> Self {
        let context_dir = home.join("memory/context");
        // Sicherstellen dass das Verzeichnis existiert
        fs::create_dir_all(&context_dir).ok();
        Self {
            context_dir,
            preamble_dirty,
        }
    }
}

/// Argumente fuer das Memory-Tool.
/// action: "write" oder "read" oder "list"
/// key: Dateiname (ohne .md)
/// content: Inhalt (nur bei write)
#[derive(Deserialize, JsonSchema)]
pub struct MemoryArgs {
    /// Aktion: "write" (Notiz speichern), "read" (Notiz lesen), "list" (alle Notizen auflisten)
    pub action: String,
    /// Name der Notiz (ohne .md Endung). Wird als Dateiname verwendet.
    #[serde(default)]
    pub key: String,
    /// Inhalt der Notiz (nur bei action "write")
    #[serde(default)]
    pub content: String,
}

#[derive(Serialize)]
pub struct MemoryResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct MemoryError(String);

impl Tool for MemoryTool {
    const NAME: &'static str = "memory";

    type Error = MemoryError;
    type Args = MemoryArgs;
    type Output = MemoryResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "memory".to_string(),
            description: "Speichere oder lese Notizen im Langzeitgedaechtnis. \
                Notizen ueberleben Sessions und werden beim naechsten Start automatisch geladen. \
                Aktionen: 'write' (speichern), 'read' (lesen), 'list' (alle auflisten)."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["write", "read", "list"],
                        "description": "Aktion: write, read oder list"
                    },
                    "key": {
                        "type": "string",
                        "description": "Name der Notiz (ohne .md). Beispiel: 'projekte', 'ideen', 'todo'"
                    },
                    "content": {
                        "type": "string",
                        "description": "Inhalt der Notiz (nur bei write)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.action.as_str() {
            "write" => {
                if args.key.is_empty() {
                    return Err(MemoryError("key ist erforderlich bei write".into()));
                }
                // Sicherheit: nur einfache Dateinamen erlauben
                if args.key.contains('/') || args.key.contains("..") {
                    return Err(MemoryError("key darf keine Pfade enthalten".into()));
                }
                let path = self.context_dir.join(format!("{}.md", args.key));
                fs::write(&path, &args.content)
                    .map_err(|e| MemoryError(format!("Schreibfehler: {}", e)))?;
                // Preamble muss beim naechsten Input neu geladen werden
                self.preamble_dirty.store(true, Ordering::Relaxed);
                Ok(MemoryResult {
                    success: true,
                    message: format!("Notiz '{}' gespeichert.", args.key),
                })
            }
            "read" => {
                if args.key.is_empty() {
                    return Err(MemoryError("key ist erforderlich bei read".into()));
                }
                let path = self.context_dir.join(format!("{}.md", args.key));
                match fs::read_to_string(&path) {
                    Ok(content) => Ok(MemoryResult {
                        success: true,
                        message: content,
                    }),
                    Err(_) => Ok(MemoryResult {
                        success: false,
                        message: format!("Notiz '{}' nicht gefunden.", args.key),
                    }),
                }
            }
            "list" => {
                let mut names: Vec<String> = Vec::new();
                if let Ok(entries) = fs::read_dir(&self.context_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "md") {
                            if let Some(name) = path.file_stem() {
                                names.push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
                names.sort();
                if names.is_empty() {
                    Ok(MemoryResult {
                        success: true,
                        message: "Keine Notizen vorhanden.".into(),
                    })
                } else {
                    Ok(MemoryResult {
                        success: true,
                        message: names.join(", "),
                    })
                }
            }
            other => Err(MemoryError(format!(
                "Unbekannte Aktion '{}'. Erlaubt: write, read, list",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_tool() -> (TempDir, MemoryTool) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = MemoryTool::new(&home, dirty);
        (tmp, tool)
    }

    fn args(action: &str, key: &str, content: &str) -> MemoryArgs {
        MemoryArgs {
            action: action.to_string(),
            key: key.to_string(),
            content: content.to_string(),
        }
    }

    // ==========================================================
    // write
    // ==========================================================

    #[tokio::test]
    async fn write_und_lesen() {
        let (_tmp, tool) = test_tool();

        let result = tool.call(args("write", "test", "Hallo Welt")).await.unwrap();
        assert!(result.success);

        let result = tool.call(args("read", "test", "")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Hallo Welt");
    }

    #[tokio::test]
    async fn write_ueberschreibt() {
        let (_tmp, tool) = test_tool();

        tool.call(args("write", "test", "Eins")).await.unwrap();
        tool.call(args("write", "test", "Zwei")).await.unwrap();

        let result = tool.call(args("read", "test", "")).await.unwrap();
        assert_eq!(result.message, "Zwei");
    }

    #[tokio::test]
    async fn write_setzt_dirty_flag() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = MemoryTool::new(&home, Arc::clone(&dirty));

        tool.call(args("write", "test", "Inhalt")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn write_ohne_key() {
        let (_tmp, tool) = test_tool();
        let result = tool.call(args("write", "", "Inhalt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn write_pfad_im_key_verboten() {
        let (_tmp, tool) = test_tool();

        let result = tool.call(args("write", "../etc/passwd", "hack")).await;
        assert!(result.is_err());

        let result = tool.call(args("write", "sub/dir", "hack")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn write_unicode_key() {
        let (_tmp, tool) = test_tool();

        let result = tool.call(args("write", "notizen-über-käse", "Gouda")).await.unwrap();
        assert!(result.success);

        let result = tool.call(args("read", "notizen-über-käse", "")).await.unwrap();
        assert_eq!(result.message, "Gouda");
    }

    // ==========================================================
    // read
    // ==========================================================

    #[tokio::test]
    async fn read_nicht_vorhanden() {
        let (_tmp, tool) = test_tool();
        let result = tool.call(args("read", "gibts_nicht", "")).await.unwrap();
        assert!(!result.success);
        assert!(result.message.contains("nicht gefunden"));
    }

    #[tokio::test]
    async fn read_ohne_key() {
        let (_tmp, tool) = test_tool();
        let result = tool.call(args("read", "", "")).await;
        assert!(result.is_err());
    }

    // ==========================================================
    // list
    // ==========================================================

    #[tokio::test]
    async fn list_leer() {
        let (_tmp, tool) = test_tool();
        let result = tool.call(args("list", "", "")).await.unwrap();
        assert!(result.success);
        assert!(result.message.contains("Keine Notizen"));
    }

    #[tokio::test]
    async fn list_mehrere_sortiert() {
        let (_tmp, tool) = test_tool();
        tool.call(args("write", "zoo", "Z")).await.unwrap();
        tool.call(args("write", "apfel", "A")).await.unwrap();
        tool.call(args("write", "mitte", "M")).await.unwrap();

        let result = tool.call(args("list", "", "")).await.unwrap();
        assert_eq!(result.message, "apfel, mitte, zoo");
    }

    // ==========================================================
    // Unbekannte Aktion
    // ==========================================================

    #[tokio::test]
    async fn unbekannte_aktion() {
        let (_tmp, tool) = test_tool();
        let result = tool.call(args("delete", "test", "")).await;
        assert!(result.is_err());
    }
}
