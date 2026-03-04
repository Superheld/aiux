// SchedulerTool: Timer und Cron-Jobs setzen, listen, loeschen.
//
// Der Cortex nutzt dieses Tool um zeitgesteuerte Events zu planen.
// Die Eintraege werden im SharedScheduler gespeichert, den der
// Brainstem in seiner run()-Loop prueft und bei Faelligkeit feuert.

use std::str::FromStr;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::brainstem::{
    SharedScheduler, ScheduleEntry, ScheduleKind,
    next_cron_duration, parse_delay,
};

/// Ergebnis des SchedulerTools.
#[derive(Serialize)]
pub struct SchedulerResult {
    pub success: bool,
    pub message: String,
}

/// Fehler-Typ fuer das SchedulerTool.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct SchedulerError(pub String);

/// Argumente fuer das SchedulerTool.
#[derive(Deserialize, JsonSchema)]
pub struct SchedulerArgs {
    /// Aktion: "set", "cron", "list", "cancel"
    pub action: String,
    /// Relative Verzoegerung: "30m", "2h", "1d" (fuer "set")
    #[serde(default)]
    pub delay: Option<String>,
    /// Cron-Ausdruck: "0 0 * * * * *" (fuer "cron")
    #[serde(default)]
    pub expr: Option<String>,
    /// Beschreibung des Timers
    #[serde(default)]
    pub label: Option<String>,
    /// ID zum Loeschen (fuer "cancel")
    #[serde(default)]
    pub id: Option<String>,
}

pub struct SchedulerTool {
    scheduler: SharedScheduler,
    next_id: std::sync::atomic::AtomicU64,
}

