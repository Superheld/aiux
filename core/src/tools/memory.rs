// MemoryTool: Arbeitsnotizen im Langzeitgedaechtnis.
//
// Ziel: memory/context/{key}.md
// Entscheidungen, Gelerntes, Projektnotizen, offene Fragen.
// Notizen ueberleben Sessions und werden beim Start in den Preamble geladen.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;

use super::{load_description, ToolArgs, ToolError, ToolResult};

const DEFAULT_DESCRIPTION: &str = "Speichere und lese Arbeitsnotizen im Langzeitgedaechtnis. \
    Nutze dieses Tool fuer Entscheidungen, Gelerntes, Projektnotizen, offene Fragen. \
    Notizen ueberleben Sessions und werden beim Start automatisch geladen.";

pub struct MemoryTool {
    context_dir: PathBuf,
    description: String,
    preamble_dirty: Arc<AtomicBool>,
}

impl MemoryTool {
    pub fn new(home: &Path, preamble_dirty: Arc<AtomicBool>) -> Self {
        let context_dir = home.join("memory/context");
        fs::create_dir_all(&context_dir).ok();
        let description = load_description(home, "tool-memory.md", DEFAULT_DESCRIPTION);
        Self {
            context_dir,
            description,
            preamble_dirty,
        }
    }

    /// Pfad zur Notiz-Datei. Prueft auf Pfad-Traversal.
    fn note_path(&self, key: &str) -> Result<PathBuf, ToolError> {
        if key.is_empty() {
            return Err(ToolError("key ist erforderlich".into()));
        }
        if key.contains('/') || key.contains("..") {
            return Err(ToolError("key darf keine Pfade enthalten".into()));
        }
        Ok(self.context_dir.join(format!("{}.md", key)))
    }
}

impl Tool for MemoryTool {
    const NAME: &'static str = "memory";

