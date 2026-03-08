// Tools: Bewusste Handlungen des Agents.
//
// Drei spezialisierte Memory-Tools:
// - SoulTool: Identitaet und Persoenlichkeit (soul.md)
// - UserTool: Wissen ueber den Menschen (user.md)
// - MemoryTool: Notizen und Gelerntes (notes.md)

pub mod memory;
pub mod scheduler;
pub mod shell;
pub mod soul;
pub mod user;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Ergebnis aller Tool-Aktionen.
#[derive(Serialize)]
pub struct ToolResult {
    pub success: bool,
    pub message: String,
}

/// Fehler-Typ fuer alle Tools.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ToolError(pub String);

/// Gemeinsame Argumente fuer alle drei Tools.
#[derive(Deserialize, JsonSchema)]
pub struct ToolArgs {
    /// Aktion: "read", "write", "edit", "append"
    pub action: String,
    /// Inhalt (bei write: kompletter neuer Inhalt, bei append: anzufuegender Text)
    #[serde(default)]
    pub content: String,
    /// Alter Text der ersetzt werden soll (nur bei edit)
    #[serde(default)]
    pub old_content: String,
    /// Neuer Text der den alten ersetzt (nur bei edit)
    #[serde(default)]
    pub new_content: String,
}

/// Fuehrt eine Aktion auf einer einzelnen Datei aus (fuer soul und user).
pub fn execute_single_file(
    path: &Path,
    args: &ToolArgs,
    preamble_dirty: &Arc<AtomicBool>,
) -> Result<ToolResult, ToolError> {
    match args.action.as_str() {
        "read" => match fs::read_to_string(path) {
            Ok(content) => Ok(ToolResult {
                success: true,
                message: content,
            }),
            Err(_) => Ok(ToolResult {
                success: false,
                message: "Datei nicht gefunden.".into(),
            }),
        },
        "write" => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ToolError(format!("Verzeichnis erstellen: {}", e)))?;
            }
            fs::write(path, &args.content)
                .map_err(|e| ToolError(format!("Schreibfehler: {}", e)))?;
            preamble_dirty.store(true, Ordering::Relaxed);
            Ok(ToolResult {
                success: true,
                message: "Gespeichert.".into(),
            })
        }
        "edit" => {
            if args.old_content.is_empty() {
                return Err(ToolError("old_content ist erforderlich bei edit".into()));
            }
            let content =
                fs::read_to_string(path).map_err(|e| ToolError(format!("Lesefehler: {}", e)))?;
            if !content.contains(&args.old_content) {
                return Err(ToolError("old_content nicht gefunden.".into()));
            }
            let new_content = content.replace(&args.old_content, &args.new_content);
            fs::write(path, &new_content)
                .map_err(|e| ToolError(format!("Schreibfehler: {}", e)))?;
            preamble_dirty.store(true, Ordering::Relaxed);
            Ok(ToolResult {
                success: true,
                message: "Bearbeitet.".into(),
            })
        }
        "append" => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ToolError(format!("Verzeichnis erstellen: {}", e)))?;
            }
            let existing = fs::read_to_string(path).unwrap_or_default();
            let new_content = if existing.is_empty() {
                args.content.clone()
            } else {
                format!("{}\n{}", existing, args.content)
            };
            fs::write(path, &new_content)
                .map_err(|e| ToolError(format!("Schreibfehler: {}", e)))?;
            preamble_dirty.store(true, Ordering::Relaxed);
            Ok(ToolResult {
                success: true,
                message: "Angefuegt.".into(),
            })
        }
        other => Err(ToolError(format!(
            "Unbekannte Aktion '{}'. Erlaubt: read, write, edit, append",
            other
        ))),
    }
}
