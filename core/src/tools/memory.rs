// MemoryTool: Notizen des Agents.
//
// Ziel: memory/notes.md
// Arbeitsnotizen, Gelerntes, offene Fragen.
// Notizen ueberleben Sessions und werden beim Start in den Preamble geladen.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;

use super::{execute_single_file, ToolArgs, ToolError, ToolResult};

const DESCRIPTION: &str = "Speichere und lese Notizen. \
    Nutze dieses Tool fuer Arbeitsnotizen, Gelerntes, Entscheidungen, offene Fragen. \
    Notizen ueberleben Sessions und werden beim Start automatisch geladen.";

pub struct MemoryTool {
    path: PathBuf,
    description: String,
    preamble_dirty: Arc<AtomicBool>,
}

impl MemoryTool {
    pub fn new(home: &Path, preamble_dirty: Arc<AtomicBool>) -> Self {
        Self {
            path: home.join("memory/notes.md"),
            description: DESCRIPTION.to_string(),
            preamble_dirty,
        }
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
                        "enum": ["read", "write", "edit", "append"],
                        "description": "Aktion: read, write, edit oder append"
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
        execute_single_file(&self.path, &args, &self.preamble_dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::Ordering;
    use tempfile::TempDir;

    fn test_tool() -> (TempDir, MemoryTool) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = MemoryTool::new(&home, dirty);
        (tmp, tool)
    }

    fn test_tool_with_dirty() -> (TempDir, MemoryTool, Arc<AtomicBool>) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = MemoryTool::new(&home, Arc::clone(&dirty));
        (tmp, tool, dirty)
    }

    fn args(action: &str, content: &str) -> ToolArgs {
        ToolArgs {
            action: action.to_string(),
            content: content.to_string(),
            old_content: String::new(),
            new_content: String::new(),
        }
    }

    fn edit_args(old: &str, new: &str) -> ToolArgs {
        ToolArgs {
            action: "edit".to_string(),
            content: String::new(),
            old_content: old.to_string(),
            new_content: new.to_string(),
        }
    }

    // ==========================================================
    // read
    // ==========================================================

    #[tokio::test]
    async fn read_existierende_datei() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/notes.md"), "Hallo").unwrap();

        let result = tool.call(args("read", "")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Hallo");
    }

    #[tokio::test]
    async fn read_nicht_vorhanden() {
        let (_tmp, tool) = test_tool();

        let result = tool.call(args("read", "")).await.unwrap();
        assert!(!result.success);
        assert!(result.message.contains("nicht gefunden"));
    }

    // ==========================================================
    // write
    // ==========================================================

    #[tokio::test]
    async fn write_neue_datei() {
        let (tmp, tool) = test_tool();

        let result = tool.call(args("write", "Neue Notiz")).await.unwrap();
        assert!(result.success);

        let content = fs::read_to_string(tmp.path().join("memory/notes.md")).unwrap();
        assert_eq!(content, "Neue Notiz");
    }

    #[tokio::test]
    async fn write_ueberschreibt() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/notes.md"), "Alt").unwrap();

        tool.call(args("write", "Neu")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/notes.md")).unwrap();
        assert_eq!(content, "Neu");
    }

    #[tokio::test]
    async fn write_setzt_dirty_flag() {
        let (_tmp, tool, dirty) = test_tool_with_dirty();

        tool.call(args("write", "Inhalt")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    // ==========================================================
    // edit
    // ==========================================================

    #[tokio::test]
    async fn edit_ersetzt_abschnitt() {
        let (tmp, tool) = test_tool();
        fs::write(
            tmp.path().join("memory/notes.md"),
            "Rust ist toll und schnell.",
        )
        .unwrap();

        let result = tool.call(edit_args("toll", "super")).await.unwrap();
        assert!(result.success);

        let content = fs::read_to_string(tmp.path().join("memory/notes.md")).unwrap();
        assert_eq!(content, "Rust ist super und schnell.");
    }

    #[tokio::test]
    async fn edit_old_content_nicht_gefunden() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/notes.md"), "Inhalt").unwrap();

        let result = tool.call(edit_args("gibts nicht", "egal")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn edit_setzt_dirty_flag() {
        let (tmp, tool, dirty) = test_tool_with_dirty();
        fs::write(tmp.path().join("memory/notes.md"), "Alt").unwrap();

        tool.call(edit_args("Alt", "Neu")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    // ==========================================================
    // append
    // ==========================================================

    #[tokio::test]
    async fn append_an_bestehende_datei() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/notes.md"), "Zeile 1").unwrap();

        tool.call(args("append", "Zeile 2")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/notes.md")).unwrap();
        assert_eq!(content, "Zeile 1\nZeile 2");
    }

    #[tokio::test]
    async fn append_an_nicht_existierende_datei() {
        let (tmp, tool) = test_tool();

        tool.call(args("append", "Erster Inhalt")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/notes.md")).unwrap();
        assert_eq!(content, "Erster Inhalt");
    }

    #[tokio::test]
    async fn append_setzt_dirty_flag() {
        let (_tmp, tool, dirty) = test_tool_with_dirty();

        tool.call(args("append", "Inhalt")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    // ==========================================================
    // Beschreibung
    // ==========================================================

    #[tokio::test]
    async fn beschreibung() {
        let (_tmp, tool) = test_tool();
        let def = tool.definition(String::new()).await;
        assert!(def.description.contains("Notizen"));
    }

    // ==========================================================
    // Unbekannte Aktion
    // ==========================================================

    #[tokio::test]
    async fn unbekannte_aktion() {
        let (_tmp, tool) = test_tool();
        let result = tool.call(args("delete", "")).await;
        assert!(result.is_err());
    }
}
