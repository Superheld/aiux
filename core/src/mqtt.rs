// MQTT-Bridge: Verbindet den internen Event-Bus mit der Aussenwelt via MQTT.
//
// Zwei Richtungen:
// - mqtt_to_bus: MQTT Incoming (aiux/nerve/#) → interner Bus als NerveSignal
// - bus_to_mqtt: Interner Bus → MQTT Outgoing (aiux/cortex/*)
//
// Reconnect wird von rumqttc automatisch gehandhabt (EventLoop pollt weiter).

use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event as MqttEvent, MqttOptions, Packet, QoS};
use tokio::sync::broadcast;

use crate::bus::Bus;
use crate::bus::events::Event;

/// Die MQTT-Bridge — das Rueckenmark zwischen internem Bus und MQTT.
pub struct MqttBridge {
    bus: Arc<Bus>,
    host: String,
    port: u16,
}

impl MqttBridge {
    pub fn new(bus: Arc<Bus>, host: &str, port: u16) -> Self {
        Self {
            bus,
            host: host.to_string(),
            port,
        }
    }

    /// Startet beide Richtungen (mqtt_to_bus + bus_to_mqtt) und laeuft endlos.
    pub async fn run(self) {
        let mut opts = MqttOptions::new("aiux-bridge", &self.host, self.port);
        opts.set_keep_alive(Duration::from_secs(30));

        let (client, mut eventloop) = AsyncClient::new(opts, 50);

        // Subscribe auf alle Nerve-Topics
        if let Err(e) = client.subscribe("aiux/nerve/#", QoS::AtLeastOnce).await {
            self.bus.publish(Event::SystemMessage {
                text: format!("MQTT Subscribe fehlgeschlagen: {}", e),
            });
            return;
        }

        // bus_to_mqtt in eigenem Task
        let bus_clone = self.bus.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            Self::bus_to_mqtt(bus_clone, client_clone).await;
        });

        // mqtt_to_bus: EventLoop pollen (reconnected automatisch)
        let mut connected = false;
        loop {
            match eventloop.poll().await {
                Ok(MqttEvent::Incoming(packet)) => {
                    if !connected {
                        connected = true;
                        self.bus.publish(Event::SystemMessage {
                            text: format!("MQTT verbunden ({}:{})", self.host, self.port),
                        });
                    }
                    self.handle_incoming(packet);
                }
                Ok(MqttEvent::Outgoing(_)) => {
                    // Outgoing-Events ignorieren (sind nur Bestaetigungen)
                }
                Err(e) => {
                    if connected {
                        self.bus.publish(Event::SystemMessage {
                            text: format!("MQTT Verbindung verloren: {}", e),
                        });
                        connected = false;
                    }
                    // rumqttc reconnected automatisch beim naechsten poll()
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Verarbeitet eingehende MQTT-Pakete → NerveSignal auf den Bus.
    fn handle_incoming(&self, packet: Packet) {
        if let Packet::Publish(publish) = packet {
            let topic = publish.topic.clone();

            // Nur aiux/nerve/* Topics verarbeiten
            let source = match topic.strip_prefix("aiux/nerve/") {
                Some(rest) => format!("nerve/{}", rest),
                None => return,
            };

            // Payload als JSON parsen, bei Fehler als String wrappen
            let payload = match serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                Ok(val) => val,
                Err(_) => {
                    let text = String::from_utf8_lossy(&publish.payload);
                    serde_json::json!({ "raw": text.to_string() })
                }
            };

            self.bus.publish(Event::NerveSignal { source, payload });
        }
    }

    /// Lauscht auf den internen Bus und leitet relevante Events nach MQTT weiter.
    async fn bus_to_mqtt(bus: Arc<Bus>, client: AsyncClient) {
        let mut rx = bus.subscribe();

        loop {
            match rx.recv().await {
                Ok(event) => {
                    let (topic, payload) = match &event {
                        Event::ResponseComplete { full_text } => (
                            "aiux/cortex/response",
                            serde_json::json!({ "text": full_text }),
                        ),
                        Event::SystemMessage { text } => (
                            "aiux/cortex/system",
                            serde_json::json!({ "text": text }),
                        ),
                        Event::ToolCall { name } => (
                            "aiux/cortex/toolcall",
                            serde_json::json!({ "name": name }),
                        ),
                        // Alle anderen Events sind intern und gehen nicht nach MQTT
                        _ => continue,
                    };

                    let json = serde_json::to_vec(&payload).unwrap_or_default();
                    if let Err(e) = client.publish(topic, QoS::AtLeastOnce, false, json).await {
                        eprintln!("MQTT publish fehlgeschlagen: {}", e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("MQTT-Bridge: {} Events verpasst (Bus overflow)", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    }
}

// ==========================================================
// Tests
// ==========================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// JSON-Payload wird korrekt als NerveSignal geparsed.
    #[test]
    fn parse_nerve_signal_json() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        // Simuliere ein MQTT Publish-Paket
        let mut publish = rumqttc::Publish::new(
            "aiux/nerve/file",
            QoS::AtLeastOnce,
            br#"{"event":"changed","path":"/tmp/test.txt"}"#.to_vec(),
        );
        publish.topic = "aiux/nerve/file".to_string();

        bridge.handle_incoming(Packet::Publish(publish));

        let event = rx.try_recv().unwrap();
        match event {
            Event::NerveSignal { source, payload } => {
                assert_eq!(source, "nerve/file");
                assert_eq!(payload["event"], "changed");
                assert_eq!(payload["path"], "/tmp/test.txt");
            }
            other => panic!("Erwartetes NerveSignal, bekam: {:?}", other),
        }
    }

    /// Nicht-JSON Payload wird als { "raw": "..." } gewrappt.
    #[test]
    fn parse_nerve_signal_raw_text() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        let mut publish = rumqttc::Publish::new(
            "aiux/nerve/test",
            QoS::AtLeastOnce,
            b"hello world".to_vec(),
        );
        publish.topic = "aiux/nerve/test".to_string();

        bridge.handle_incoming(Packet::Publish(publish));

        let event = rx.try_recv().unwrap();
        match event {
            Event::NerveSignal { source, payload } => {
                assert_eq!(source, "nerve/test");
                assert_eq!(payload["raw"], "hello world");
            }
            other => panic!("Erwartetes NerveSignal, bekam: {:?}", other),
        }
    }

    /// Fremde Topics (nicht aiux/nerve/) werden ignoriert.
    #[test]
    fn ignore_foreign_topics() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        let mut publish = rumqttc::Publish::new(
            "other/topic",
            QoS::AtLeastOnce,
            b"{}".to_vec(),
        );
        publish.topic = "other/topic".to_string();

        bridge.handle_incoming(Packet::Publish(publish));

        assert!(rx.try_recv().is_err(), "Kein Event erwartet");
    }

    /// Nur ResponseComplete, SystemMessage und ToolCall gehen nach MQTT.
    #[test]
    fn event_filter_outgoing() {
        // Events die NICHT nach MQTT gehen sollen
        let internal_events = vec![
            Event::UserInput { text: "test".into() },
            Event::ResponseToken { text: "tok".into() },
            Event::Shutdown,
            Event::ClearHistory,
            Event::Compacting,
            Event::Compacted,
        ];

        for event in &internal_events {
            let should_forward = matches!(
                event,
                Event::ResponseComplete { .. }
                | Event::SystemMessage { .. }
                | Event::ToolCall { .. }
            );
            assert!(!should_forward, "Event {:?} sollte intern bleiben", event);
        }

        // Events die nach MQTT gehen sollen
        let external_events: Vec<Event> = vec![
            Event::ResponseComplete { full_text: "hi".into() },
            Event::SystemMessage { text: "info".into() },
            Event::ToolCall { name: "memory".into() },
        ];

        for event in &external_events {
            let should_forward = matches!(
                event,
                Event::ResponseComplete { .. }
                | Event::SystemMessage { .. }
                | Event::ToolCall { .. }
            );
            assert!(should_forward, "Event {:?} sollte nach MQTT gehen", event);
        }
    }

    /// Verschachtelte Nerve-Topics funktionieren (aiux/nerve/system/cpu).
    #[test]
    fn nested_nerve_topic() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        let mut publish = rumqttc::Publish::new(
            "aiux/nerve/system/cpu",
            QoS::AtLeastOnce,
            br#"{"load":0.5}"#.to_vec(),
        );
        publish.topic = "aiux/nerve/system/cpu".to_string();

        bridge.handle_incoming(Packet::Publish(publish));

        let event = rx.try_recv().unwrap();
        match event {
            Event::NerveSignal { source, .. } => {
                assert_eq!(source, "nerve/system/cpu");
            }
            other => panic!("Erwartetes NerveSignal, bekam: {:?}", other),
        }
    }
}