    type Error = ToolError;
    type Args = ToolArgs;
    type Output = ToolResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "memory".to_string(),
            description: self.description.clone(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["read", "write", "edit", "append", "list"],
                        "description": "Aktion: read, write, edit, append oder list"
                    },
                    "key": {
                        "type": "string",
                        "description": "Name der Notiz (ohne .md). Beispiel: 'projekte', 'ideen', 'todo'"
                    },
                    "content": {
                        "type": "string",
                        "description": "Inhalt (bei write: komplett neu, bei append: anzufuegen)"
                    },
                    "old_content": {
                        "type": "string",
                        "description": "Text der ersetzt werden soll (nur bei edit)"
                    },
                    "new_content": {
                        "type": "string",
                        "description": "Neuer Text (nur bei edit)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.action.as_str() {
            "read" => {
                let path = self.note_path(&args.key)?;
                match fs::read_to_string(&path) {
                    Ok(content) => Ok(ToolResult {
                        success: true,
                        message: content,
                    }),
                    Err(_) => Ok(ToolResult {
                        success: false,
                        message: format!("Notiz '{}' nicht gefunden.", args.key),
                    }),
                }
            }
            "write" => {
                let path = self.note_path(&args.key)?;
                fs::write(&path, &args.content)
                    .map_err(|e| ToolError(format!("Schreibfehler: {}", e)))?;
                self.preamble_dirty.store(true, Ordering::Relaxed);
                Ok(ToolResult {
                    success: true,
                    message: format!("Notiz '{}' gespeichert.", args.key),
                })
            }
            "edit" => {
                let path = self.note_path(&args.key)?;
                if args.old_content.is_empty() {
                    return Err(ToolError("old_content ist erforderlich bei edit".into()));
                }
                let content = fs::read_to_string(&path)
                    .map_err(|e| ToolError(format!("Lesefehler: {}", e)))?;
                if !content.contains(&args.old_content) {
                    return Err(ToolError("old_content nicht gefunden.".into()));
                }
                let new_content = content.replace(&args.old_content, &args.new_content);
                fs::write(&path, &new_content)
                    .map_err(|e| ToolError(format!("Schreibfehler: {}", e)))?;
                self.preamble_dirty.store(true, Ordering::Relaxed);
                Ok(ToolResult {
                    success: true,
                    message: format!("Notiz '{}' bearbeitet.", args.key),
                })
            }
            "append" => {
                let path = self.note_path(&args.key)?;
                let existing = fs::read_to_string(&path).unwrap_or_default();
                let new_content = if existing.is_empty() {
                    args.content.clone()
                } else {
                    format!("{}\n{}", existing, args.content)
                };
                fs::write(&path, &new_content)
                    .map_err(|e| ToolError(format!("Schreibfehler: {}", e)))?;
                self.preamble_dirty.store(true, Ordering::Relaxed);
                Ok(ToolResult {
                    success: true,
                    message: format!("An Notiz '{}' angefuegt.", args.key),
                })
            }
            "list" => {
                let mut names: Vec<String> = Vec::new();
                if let Ok(entries) = fs::read_dir(&self.context_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|ext| ext == "md") {
                            if let Some(name) = path.file_stem() {
                                names.push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
                names.sort();
                if names.is_empty() {
                    Ok(ToolResult {
                        success: true,
                        message: "Keine Notizen vorhanden.".into(),
                    })
                } else {
                    Ok(ToolResult {
                        success: true,
                        message: names.join(", "),
                    })
                }
            }
            other => Err(ToolError(format!(
                "Unbekannte Aktion '{}'. Erlaubt: read, write, edit, append, list",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use tempfile::TempDir;

    fn test_tool() -> (TempDir, MemoryTool) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = MemoryTool::new(&home, dirty);
        (tmp, tool)
    }

    fn test_tool_with_dirty() -> (TempDir, MemoryTool, Arc<AtomicBool>) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = MemoryTool::new(&home, Arc::clone(&dirty));
        (tmp, tool, dirty)
    }

    fn args(action: &str, key: &str, content: &str) -> ToolArgs {
        ToolArgs {
            action: action.to_string(),
            content: content.to_string(),
            old_content: String::new(),
            new_content: String::new(),
            key: key.to_string(),
        }
    }

    fn edit_args(key: &str, old: &str, new: &str) -> ToolArgs {
        ToolArgs {
            action: "edit".to_string(),
            content: String::new(),
            old_content: old.to_string(),
            new_content: new.to_string(),
            key: key.to_string(),
        }
    }

    // ==========================================================
    // read
    // ==========================================================

    #[tokio::test]
    async fn read_existierende_notiz() {
        let (_tmp, tool) = test_tool();
        tool.call(args("write", "test", "Hallo")).await.unwrap();

        let result = tool.call(args("read", "test", "")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Hallo");
    }

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
        let (_tmp, tool, dirty) = test_tool_with_dirty();
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
    // edit
    // ==========================================================

    #[tokio::test]
    async fn edit_ersetzt_abschnitt() {
        let (_tmp, tool) = test_tool();
        tool.call(args("write", "test", "Rust ist toll und schnell.")).await.unwrap();

        let result = tool.call(edit_args("test", "toll", "super")).await.unwrap();
        assert!(result.success);

        let result = tool.call(args("read", "test", "")).await.unwrap();
        assert_eq!(result.message, "Rust ist super und schnell.");
    }

    #[tokio::test]
    async fn edit_old_content_nicht_gefunden() {
        let (_tmp, tool) = test_tool();
        tool.call(args("write", "test", "Inhalt")).await.unwrap();

        let result = tool.call(edit_args("test", "gibts nicht", "egal")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn edit_setzt_dirty_flag() {
        let (_tmp, tool, dirty) = test_tool_with_dirty();
        tool.call(args("write", "test", "Alt")).await.unwrap();
        dirty.store(false, Ordering::Relaxed); // Reset nach write

        tool.call(edit_args("test", "Alt", "Neu")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    // ==========================================================
    // append
    // ==========================================================

    #[tokio::test]
    async fn append_an_bestehende_notiz() {
        let (_tmp, tool) = test_tool();
        tool.call(args("write", "test", "Zeile 1")).await.unwrap();

        tool.call(args("append", "test", "Zeile 2")).await.unwrap();

        let result = tool.call(args("read", "test", "")).await.unwrap();
        assert_eq!(result.message, "Zeile 1\nZeile 2");
    }

    #[tokio::test]
    async fn append_an_nicht_existierende_notiz() {
        let (_tmp, tool) = test_tool();

        tool.call(args("append", "neu", "Erster Inhalt")).await.unwrap();

        let result = tool.call(args("read", "neu", "")).await.unwrap();
        assert_eq!(result.message, "Erster Inhalt");
    }

    #[tokio::test]
    async fn append_setzt_dirty_flag() {
        let (_tmp, tool, dirty) = test_tool_with_dirty();
        tool.call(args("append", "test", "Inhalt")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
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
    // Beschreibung
    // ==========================================================

    #[tokio::test]
    async fn beschreibung_aus_datei() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join(".system")).unwrap();
        fs::write(home.join(".system/tool-memory.md"), "Custom Memory Beschreibung").unwrap();

        let dirty = Arc::new(AtomicBool::new(false));
        let tool = MemoryTool::new(&home, dirty);
        let def = tool.definition(String::new()).await;
        assert_eq!(def.description, "Custom Memory Beschreibung");
    }

    #[tokio::test]
    async fn beschreibung_fallback() {
        let (_tmp, tool) = test_tool();
        let def = tool.definition(String::new()).await;
        assert!(def.description.contains("Langzeitgedaechtnis"));
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
