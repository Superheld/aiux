// SoulTool: Identitaet und Persoenlichkeit des Agents.
//
// Ziel: memory/soul.md
// Der Agent kann sein Selbstbild, seine Werte und seinen
// Kommunikationsstil lesen und aktualisieren.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use rig::completion::ToolDefinition;
use rig::tool::Tool;

use super::{execute_single_file, load_description, ToolArgs, ToolError, ToolResult};

const DEFAULT_DESCRIPTION: &str = "Aktualisiere deine Identitaet und Persoenlichkeit. \
    Nutze dieses Tool wenn du merkst dass sich deine Werte, dein Kommunikationsstil \
    oder dein Selbstbild veraendert haben. Die soul.md definiert wer du bist.";

pub struct SoulTool {
    path: PathBuf,
    description: String,
    preamble_dirty: Arc<AtomicBool>,
}

impl SoulTool {
    pub fn new(home: &Path, preamble_dirty: Arc<AtomicBool>) -> Self {
        let description = load_description(home, "tool-soul.md", DEFAULT_DESCRIPTION);
        Self {
            path: home.join("memory/soul.md"),
            description,
            preamble_dirty,
        }
    }
}

impl Tool for SoulTool {
    const NAME: &'static str = "soul";

    type Error = ToolError;
    type Args = ToolArgs;
    type Output = ToolResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "soul".to_string(),
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

    fn test_tool() -> (TempDir, SoulTool) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = SoulTool::new(&home, dirty);
        (tmp, tool)
    }

    fn test_tool_with_dirty() -> (TempDir, SoulTool, Arc<AtomicBool>) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        let dirty = Arc::new(AtomicBool::new(false));
        let tool = SoulTool::new(&home, Arc::clone(&dirty));
        (tmp, tool, dirty)
    }

    fn args(action: &str, content: &str) -> ToolArgs {
        ToolArgs {
            action: action.to_string(),
            content: content.to_string(),
            old_content: String::new(),
            new_content: String::new(),
            key: String::new(),
        }
    }

    fn edit_args(old: &str, new: &str) -> ToolArgs {
        ToolArgs {
            action: "edit".to_string(),
            content: String::new(),
            old_content: old.to_string(),
            new_content: new.to_string(),
            key: String::new(),
        }
    }

    // ==========================================================
    // read
    // ==========================================================

    #[tokio::test]
    async fn read_existierende_datei() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/soul.md"), "Ich bin AIUX.").unwrap();

        let result = tool.call(args("read", "")).await.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Ich bin AIUX.");
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

        let result = tool.call(args("write", "Neue Seele")).await.unwrap();
        assert!(result.success);

        let content = fs::read_to_string(tmp.path().join("memory/soul.md")).unwrap();
        assert_eq!(content, "Neue Seele");
    }

    #[tokio::test]
    async fn write_ueberschreibt() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/soul.md"), "Alt").unwrap();

        tool.call(args("write", "Neu")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/soul.md")).unwrap();
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
        fs::write(tmp.path().join("memory/soul.md"), "Ich bin ruhig und freundlich.").unwrap();

        let result = tool.call(edit_args("ruhig", "laut")).await.unwrap();
        assert!(result.success);

        let content = fs::read_to_string(tmp.path().join("memory/soul.md")).unwrap();
        assert_eq!(content, "Ich bin laut und freundlich.");
    }

    #[tokio::test]
    async fn edit_old_content_nicht_gefunden() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/soul.md"), "Inhalt").unwrap();

        let result = tool.call(edit_args("gibts nicht", "egal")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn edit_setzt_dirty_flag() {
        let (tmp, tool, dirty) = test_tool_with_dirty();
        fs::write(tmp.path().join("memory/soul.md"), "Alt").unwrap();

        tool.call(edit_args("Alt", "Neu")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    // ==========================================================
    // append
    // ==========================================================

    #[tokio::test]
    async fn append_an_bestehende_datei() {
        let (tmp, tool) = test_tool();
        fs::write(tmp.path().join("memory/soul.md"), "Zeile 1").unwrap();

        tool.call(args("append", "Zeile 2")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/soul.md")).unwrap();
        assert_eq!(content, "Zeile 1\nZeile 2");
    }

    #[tokio::test]
    async fn append_an_nicht_existierende_datei() {
        let (tmp, tool) = test_tool();

        tool.call(args("append", "Erster Inhalt")).await.unwrap();

        let content = fs::read_to_string(tmp.path().join("memory/soul.md")).unwrap();
        assert_eq!(content, "Erster Inhalt");
    }

    #[tokio::test]
    async fn append_setzt_dirty_flag() {
        let (_tmp, tool, dirty) = test_tool_with_dirty();

        tool.call(args("append", "Inhalt")).await.unwrap();
        assert!(dirty.load(Ordering::Relaxed));
    }

    // ==========================================================
    // Beschreibung aus Datei laden
    // ==========================================================

    #[tokio::test]
    async fn beschreibung_aus_datei() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory")).unwrap();
        fs::create_dir_all(home.join(".system")).unwrap();
        fs::write(home.join(".system/tool-soul.md"), "Custom Soul Beschreibung").unwrap();

        let dirty = Arc::new(AtomicBool::new(false));
        let tool = SoulTool::new(&home, dirty);
        let def = tool.definition(String::new()).await;
        assert_eq!(def.description, "Custom Soul Beschreibung");
    }

    #[tokio::test]
    async fn beschreibung_fallback() {
        let (_tmp, tool) = test_tool();
        let def = tool.definition(String::new()).await;
        assert!(def.description.contains("Identitaet"));
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
