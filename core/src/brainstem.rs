// Brainstem: Reflexe und autonome Verarbeitung.
//
// Lauscht auf dem internen Bus nach NerveSignals, prueft sie gegen
// die Registry und fuehrt interpret.rhai Scripts in einer Sandbox aus.
// Das Script entscheidet per Rueckgabewert ob/wohin weitergeleitet wird.
//
// Beim Start scannt der Brainstem home/nerves/*/ und laedt Manifeste.
// Ohne Nerves ist die Registry leer und er tut nichts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::broadcast;

use crate::bus::Bus;
use crate::bus::events::Event;

// ==========================================================
// Nerve-Manifest und Channels (aus TOML)
// ==========================================================

/// Nerve-Manifest: Wer bin ich, was starten.
#[derive(Debug, Clone, Deserialize)]
pub struct NerveManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub binary: String,
}

/// Ein einzelner Channel den ein Nerve publiziert.
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelDef {
    pub topic: String,
    pub description: String,
}

/// Alle Channels eines Nerve.
#[derive(Debug, Clone, Deserialize)]
pub struct NerveChannels {
    pub publish: Vec<ChannelDef>,
}

// ==========================================================
// Registry
// ==========================================================

/// Ein registrierter Nerve mit Manifest und Channels.
#[derive(Debug, Clone)]
pub struct NerveEntry {
    pub manifest: NerveManifest,
    pub channels: NerveChannels,
    pub path: PathBuf,
}

/// Registry aller bekannten Nerves.
#[derive(Debug)]
pub struct NerveRegistry {
    /// Key = Nerve-Name (aus manifest.toml)
    nerves: HashMap<String, NerveEntry>,
}

impl NerveRegistry {
    fn new() -> Self {
        Self { nerves: HashMap::new() }
    }

    /// Nerve registrieren. Gibt false zurueck wenn der Name schon existiert.
    fn register(&mut self, entry: NerveEntry) -> bool {
        let name = entry.manifest.name.clone();
        if self.nerves.contains_key(&name) {
            return false;
        }
        self.nerves.insert(name, entry);
        true
    }

    /// Nerve anhand des source-Strings finden.
    /// source kommt als "nerve/<name>" oder "nerve/<name>/<sub>" — wir matchen auf den Namen.
    pub fn find_by_source(&self, source: &str) -> Option<&NerveEntry> {
        // source = "nerve/file" oder "nerve/file/changed" → name = "file"
        let after_prefix = source.strip_prefix("nerve/")?;
        let name = after_prefix.split('/').next()?;

        // Suche Nerve dessen Name passt
        self.nerves.values().find(|entry| {
            // Exakter Name oder Name ist Prefix des Source-Pfads
            entry.manifest.name == name
                || entry.manifest.name.replace('-', "") == name
        })
    }

    /// Prueft ob ein MQTT-Topic fuer diesen Nerve deklariert ist.
    pub fn is_topic_declared(&self, entry: &NerveEntry, topic: &str) -> bool {
        entry.channels.publish.iter().any(|ch| ch.topic == topic)
    }

    /// Anzahl registrierter Nerves.
    pub fn len(&self) -> usize {
        self.nerves.len()
    }

    /// Alle registrierten Nerve-Namen.
    pub fn names(&self) -> Vec<&str> {
        self.nerves.keys().map(|s| s.as_str()).collect()
    }
}

// ==========================================================
// Boot-Scan
// ==========================================================

/// Scannt home/nerves/*/ und laedt Manifeste + Channels.
/// Fehlerhafte Verzeichnisse werden uebersprungen (mit Warnung auf dem Bus).
fn boot_scan(home: &Path, bus: &Bus) -> NerveRegistry {
    let mut registry = NerveRegistry::new();
    let nerves_dir = home.join("nerves");

    if !nerves_dir.exists() {
        return registry;
    }

    let entries = match std::fs::read_dir(&nerves_dir) {
        Ok(e) => e,
        Err(e) => {
            bus.publish(Event::SystemMessage {
                text: format!("Brainstem: nerves/ nicht lesbar: {}", e),
            });
            return registry;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        // manifest.toml laden
        let manifest_path = path.join("manifest.toml");
        let manifest: NerveManifest = match load_toml(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: {}/manifest.toml fehlerhaft: {}", dir_name, e),
                });
                continue;
            }
        };

        // channels.toml laden
        let channels_path = path.join("channels.toml");
        let channels: NerveChannels = match load_toml(&channels_path) {
            Ok(c) => c,
            Err(e) => {
                bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: {}/channels.toml fehlerhaft: {}", dir_name, e),
                });
                continue;
            }
        };

        let name = manifest.name.clone();
        let n_channels = channels.publish.len();

        let entry = NerveEntry { manifest, channels, path: path.clone() };
        if registry.register(entry) {
            bus.publish(Event::SystemMessage {
                text: format!("Nerve geladen: {} ({} Channel{})",
                    name, n_channels, if n_channels != 1 { "s" } else { "" }),
            });
        } else {
            bus.publish(Event::SystemMessage {
                text: format!("Brainstem: Nerve '{}' doppelt, uebersprungen", name),
            });
        }
    }

    registry
}