impl SchedulerTool {
    pub fn new(scheduler: SharedScheduler) -> Self {
        Self {
            scheduler,
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    fn gen_id(&self) -> String {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("t{}", id)
    }
}

impl Tool for SchedulerTool {
    const NAME: &'static str = "scheduler";

    type Error = SchedulerError;
    type Args = SchedulerArgs;
    type Output = SchedulerResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "scheduler".to_string(),
            description: "Timer und Cron-Jobs verwalten. \
                Setze einmalige Timer (set), wiederkehrende Cron-Jobs (cron), \
                zeige alle aktiven Eintraege (list) oder loesche einen (cancel).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["set", "cron", "list", "cancel"],
                        "description": "Aktion: set (einmaliger Timer), cron (wiederkehrend), list (anzeigen), cancel (loeschen)"
                    },
                    "delay": {
                        "type": "string",
                        "description": "Relative Verzoegerung: '30m', '2h', '1d' (nur bei set)"
                    },
                    "expr": {
                        "type": "string",
                        "description": "Cron-Ausdruck mit 7 Feldern: Sek Min Std Tag Mon Wochentag Jahr (nur bei cron)"
                    },
                    "label": {
                        "type": "string",
                        "description": "Beschreibung des Timers"
                    },
                    "id": {
                        "type": "string",
                        "description": "ID des zu loeschenden Eintrags (nur bei cancel)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.action.as_str() {
            "set" => {
                let delay_str = args.delay
                    .ok_or_else(|| SchedulerError("'delay' ist erforderlich bei 'set'".into()))?;
                let label = args.label.unwrap_or_else(|| "timer".into());
                let duration = parse_delay(&delay_str)
                    .map_err(|e| SchedulerError(e))?;

                let id = self.gen_id();
                let entry = ScheduleEntry {
                    id: id.clone(),
                    label: label.clone(),
                    kind: ScheduleKind::Once {
                        fire_at: Instant::now() + duration,
                    },
                };

                self.scheduler.lock().unwrap().push(entry);

                Ok(SchedulerResult {
                    success: true,
                    message: format!("Timer '{}' gesetzt (id: {}, in {})", label, id, delay_str),
                })
            }
            "cron" => {
                let expr = args.expr
                    .ok_or_else(|| SchedulerError("'expr' ist erforderlich bei 'cron'".into()))?;
                let label = args.label.unwrap_or_else(|| "cron".into());

                let schedule = cron::Schedule::from_str(&expr)
                    .map_err(|e| SchedulerError(format!("Ungueltiger Cron-Ausdruck: {}", e)))?;

                let dur = next_cron_duration(&schedule);
                let id = self.gen_id();
                let entry = ScheduleEntry {
                    id: id.clone(),
                    label: label.clone(),
                    kind: ScheduleKind::Cron {
                        schedule,
                        next_fire: Instant::now() + dur,
                    },
                };

                self.scheduler.lock().unwrap().push(entry);

                Ok(SchedulerResult {
                    success: true,
                    message: format!("Cron-Job '{}' gesetzt (id: {}, expr: {})", label, id, expr),
                })
            }
            "list" => {
                let entries = self.scheduler.lock().unwrap();
                if entries.is_empty() {
                    return Ok(SchedulerResult {
                        success: true,
                        message: "Keine aktiven Timer oder Cron-Jobs.".into(),
                    });
                }

                let now = Instant::now();
                let mut lines = Vec::new();
                for entry in entries.iter() {
                    let kind_str = match &entry.kind {
                        ScheduleKind::Once { fire_at } => {
                            if *fire_at > now {
                                let remaining = *fire_at - now;
                                format!("once (in {}s)", remaining.as_secs())
                            } else {
                                "once (faellig)".into()
                            }
                        }
                        ScheduleKind::Cron { next_fire, .. } => {
                            if *next_fire > now {
                                let remaining = *next_fire - now;
                                format!("cron (naechster in {}s)", remaining.as_secs())
                            } else {
                                "cron (faellig)".into()
                            }
                        }
                    };
                    lines.push(format!("  [{}] {} ({})", entry.id, entry.label, kind_str));
                }

                Ok(SchedulerResult {
                    success: true,
                    message: format!("Aktive Eintraege:\n{}", lines.join("\n")),
                })
            }
            "cancel" => {
                let id = args.id
                    .ok_or_else(|| SchedulerError("'id' ist erforderlich bei 'cancel'".into()))?;

                let mut entries = self.scheduler.lock().unwrap();
                let before = entries.len();
                entries.retain(|e| e.id != id);

                if entries.len() < before {
                    Ok(SchedulerResult {
                        success: true,
                        message: format!("Eintrag '{}' geloescht.", id),
                    })
                } else {
                    Ok(SchedulerResult {
                        success: false,
                        message: format!("Kein Eintrag mit ID '{}' gefunden.", id),
                    })
                }
            }
            other => Err(SchedulerError(format!(
                "Unbekannte Aktion '{}'. Erlaubt: set, cron, list, cancel", other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn test_tool() -> SchedulerTool {
        let scheduler = Arc::new(Mutex::new(Vec::new()));
        SchedulerTool::new(scheduler)
    }

    #[tokio::test]
    async fn set_timer() {
        let tool = test_tool();
        let result = tool.call(SchedulerArgs {
            action: "set".into(),
            delay: Some("30m".into()),
            expr: None,
            label: Some("test".into()),
            id: None,
        }).await.unwrap();

        assert!(result.success);
        assert!(result.message.contains("test"));
        assert_eq!(tool.scheduler.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn set_ohne_delay() {
        let tool = test_tool();
        let result = tool.call(SchedulerArgs {
            action: "set".into(),
            delay: None,
            expr: None,
            label: Some("test".into()),
            id: None,
        }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cron_job() {
        let tool = test_tool();
        let result = tool.call(SchedulerArgs {
            action: "cron".into(),
            delay: None,
            expr: Some("0 0 * * * * *".into()),
            label: Some("stuendlich".into()),
            id: None,
        }).await.unwrap();

        assert!(result.success);
        assert!(result.message.contains("stuendlich"));
        assert_eq!(tool.scheduler.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn cron_ungueltig() {
        let tool = test_tool();
        let result = tool.call(SchedulerArgs {
            action: "cron".into(),
            delay: None,
            expr: Some("ungueltig".into()),
            label: None,
            id: None,
        }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_leer() {
        let tool = test_tool();
        let result = tool.call(SchedulerArgs {
            action: "list".into(),
            delay: None,
            expr: None,
            label: None,
            id: None,
        }).await.unwrap();

        assert!(result.success);
        assert!(result.message.contains("Keine aktiven"));
    }

    #[tokio::test]
    async fn list_mit_eintraegen() {
        let tool = test_tool();
        // Timer setzen
        tool.call(SchedulerArgs {
            action: "set".into(),
            delay: Some("1h".into()),
            expr: None,
            label: Some("erinnerung".into()),
            id: None,
        }).await.unwrap();

        let result = tool.call(SchedulerArgs {
            action: "list".into(),
            delay: None,
            expr: None,
            label: None,
            id: None,
        }).await.unwrap();

        assert!(result.success);
        assert!(result.message.contains("erinnerung"));
    }

    #[tokio::test]
    async fn cancel_existierend() {
        let tool = test_tool();
        // Timer setzen
        tool.call(SchedulerArgs {
            action: "set".into(),
            delay: Some("1h".into()),
            expr: None,
            label: Some("test".into()),
            id: None,
        }).await.unwrap();

        // ID aus der Message extrahieren
        let id = tool.scheduler.lock().unwrap()[0].id.clone();

        let result = tool.call(SchedulerArgs {
            action: "cancel".into(),
            delay: None,
            expr: None,
            label: None,
            id: Some(id),
        }).await.unwrap();

        assert!(result.success);
        assert!(result.message.contains("geloescht"));
        assert_eq!(tool.scheduler.lock().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn cancel_nicht_vorhanden() {
        let tool = test_tool();
        let result = tool.call(SchedulerArgs {
            action: "cancel".into(),
            delay: None,
            expr: None,
            label: None,
            id: Some("gibts-nicht".into()),
        }).await.unwrap();

        assert!(!result.success);
    }

    #[tokio::test]
    async fn unbekannte_aktion() {
        let tool = test_tool();
        let result = tool.call(SchedulerArgs {
            action: "delete".into(),
            delay: None,
            expr: None,
            label: None,
            id: None,
        }).await;
        assert!(result.is_err());
    }
}
