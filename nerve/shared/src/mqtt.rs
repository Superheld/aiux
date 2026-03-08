// MQTT-Verbindung und Message-Hilfsfunktionen fuer Nerves.
//
// Jeder Nerve nutzt diese Funktionen um sich mit dem Broker zu verbinden
// und Messages im AIUX-Schema zu publishen.

use chrono::Utc;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json::json;

/// MQTT-Verbindung herstellen.
/// Gibt Client und EventLoop zurueck (EventLoop muss gepumpt werden).
pub fn connect(client_id: &str, host: &str, port: u16) -> (AsyncClient, rumqttc::EventLoop) {
    let mut opts = MqttOptions::new(client_id, host, port);
    opts.set_keep_alive(std::time::Duration::from_secs(30));
    AsyncClient::new(opts, 64)
}

/// Standard-Message im AIUX-Schema bauen.
pub fn build_message(source: &str, event: &str, data: serde_json::Value) -> String {
    json!({
        "ts": Utc::now().to_rfc3339(),
        "source": source,
        "event": event,
        "data": data,
    })
    .to_string()
}

/// Message auf ein Topic publishen.
pub async fn publish(client: &AsyncClient, topic: &str, payload: &str) -> Result<(), String> {
    client
        .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
        .await
        .map_err(|e| format!("MQTT publish fehlgeschlagen: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_pflichtfelder() {
        let msg = build_message("nerve/test", "ping", json!({"x": 1}));
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();

        assert!(parsed["ts"].is_string());
        assert_eq!(parsed["source"], "nerve/test");
        assert_eq!(parsed["event"], "ping");
        assert_eq!(parsed["data"]["x"], 1);
    }
}