/// TOML-Datei laden und deserialisieren.
fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Datei nicht lesbar: {}", e))?;
    toml::from_str(&content)
        .map_err(|e| format!("TOML-Fehler: {}", e))
}

// ==========================================================
// Brainstem
// ==========================================================

/// Der Brainstem — Reflexe und autonome Verarbeitung.
pub struct Brainstem {
    bus: Arc<Bus>,
    registry: NerveRegistry,
    engine: rhai::Engine,
}

impl Brainstem {
    /// Neuer Brainstem mit Boot-Scan und rhai-Engine.
    pub fn new(bus: Arc<Bus>, home: &Path) -> Self {
        let registry = boot_scan(home, &bus);
        let mut engine = rhai::Engine::new();
        engine.set_max_operations(10_000);
        engine.set_max_call_levels(32);
        engine.set_max_string_size(64_000);
        Self { bus, registry, engine }
    }

    /// Event-Loop: lauscht auf NerveSignals und verarbeitet sie.
    pub async fn run(&self) {
        let mut rx = self.bus.subscribe();

        loop {
            match rx.recv().await {
                Ok(Event::NerveSignal { ref source, ref event, ref data, ref ts }) => {
                    self.handle_nerve_signal(source, event, data, ts);
                }
                Ok(Event::Shutdown) => break,
                Ok(_) => {} // Andere Events ignorieren
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("Brainstem: {} Events verpasst", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }

    /// NerveSignal verarbeiten: Registry-Lookup → interpret.rhai ausfuehren → weiterleiten.
    fn handle_nerve_signal(&self, source: &str, event: &str, data: &serde_json::Value, ts: &str) {
        let entry = match self.registry.find_by_source(source) {
            Some(e) => e,
            None => {
                self.bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: Unbekannter Nerve '{}'", source),
                });
                return;
            }
        };

        // interpret.rhai suchen
        let script_path = entry.path.join("interpret.rhai");
        if !script_path.exists() {
            // Kein Script → nur loggen (Fallback)
            self.bus.publish(Event::SystemMessage {
                text: format!("Brainstem: {} → {} (kein interpret.rhai)", source, event),
            });
            return;
        }

        // Script laden
        let script = match std::fs::read_to_string(&script_path) {
            Ok(s) => s,
            Err(e) => {
                self.bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: interpret.rhai nicht lesbar ({}): {}", entry.manifest.name, e),
                });
                return;
            }
        };

        // Scope mit Konstanten fuellen
        let mut scope = rhai::Scope::new();
        scope.push_constant("source", source.to_string());
        scope.push_constant("event", event.to_string());
        scope.push_constant("data", data.to_string());
        scope.push_constant("ts", ts.to_string());

        // Script ausfuehren
        let result = match self.engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &script) {
            Ok(r) => r,
            Err(e) => {
                self.bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: Script-Fehler ({}): {}", entry.manifest.name, e),
                });
                return;
            }
        };

        // Ergebnis verarbeiten
        self.process_script_result(&entry.manifest.name, result);
    }

    /// Verarbeitet das Ergebnis eines interpret.rhai Scripts.
    /// Erwartet ein rhai-Map mit forward, target, text.
    fn process_script_result(&self, nerve_name: &str, result: rhai::Dynamic) {
        let map = match result.try_cast::<rhai::Map>() {
            Some(m) => m,
            None => {
                self.bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: Script-Ergebnis ist kein Map ({})", nerve_name),
                });
                return;
            }
        };

        // forward: bool — weiterleiten?
        let forward = map.get("forward")
            .and_then(|v| v.clone().try_cast::<bool>())
            .unwrap_or(false);

        if !forward {
            return;
        }

        let target = map.get("target")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default();

        let text = map.get("text")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default();

        match target.as_str() {
            "cortex" => {
                self.bus.publish(Event::SystemMessage {
                    text: format!("Brainstem [{}]: {}", nerve_name, text),
                });
            }
            // Spaeter: "mqtt" → MQTT publish
            _ => {} // "ignore" oder unbekannt → nichts tun
        }
    }

    /// Zugriff auf die Registry (fuer Tests).
    pub fn registry(&self) -> &NerveRegistry {
        &self.registry
    }
}

