// Self-Registration: Nerve meldet sich beim Brainstem an.
//
// Jeder Nerve ruft beim Start register() auf. Das schickt eine
// standardisierte Message auf aiux/nerve/register.

use rumqttc::AsyncClient;
use serde_json::json;

use crate::mqtt;

/// Nerve-Identitaet fuer die Registrierung.
pub struct NerveInfo {
    /// Eindeutiger Name (z.B. "system-monitor")
    pub name: String,
    /// Version (z.B. "0.1.0")
    pub version: String,
    /// Beschreibung (Text fuer den Cortex)
    pub description: String,
    /// MQTT source (z.B. "nerve/system")
    pub source: String,
    /// Topics die dieser Nerve publishen wird
    pub channels: Vec<String>,
    /// Pfad zum Nerve-Verzeichnis (relativ zu home, fuer interpret.rhai)
    pub home: Option<String>,
}

/// Nerve beim Brainstem registrieren.
/// Schickt eine Register-Message auf aiux/nerve/register.
pub async fn register(client: &AsyncClient, info: &NerveInfo) -> Result<(), String> {
    let data = json!({
        "name": info.name,
        "version": info.version,
        "description": info.description,
        "channels": info.channels,
        "home": info.home,
    });

    let payload = mqtt::build_message(&info.source, "register", data);
    mqtt::publish(client, "aiux/nerve/register", &payload).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nerve_info_erstellen() {
        let info = NerveInfo {
            name: "system-monitor".into(),
            version: "0.1.0".into(),
            description: "CPU, RAM, Disk".into(),
            source: "nerve/system".into(),
            channels: vec!["aiux/nerve/system/stats".into()],
            home: Some("nerves/system-monitor".into()),
        };

        assert_eq!(info.name, "system-monitor");
        assert_eq!(info.channels.len(), 1);
    }
}
