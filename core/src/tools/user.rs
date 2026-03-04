// UserTool: Wissen ueber den Menschen.
//
// Ziel: memory/user.md
// Der Agent kann sein Bild vom User lesen und aktualisieren:
// Vorlieben, Skills, Gewohnheiten, Projekte.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;

use super::{execute_single_file, ToolArgs, ToolError, ToolResult};

const DESCRIPTION: &str = "Aktualisiere dein Wissen ueber deinen Menschen. \
    Nutze dieses Tool wenn du Neues ueber Bruce erfahren hast - \
    Vorlieben, Skills, Gewohnheiten, Projekte. Die user.md ist dein Bild von ihm.";

pub struct UserTool {
    path: PathBuf,
    description: String,
    preamble_dirty: Arc<AtomicBool>,
}

impl UserTool {
    pub fn new(home: &Path, preamble_dirty: Arc<AtomicBool>) -> Self {
        Self {
            path: home.join("memory/user.md"),
            description: DESCRIPTION.to_string(),
            preamble_dirty,
        }
    }
}

impl Tool for UserTool {
    const NAME: &'static str = "user";

    type Error = ToolError;
    type Args = ToolArgs;
    type Output = ToolResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "user".to_string(),
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

    fn test_tool() -> (TempDir, UserTool) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = UserTool::new(&home, dirty);
        (tmp, tool)
    }

    fn test_tool_with_dirty() -> (TempDir, UserTool, Arc<AtomicBool>) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = UserTool::new(&home, Arc::clone(&dirty));
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

    #[tokio::test]
    async fn read_existierende_datei() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/user.md"), "Bruce ist cool.").unwrap();

        let result = tool.call(args("read", "")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Bruce ist cool.");
    }

    #[tokio::test]
    async fn read_nicht_vorhanden() {
        let (_tmp, tool) = test_tool();
        let result = tool.call(args("read", "")).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn write_neue_datei() {
        let (tmp, tool) = test_tool();
        tool.call(args("write", "Neuer Inhalt")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/user.md")).unwrap();
        assert_eq!(content, "Neuer Inhalt");
    }

    #[tokio::test]
    async fn write_setzt_dirty_flag() {
        let (_tmp, tool, dirty) = test_tool_with_dirty();
        tool.call(args("write", "Inhalt")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn edit_ersetzt_abschnitt() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/user.md"), "Bruce mag Rust und Python.").unwrap();

        tool.call(edit_args("Python", "Go")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/user.md")).unwrap();
        assert_eq!(content, "Bruce mag Rust und Go.");
    }

    #[tokio::test]
    async fn edit_old_content_nicht_gefunden() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/user.md"), "Inhalt").unwrap();

        let result = tool.call(edit_args("gibts nicht", "egal")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn append_an_bestehende_datei() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/user.md"), "Zeile 1").unwrap();

        tool.call(args("append", "Zeile 2")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/user.md")).unwrap();
        assert_eq!(content, "Zeile 1\nZeile 2");
    }

    #[tokio::test]
    async fn append_an_nicht_existierende_datei() {
        let (tmp, tool) = test_tool();
        tool.call(args("append", "Erster Inhalt")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/user.md")).unwrap();
        assert_eq!(content, "Erster Inhalt");
    }

    #[tokio::test]
    async fn beschreibung() {
        let (_tmp, tool) = test_tool();
        let def = tool.definition(String::new()).await;
        assert!(def.description.contains("Wissen ueber deinen Menschen"));
    }
}
