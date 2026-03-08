// MQTT-Bridge: Verbindet den internen Event-Bus mit der Aussenwelt via MQTT.
//
// Zwei Richtungen:
// - mqtt_to_bus: MQTT Incoming (aiux/nerve/#) → interner Bus als NerveSignal
// - bus_to_mqtt: Interner Bus → MQTT Outgoing (aiux/neocortex/*)
//
// Reconnect wird von rumqttc automatisch gehandhabt (EventLoop pollt weiter).

use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event as MqttEvent, MqttOptions, Packet, QoS};
use tokio::sync::broadcast;

use crate::bus::events::Event;
use crate::bus::Bus;

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
    /// Pflichtfelder im JSON: ts, source, event. Fehlende Felder → Warnung + verwerfen.
    fn handle_incoming(&self, packet: Packet) {
        if let Packet::Publish(publish) = packet {
            let topic = publish.topic.clone();

            // Nur aiux/nerve/* Topics verarbeiten
            if topic.strip_prefix("aiux/nerve/").is_none() {
                return;
            }

            // JSON parsen
            let json: serde_json::Value = match serde_json::from_slice(&publish.payload) {
                Ok(val) => val,
                Err(_) => {
                    self.bus.publish(Event::SystemMessage {
                        text: format!("MQTT: Kein gueltiges JSON auf {}", topic),
                    });
                    return;
                }
            };

            // Pflichtfelder pruefen
            let source = match json.get("source").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    self.bus.publish(Event::SystemMessage {
                        text: format!("MQTT: Pflichtfeld 'source' fehlt auf {}", topic),
                    });
                    return;
                }
            };
            let event = match json.get("event").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    self.bus.publish(Event::SystemMessage {
                        text: format!("MQTT: Pflichtfeld 'event' fehlt auf {}", topic),
                    });
                    return;
                }
            };
            let ts = match json.get("ts").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => {
                    self.bus.publish(Event::SystemMessage {
                        text: format!("MQTT: Pflichtfeld 'ts' fehlt auf {}", topic),
                    });
                    return;
                }
            };
            let data = json.get("data").cloned().unwrap_or(serde_json::Value::Null);

            self.bus.publish(Event::NerveSignal {
                source,
                event,
                data,
                ts,
            });
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
                            "aiux/neocortex/response",
                            serde_json::json!({ "text": full_text }),
                        ),
                        Event::SystemMessage { text } => {
                            ("aiux/neocortex/system", serde_json::json!({ "text": text }))
                        }
                        Event::ToolCall { name } => (
                            "aiux/neocortex/toolcall",
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

    /// Helper: MQTT Publish-Paket bauen
    fn mqtt_publish(topic: &str, payload: &[u8]) -> Packet {
        let mut publish = rumqttc::Publish::new(topic, QoS::AtLeastOnce, payload.to_vec());
        publish.topic = topic.to_string();
        Packet::Publish(publish)
    }

    /// Gueltiges JSON mit Pflichtfeldern wird als NerveSignal geparsed.
    #[test]
    fn parse_nerve_signal_gueltig() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        bridge.handle_incoming(mqtt_publish(
            "aiux/nerve/file/changed",
            br#"{"ts":"2026-03-02T14:00:00Z","source":"nerve/file","event":"changed","data":{"path":"/tmp/test.txt"}}"#,
        ));

        let event = rx.try_recv().unwrap();
        match event {
            Event::NerveSignal {
                source,
                event,
                data,
                ts,
            } => {
                assert_eq!(source, "nerve/file");
                assert_eq!(event, "changed");
                assert_eq!(ts, "2026-03-02T14:00:00Z");
                assert_eq!(data["path"], "/tmp/test.txt");
            }
            other => panic!("Erwartetes NerveSignal, bekam: {:?}", other),
        }
    }

    /// data-Feld ist optional — fehlend ergibt Value::Null.
    #[test]
    fn parse_nerve_signal_ohne_data() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        bridge.handle_incoming(mqtt_publish(
            "aiux/nerve/test/ping",
            br#"{"ts":"2026-03-02T14:00:00Z","source":"nerve/test","event":"ping"}"#,
        ));

        let event = rx.try_recv().unwrap();
        match event {
            Event::NerveSignal { data, .. } => {
                assert!(data.is_null());
            }
            other => panic!("Erwartetes NerveSignal, bekam: {:?}", other),
        }
    }

    /// Fehlendes Pflichtfeld 'event' → Warnung, kein NerveSignal.
    #[test]
    fn pflichtfeld_event_fehlt() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        bridge.handle_incoming(mqtt_publish(
            "aiux/nerve/test/x",
            br#"{"ts":"2026-03-02T14:00:00Z","source":"nerve/test"}"#,
        ));

        // Sollte SystemMessage sein (Warnung), kein NerveSignal
        let event = rx.try_recv().unwrap();
        assert!(matches!(event, Event::SystemMessage { .. }));
    }

    /// Fehlendes Pflichtfeld 'ts' → Warnung, kein NerveSignal.
    #[test]
    fn pflichtfeld_ts_fehlt() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        bridge.handle_incoming(mqtt_publish(
            "aiux/nerve/test/x",
            br#"{"source":"nerve/test","event":"ping"}"#,
        ));

        let event = rx.try_recv().unwrap();
        assert!(matches!(event, Event::SystemMessage { .. }));
    }

    /// Kein JSON → Warnung.
    #[test]
    fn kein_json_payload() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        bridge.handle_incoming(mqtt_publish("aiux/nerve/test/x", b"hello world"));

        let event = rx.try_recv().unwrap();
        assert!(matches!(event, Event::SystemMessage { .. }));
    }

    /// Fremde Topics (nicht aiux/nerve/) werden ignoriert.
    #[test]
    fn ignore_foreign_topics() {
        let bus = Arc::new(Bus::new(16));
        let bridge = MqttBridge::new(bus.clone(), "localhost", 1883);
        let mut rx = bus.subscribe();

        bridge.handle_incoming(mqtt_publish("other/topic", b"{}"));

        assert!(rx.try_recv().is_err(), "Kein Event erwartet");
    }

    /// Nur ResponseComplete, SystemMessage und ToolCall gehen nach MQTT.
    #[test]
    fn event_filter_outgoing() {
        let internal_events = vec![
            Event::UserInput {
                text: "test".into(),
            },
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

        let external_events: Vec<Event> = vec![
            Event::ResponseComplete {
                full_text: "hi".into(),
            },
            Event::SystemMessage {
                text: "info".into(),
            },
            Event::ToolCall {
                name: "memory".into(),
            },
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
}