// ==========================================================
// Tests
// ==========================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn test_home() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        (tmp, home)
    }

    fn write_nerve(home: &Path, name: &str, manifest: &str, channels: &str) {
        let dir = home.join("nerves").join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.toml"), manifest).unwrap();
        fs::write(dir.join("channels.toml"), channels).unwrap();
    }

    fn write_nerve_with_script(home: &Path, name: &str, manifest: &str, channels: &str, script: &str) {
        write_nerve(home, name, manifest, channels);
        let dir = home.join("nerves").join(name);
        fs::write(dir.join("interpret.rhai"), script).unwrap();
    }

    const VALID_MANIFEST: &str = r#"
name = "test-nerve"
version = "0.1.0"
description = "Test"
binary = "./test"
"#;

    const VALID_CHANNELS: &str = r#"
[[publish]]
topic = "aiux/nerve/test/ping"
description = "Test-Ping"
"#;

    // -- manifest.toml parsen --

    #[test]
    fn manifest_parsen_gueltig() {
        let m: NerveManifest = toml::from_str(VALID_MANIFEST).unwrap();
        assert_eq!(m.name, "test-nerve");
        assert_eq!(m.version, "0.1.0");
        assert_eq!(m.binary, "./test");
    }

    #[test]
    fn manifest_fehlendes_feld() {
        let result = toml::from_str::<NerveManifest>(r#"
name = "test"
version = "0.1.0"
"#);
        assert!(result.is_err());
    }

    #[test]
    fn manifest_kaputtes_toml() {
        let result = toml::from_str::<NerveManifest>("das ist {{kein toml");
        assert!(result.is_err());
    }

    // -- channels.toml parsen --

    #[test]
    fn channels_parsen_gueltig() {
        let c: NerveChannels = toml::from_str(VALID_CHANNELS).unwrap();
        assert_eq!(c.publish.len(), 1);
        assert_eq!(c.publish[0].topic, "aiux/nerve/test/ping");
    }

    #[test]
    fn channels_mehrere_topics() {
        let c: NerveChannels = toml::from_str(r#"
[[publish]]
topic = "aiux/nerve/file/changed"
description = "Aenderung"

[[publish]]
topic = "aiux/nerve/file/deleted"
description = "Geloescht"
"#).unwrap();
        assert_eq!(c.publish.len(), 2);
    }

    #[test]
    fn channels_leere_liste() {
        let result = toml::from_str::<NerveChannels>("");
        assert!(result.is_err()); // publish ist Pflicht
    }

    // -- Boot-Scan --

    #[test]
    fn boot_scan_leeres_verzeichnis() {
        let (_tmp, home) = test_home();
        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn boot_scan_kein_nerves_verzeichnis() {
        let (_tmp, home) = test_home();
        // nerves/ existiert nicht → leere Registry, kein Fehler
        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn boot_scan_ein_nerve() {
        let (_tmp, home) = test_home();
        write_nerve(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS);

        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);
        assert_eq!(registry.len(), 1);
        assert!(registry.names().contains(&"test-nerve"));
    }

    #[test]
    fn boot_scan_mehrere_nerves() {
        let (_tmp, home) = test_home();
        write_nerve(&home, "nerve-a", r#"
name = "nerve-a"
version = "0.1.0"
description = "A"
binary = "./a"
"#, VALID_CHANNELS);
        write_nerve(&home, "nerve-b", r#"
name = "nerve-b"
version = "0.1.0"
description = "B"
binary = "./b"
"#, VALID_CHANNELS);

        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn boot_scan_fehlerhaftes_manifest_ueberspringen() {
        let (_tmp, home) = test_home();
        // Guter Nerve
        write_nerve(&home, "gut", VALID_MANIFEST, VALID_CHANNELS);
        // Kaputter Nerve (fehlendes Feld)
        write_nerve(&home, "kaputt", "name = \"kaputt\"", VALID_CHANNELS);

        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);
        assert_eq!(registry.len(), 1); // Nur der gute
    }

    // -- Registry Lookup --

    #[test]
    fn registry_find_by_source() {
        let (_tmp, home) = test_home();
        write_nerve(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS);

        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);

        // "nerve/test-nerve" → findet test-nerve
        let found = registry.find_by_source("nerve/test-nerve");
        assert!(found.is_some());
        assert_eq!(found.unwrap().manifest.name, "test-nerve");
    }

    #[test]
    fn registry_find_by_source_mit_sub_topic() {
        let (_tmp, home) = test_home();
        write_nerve(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS);

        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);

        // "nerve/test-nerve/ping" → findet auch test-nerve
        let found = registry.find_by_source("nerve/test-nerve/ping");
        assert!(found.is_some());
    }

    #[test]
    fn registry_find_unbekannter_nerve() {
        let (_tmp, home) = test_home();
        write_nerve(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS);

        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);

        let found = registry.find_by_source("nerve/unknown");
        assert!(found.is_none());
    }

    #[test]
    fn registry_topic_deklariert() {
        let (_tmp, home) = test_home();
        write_nerve(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS);

        let bus = Bus::new(16);
        let registry = boot_scan(&home, &bus);

        let entry = registry.find_by_source("nerve/test-nerve").unwrap();
        assert!(registry.is_topic_declared(entry, "aiux/nerve/test/ping"));
        assert!(!registry.is_topic_declared(entry, "aiux/nerve/test/unknown"));
    }

    // -- Brainstem handle_nerve_signal --

    #[test]
    fn brainstem_unbekannter_nerve() {
        let (_tmp, home) = test_home();
        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/ghost", "boo", &serde_json::Value::Null, "2026-03-02T14:00:00Z"
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("Unbekannter Nerve")),
            other => panic!("Erwartetes SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn brainstem_ohne_interpret_script() {
        let (_tmp, home) = test_home();
        write_nerve(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS);
        // Kein interpret.rhai!

        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve", "ping", &serde_json::Value::Null, "2026-03-02T14:00:00Z"
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("kein interpret.rhai"));
            }
            other => panic!("Erwartetes SystemMessage, bekam: {:?}", other),
        }
    }

    // -- rhai Script-Ausfuehrung --

    #[test]
    fn rhai_forward_true_cortex() {
        let (_tmp, home) = test_home();
        write_nerve_with_script(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS, r#"
            #{ forward: true, target: "cortex", text: `Hallo von ${source}` }
        "#);

        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve", "ping", &serde_json::Value::Null, "2026-03-02T14:00:00Z"
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("Hallo von nerve/test-nerve"));
            }
            other => panic!("Erwartetes SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_forward_false() {
        let (_tmp, home) = test_home();
        write_nerve_with_script(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS, r#"
            #{ forward: false, target: "cortex", text: "nix" }
        "#);

        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve", "ping", &serde_json::Value::Null, "2026-03-02T14:00:00Z"
        );

        // Kein Event erwartet (forward: false)
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn rhai_script_variablen() {
        let (_tmp, home) = test_home();
        write_nerve_with_script(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS, r#"
            #{ forward: true, target: "cortex", text: `${source}|${event}|${ts}` }
        "#);

        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve", "ping", &serde_json::json!({"x":1}), "2026-03-02T14:00:00Z"
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("nerve/test-nerve|ping|2026-03-02T14:00:00Z"));
            }
            other => panic!("Erwartetes SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_script_fehler() {
        let (_tmp, home) = test_home();
        write_nerve_with_script(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS,
            "das ist kein rhai {{{");

        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve", "ping", &serde_json::Value::Null, "2026-03-02T14:00:00Z"
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("Script-Fehler"));
            }
            other => panic!("Erwartetes SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_kein_map_zurueck() {
        let (_tmp, home) = test_home();
        write_nerve_with_script(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS,
            "42");  // Gibt eine Zahl zurueck, kein Map

        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve", "ping", &serde_json::Value::Null, "2026-03-02T14:00:00Z"
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("kein Map"));
            }
            other => panic!("Erwartetes SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_endlosschleife_abbruch() {
        let (_tmp, home) = test_home();
        write_nerve_with_script(&home, "test-nerve", VALID_MANIFEST, VALID_CHANNELS,
            "loop { }");  // Endlosschleife → set_max_operations bricht ab

        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home);
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve", "ping", &serde_json::Value::Null, "2026-03-02T14:00:00Z"
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("Script-Fehler"));
            }
            other => panic!("Erwartetes SystemMessage, bekam: {:?}", other),
        }
    }
}
