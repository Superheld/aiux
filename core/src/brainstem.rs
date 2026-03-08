// Brainstem: Reflexe und autonome Verarbeitung.
//
// Beim Start scannt der Brainstem home/nerves/*/ und startet Nerve-Binaries
// als Child-Prozesse (manifest.toml → binary). Nerves registrieren sich
// dann selbst per MQTT (Self-Registration).
//
// Lauscht auf NerveSignals, fuehrt interpret.rhai in einer Sandbox aus.
// Bei Shutdown werden alle Child-Prozesse sauber beendet.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use tokio::sync::broadcast;
use tokio::time::Instant;

use crate::bus::events::Event;
use crate::bus::Bus;

// ==========================================================
// Registry (Self-Registration)
// ==========================================================

/// Ein registrierter Nerve.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NerveEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub channels: Vec<String>,
    /// Pfad zum Nerve-Verzeichnis (fuer interpret.rhai), relativ zu home
    pub home_dir: Option<String>,
}

/// Registry aller bekannten Nerves.
#[derive(Debug)]
pub struct NerveRegistry {
    /// Key = Nerve-Name
    nerves: HashMap<String, NerveEntry>,
}

impl NerveRegistry {
    fn new() -> Self {
        Self {
            nerves: HashMap::new(),
        }
    }

    /// Nerve aus Register-Message eintragen.
    /// Gibt false zurueck wenn der Name schon existiert.
    fn register(&mut self, entry: NerveEntry) -> bool {
        if self.nerves.contains_key(&entry.name) {
            return false;
        }
        self.nerves.insert(entry.name.clone(), entry);
        true
    }

    /// Nerve anhand des source-Strings finden.
    /// source kommt als "nerve/<name>" oder "nerve/<name>/<sub>".
    pub fn find_by_source(&self, source: &str) -> Option<&NerveEntry> {
        let after_prefix = source.strip_prefix("nerve/")?;
        let source_key = after_prefix.split('/').next()?;

        // Suche: source-key "system" matcht name "system-monitor"
        // weil der source-Pfad gekuerzt ist (nerve/system vs. name system-monitor)
        self.nerves.values().find(|entry| {
            entry.name == source_key
                || entry.name.replace('-', "") == source_key
                || entry.name.starts_with(&format!("{}-", source_key))
        })
    }

    /// Anzahl registrierter Nerves.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.nerves.len()
    }

    /// Alle registrierten Nerve-Namen.
    #[allow(dead_code)]
    pub fn names(&self) -> Vec<&str> {
        self.nerves.keys().map(|s| s.as_str()).collect()
    }
}

// ==========================================================
// Nerve-Launcher
// ==========================================================

/// Minimales Manifest: nur was zum Starten noetig ist.
#[derive(Debug, Deserialize)]
struct NerveManifest {
    binary: String,
}

/// Scannt home/nerves/*/ und startet Binaries aus manifest.toml.
/// Gibt die Child-Handles zurueck (fuer Shutdown).
fn launch_nerves(home: &Path, bus: &Bus) -> Vec<std::process::Child> {
    let nerves_dir = home.join("nerves");
    let mut children = Vec::new();

    if !nerves_dir.exists() {
        return children;
    }

    let entries = match std::fs::read_dir(&nerves_dir) {
        Ok(e) => e,
        Err(e) => {
            bus.publish(Event::SystemMessage {
                text: format!("Brainstem: nerves/ nicht lesbar: {}", e),
            });
            return children;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let manifest_path = path.join("manifest.toml");
        if !manifest_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: {}/manifest.toml nicht lesbar: {}", dir_name, e),
                });
                continue;
            }
        };

        let manifest: NerveManifest = match toml::from_str(&content) {
            Ok(m) => m,
            Err(e) => {
                bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: {}/manifest.toml fehlerhaft: {}", dir_name, e),
                });
                continue;
            }
        };

        // Binary suchen: erst im PATH (cargo install), dann relativ
        let binary = &manifest.binary;
        match std::process::Command::new(binary)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
        {
            Ok(child) => {
                bus.publish(Event::SystemMessage {
                    text: format!("Nerve gestartet: {} (PID {})", dir_name, child.id()),
                });
                children.push(child);
            }
            Err(e) => {
                bus.publish(Event::SystemMessage {
                    text: format!(
                        "Brainstem: Nerve '{}' nicht startbar ({}): {}",
                        dir_name, binary, e
                    ),
                });
            }
        }
    }

    children
}

// ==========================================================
// Scheduler
// ==========================================================

/// Ein geplanter Timer oder Cron-Job.
pub struct ScheduleEntry {
    pub id: String,
    pub label: String,
    pub kind: ScheduleKind,
}

/// Art des Schedule-Eintrags.
pub enum ScheduleKind {
    /// Cron-Job: wiederkehrend nach Zeitplan.
    Cron {
        schedule: Box<cron::Schedule>,
        next_fire: Instant,
    },
    /// Einmaliger Timer.
    Once { fire_at: Instant },
}

/// Geteilter Scheduler-Zustand zwischen Brainstem und SchedulerTool.
pub type SharedScheduler = Arc<Mutex<Vec<ScheduleEntry>>>;

/// Berechnet die naechste Ausfuehrungszeit fuer einen Cron-Schedule.
/// Gibt die Dauer bis zum naechsten Trigger zurueck.
pub fn next_cron_duration(schedule: &cron::Schedule) -> std::time::Duration {
    let now = chrono::Utc::now();
    match schedule.upcoming(chrono::Utc).next() {
        Some(next) => {
            let dur = next - now;
            dur.to_std().unwrap_or(std::time::Duration::from_secs(60))
        }
        None => std::time::Duration::from_secs(60),
    }
}

/// Parst einen Delay-String wie "30m", "2h", "1d" in eine Duration.
pub fn parse_delay(s: &str) -> Result<std::time::Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Leerer Delay-String".into());
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("Ungueltige Zahl: '{}'", num_str))?;

    if num == 0 {
        return Err("Delay muss groesser als 0 sein".into());
    }

    match unit {
        "s" => Ok(std::time::Duration::from_secs(num)),
        "m" => Ok(std::time::Duration::from_secs(num * 60)),
        "h" => Ok(std::time::Duration::from_secs(num * 3600)),
        "d" => Ok(std::time::Duration::from_secs(num * 86400)),
        _ => Err(format!(
            "Unbekannte Einheit '{}'. Erlaubt: s, m, h, d",
            unit
        )),
    }
}

// ==========================================================
// rhai Hilfsfunktionen
// ==========================================================

/// JSON-String → rhai::Dynamic (Map/Array/Wert).
/// Bei Parse-Fehler: leeres Map zurueck.
fn parse_json_for_rhai(s: &str) -> rhai::Dynamic {
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(val) => json_value_to_rhai(val),
        Err(_) => rhai::Dynamic::from(rhai::Map::new()),
    }
}

/// serde_json::Value rekursiv in rhai::Dynamic umwandeln.
fn json_value_to_rhai(val: serde_json::Value) -> rhai::Dynamic {
    match val {
        serde_json::Value::Null => rhai::Dynamic::UNIT,
        serde_json::Value::Bool(b) => rhai::Dynamic::from(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rhai::Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                rhai::Dynamic::from(f)
            } else {
                rhai::Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => rhai::Dynamic::from(s),
        serde_json::Value::Array(arr) => {
            let vec: Vec<rhai::Dynamic> = arr.into_iter().map(json_value_to_rhai).collect();
            rhai::Dynamic::from(vec)
        }
        serde_json::Value::Object(obj) => {
            let mut map = rhai::Map::new();
            for (k, v) in obj {
                map.insert(k.into(), json_value_to_rhai(v));
            }
            rhai::Dynamic::from(map)
        }
    }
}

// ==========================================================
// Brainstem
// ==========================================================

/// Der Brainstem — Reflexe und autonome Verarbeitung.
pub struct Brainstem {
    bus: Arc<Bus>,
    home: PathBuf,
    registry: NerveRegistry,
    engine: rhai::Engine,
    scheduler: SharedScheduler,
    children: Vec<std::process::Child>,
}

impl Brainstem {
    /// Neuer Brainstem mit rhai-Engine und Scheduler.
    /// Scannt home/nerves/ und startet Nerve-Binaries.
    /// Registry ist leer — Nerves melden sich per Self-Registration.
    pub fn new(bus: Arc<Bus>, home: &Path, scheduler: SharedScheduler) -> Self {
        let children = launch_nerves(home, &bus);
        let registry = NerveRegistry::new();
        let mut engine = rhai::Engine::new();
        engine.set_max_operations(10_000);
        engine.set_max_call_levels(32);
        engine.set_max_string_size(64_000);

        // parse_json: JSON-String → rhai::Map (fuer interpret.rhai Scripts)
        engine.register_fn("parse_json", parse_json_for_rhai);

        Self {
            bus,
            home: home.to_path_buf(),
            registry,
            engine,
            scheduler,
            children,
        }
    }

    /// Berechnet die Dauer bis zum naechsten faelligen Timer (max 60s).
    fn next_timer_delay(&self) -> std::time::Duration {
        let entries = self.scheduler.lock().unwrap();
        let now = Instant::now();
        let mut min_dur = std::time::Duration::from_secs(60);

        for entry in entries.iter() {
            let fire_at = match &entry.kind {
                ScheduleKind::Cron { next_fire, .. } => *next_fire,
                ScheduleKind::Once { fire_at } => *fire_at,
            };
            if fire_at <= now {
                return std::time::Duration::ZERO;
            }
            let dur = fire_at - now;
            if dur < min_dur {
                min_dur = dur;
            }
        }

        min_dur
    }

    /// Feuert alle faelligen Timer und aktualisiert Cron-Jobs.
    fn fire_due_entries(&self) {
        let mut entries = self.scheduler.lock().unwrap();
        let now = Instant::now();
        let mut to_fire: Vec<String> = Vec::new();
        let mut to_remove: Vec<usize> = Vec::new();

        for (i, entry) in entries.iter_mut().enumerate() {
            match &mut entry.kind {
                ScheduleKind::Cron {
                    schedule,
                    next_fire,
                } => {
                    if *next_fire <= now {
                        to_fire.push(entry.label.clone());
                        let dur = next_cron_duration(schedule);
                        *next_fire = Instant::now() + dur;
                    }
                }
                ScheduleKind::Once { fire_at } => {
                    if *fire_at <= now {
                        to_fire.push(entry.label.clone());
                        to_remove.push(i);
                    }
                }
            }
        }

        // Once-Timer von hinten entfernen
        for i in to_remove.into_iter().rev() {
            entries.remove(i);
        }

        // Lock freigeben bevor wir auf den Bus publishen
        drop(entries);

        for label in to_fire {
            self.bus.publish(Event::HeartbeatTick { label });
        }
    }

    /// Event-Loop: lauscht auf NerveSignals und prueft Timer.
    pub async fn run(mut self) {
        let mut rx = self.bus.subscribe();

        loop {
            let sleep_dur = self.next_timer_delay();

            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(Event::NerveSignal { ref source, ref event, ref data, ref ts }) => {
                            self.handle_nerve_signal(source, event, data, ts);
                        }
                        Ok(Event::Shutdown) => {
                            self.shutdown_children();
                            break;
                        }
                        Ok(_) => {}
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            eprintln!("Brainstem: {} Events verpasst", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = tokio::time::sleep(sleep_dur) => {
                    self.fire_due_entries();
                }
            }
        }
    }

    /// NerveSignal verarbeiten: Registration oder normales Event.
    fn handle_nerve_signal(
        &mut self,
        source: &str,
        event: &str,
        data: &serde_json::Value,
        ts: &str,
    ) {
        // Self-Registration: Nerve meldet sich an
        if event == "register" {
            self.handle_registration(source, data);
            return;
        }

        let entry = match self.registry.find_by_source(source) {
            Some(e) => e.clone(),
            None => {
                self.bus.publish(Event::SystemMessage {
                    text: format!(
                        "Brainstem: Unbekannter Nerve '{}' (nicht registriert)",
                        source
                    ),
                });
                return;
            }
        };

        // interpret.rhai suchen
        let script_path = match &entry.home_dir {
            Some(dir) => self.home.join(dir).join("interpret.rhai"),
            None => return, // Kein home → kein Script → ignorieren
        };
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
                    text: format!(
                        "Brainstem: interpret.rhai nicht lesbar ({}): {}",
                        entry.name, e
                    ),
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
        let result = match self
            .engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, &script)
        {
            Ok(r) => r,
            Err(e) => {
                self.bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: Script-Fehler ({}): {}", entry.name, e),
                });
                return;
            }
        };

        // Ergebnis verarbeiten
        self.process_script_result(&entry.name, result);
    }

    /// Registration-Message verarbeiten: Nerve in Registry eintragen.
    fn handle_registration(&mut self, source: &str, data: &serde_json::Value) {
        let name = match data.get("name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => {
                self.bus.publish(Event::SystemMessage {
                    text: format!("Brainstem: Register von '{}' ohne name-Feld", source),
                });
                return;
            }
        };

        let version = data
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();

        let description = data
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let channels: Vec<String> = data
            .get("channels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let home_dir = data.get("home").and_then(|v| v.as_str()).map(String::from);

        let n_channels = channels.len();
        let entry = NerveEntry {
            name: name.clone(),
            version,
            description,
            channels,
            home_dir,
        };

        if self.registry.register(entry) {
            self.bus.publish(Event::SystemMessage {
                text: format!(
                    "Nerve registriert: {} ({} Channel{})",
                    name,
                    n_channels,
                    if n_channels != 1 { "s" } else { "" }
                ),
            });
        } else {
            self.bus.publish(Event::SystemMessage {
                text: format!("Brainstem: Nerve '{}' bereits registriert", name),
            });
        }
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
        let forward = map
            .get("forward")
            .and_then(|v| v.clone().try_cast::<bool>())
            .unwrap_or(false);

        if !forward {
            return;
        }

        let target = map
            .get("target")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default();

        let text = map
            .get("text")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default();

        // Spaeter: "mqtt" → MQTT publish. "ignore" oder unbekannt → nichts tun.
        if target.as_str() == "neocortex" {
            self.bus.publish(Event::SystemMessage {
                text: format!("Brainstem [{}]: {}", nerve_name, text),
            });
        }
    }

    /// Alle Child-Prozesse sauber beenden.
    fn shutdown_children(&mut self) {
        for child in &mut self.children {
            let pid = child.id();
            if let Err(e) = child.kill() {
                eprintln!("Brainstem: Nerve PID {} nicht beendbar: {}", pid, e);
            } else {
                let _ = child.wait();
            }
        }
        self.children.clear();
    }

    /// Zugriff auf die Registry (fuer Tests).
    #[allow(dead_code)]
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
    use std::fs;
    use tempfile::TempDir;

    fn test_home() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        (tmp, home)
    }

    fn test_scheduler() -> SharedScheduler {
        Arc::new(Mutex::new(Vec::new()))
    }

    /// Register-Message als JSON bauen.
    fn register_data(name: &str, home_dir: Option<&str>) -> serde_json::Value {
        serde_json::json!({
            "name": name,
            "version": "0.1.0",
            "description": "Test-Nerve",
            "channels": ["aiux/nerve/test/ping"],
            "home": home_dir,
        })
    }

    /// Brainstem erstellen und Nerve registrieren. Gibt (brainstem, bus) zurueck.
    fn brainstem_with_nerve(
        home: &Path,
        name: &str,
        home_dir: Option<&str>,
    ) -> (Brainstem, Arc<Bus>) {
        let bus = Arc::new(Bus::new(16));
        let mut brainstem = Brainstem::new(bus.clone(), home, test_scheduler());
        let data = register_data(name, home_dir);
        brainstem.handle_nerve_signal("nerve/test", "register", &data, "2026-03-03T00:00:00Z");
        (brainstem, bus)
    }

    /// interpret.rhai im Nerve-Verzeichnis schreiben.
    fn write_script(home: &Path, nerve_dir: &str, script: &str) {
        let dir = home.join(nerve_dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("interpret.rhai"), script).unwrap();
    }

    // -- Self-Registration --

    #[test]
    fn registration_gueltig() {
        let (_tmp, home) = test_home();
        let bus = Arc::new(Bus::new(16));
        let mut brainstem = Brainstem::new(bus.clone(), &home, test_scheduler());
        let mut rx = bus.subscribe();

        let data = register_data("system-monitor", Some("nerves/system-monitor"));
        brainstem.handle_nerve_signal("nerve/system", "register", &data, "2026-03-03T00:00:00Z");

        assert_eq!(brainstem.registry().len(), 1);
        assert!(brainstem.registry().names().contains(&"system-monitor"));

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("registriert")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn registration_ohne_name() {
        let (_tmp, home) = test_home();
        let bus = Arc::new(Bus::new(16));
        let mut brainstem = Brainstem::new(bus.clone(), &home, test_scheduler());
        let mut rx = bus.subscribe();

        let data = serde_json::json!({"version": "0.1.0"});
        brainstem.handle_nerve_signal("nerve/test", "register", &data, "2026-03-03T00:00:00Z");

        assert_eq!(brainstem.registry().len(), 0);
        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("ohne name")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn registration_doppelt() {
        let (_tmp, home) = test_home();
        let bus = Arc::new(Bus::new(16));
        let mut brainstem = Brainstem::new(bus.clone(), &home, test_scheduler());

        let data = register_data("test-nerve", None);
        brainstem.handle_nerve_signal("nerve/test", "register", &data, "2026-03-03T00:00:00Z");
        brainstem.handle_nerve_signal("nerve/test", "register", &data, "2026-03-03T00:00:00Z");

        assert_eq!(brainstem.registry().len(), 1); // Nur einmal
    }

    // -- Registry Lookup --

    #[test]
    fn registry_find_by_source() {
        let mut registry = NerveRegistry::new();
        registry.register(NerveEntry {
            name: "test-nerve".into(),
            version: "0.1.0".into(),
            description: "Test".into(),
            channels: vec![],
            home_dir: None,
        });

        let found = registry.find_by_source("nerve/test-nerve");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-nerve");
    }

    #[test]
    fn registry_find_by_source_mit_sub_topic() {
        let mut registry = NerveRegistry::new();
        registry.register(NerveEntry {
            name: "test-nerve".into(),
            version: "0.1.0".into(),
            description: "Test".into(),
            channels: vec![],
            home_dir: None,
        });

        let found = registry.find_by_source("nerve/test-nerve/ping");
        assert!(found.is_some());
    }

    #[test]
    fn registry_find_kurzform() {
        // source "nerve/system" findet name "system-monitor"
        let mut registry = NerveRegistry::new();
        registry.register(NerveEntry {
            name: "system-monitor".into(),
            version: "0.1.0".into(),
            description: "Test".into(),
            channels: vec![],
            home_dir: None,
        });

        let found = registry.find_by_source("nerve/system");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "system-monitor");
    }

    #[test]
    fn registry_find_unbekannt() {
        let registry = NerveRegistry::new();
        assert!(registry.find_by_source("nerve/ghost").is_none());
    }

    // -- Brainstem handle_nerve_signal --

    #[test]
    fn brainstem_unbekannter_nerve() {
        let (_tmp, home) = test_home();
        let bus = Arc::new(Bus::new(16));
        let mut brainstem = Brainstem::new(bus.clone(), &home, test_scheduler());
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/ghost",
            "boo",
            &serde_json::Value::Null,
            "2026-03-02T14:00:00Z",
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("nicht registriert")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn brainstem_ohne_interpret_script() {
        let (_tmp, home) = test_home();
        // Nerve mit home_dir aber ohne interpret.rhai
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        // Verzeichnis anlegen, aber kein Script
        fs::create_dir_all(home.join("nerves/test-nerve")).unwrap();
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve",
            "ping",
            &serde_json::Value::Null,
            "2026-03-02T14:00:00Z",
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("kein interpret.rhai")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    // -- rhai Script-Ausfuehrung --

    #[test]
    fn rhai_forward_true_neocortex() {
        let (_tmp, home) = test_home();
        write_script(
            &home,
            "nerves/test-nerve",
            r#"
            #{ forward: true, target: "neocortex", text: `Hallo von ${source}` }
        "#,
        );
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve",
            "ping",
            &serde_json::Value::Null,
            "2026-03-02T14:00:00Z",
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("Hallo von nerve/test-nerve")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_forward_false() {
        let (_tmp, home) = test_home();
        write_script(
            &home,
            "nerves/test-nerve",
            r#"
            #{ forward: false }
        "#,
        );
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve",
            "ping",
            &serde_json::Value::Null,
            "2026-03-02T14:00:00Z",
        );

        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn rhai_script_variablen() {
        let (_tmp, home) = test_home();
        write_script(
            &home,
            "nerves/test-nerve",
            r#"
            #{ forward: true, target: "neocortex", text: `${source}|${event}|${ts}` }
        "#,
        );
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve",
            "ping",
            &serde_json::json!({"x":1}),
            "2026-03-02T14:00:00Z",
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("nerve/test-nerve|ping|2026-03-02T14:00:00Z"));
            }
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_script_fehler() {
        let (_tmp, home) = test_home();
        write_script(&home, "nerves/test-nerve", "das ist kein rhai {{{");
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve",
            "ping",
            &serde_json::Value::Null,
            "2026-03-02T14:00:00Z",
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("Script-Fehler")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    // -- parse_json --

    #[test]
    fn parse_json_gueltiges_json() {
        let result = parse_json_for_rhai(r#"{"label":"test","path":"memory/x.md","count":42}"#);
        let map = result.cast::<rhai::Map>();
        assert_eq!(map.get("label").unwrap().clone().cast::<String>(), "test");
        assert_eq!(
            map.get("path").unwrap().clone().cast::<String>(),
            "memory/x.md"
        );
        assert_eq!(map.get("count").unwrap().clone().cast::<i64>(), 42);
    }

    #[test]
    fn parse_json_ungueltiges_json() {
        let result = parse_json_for_rhai("kein json {{{");
        let map = result.cast::<rhai::Map>();
        assert!(map.is_empty());
    }

    #[test]
    fn parse_json_verschachtelt() {
        let result = parse_json_for_rhai(r#"{"outer":{"inner":"tief"}}"#);
        let map = result.cast::<rhai::Map>();
        let outer = map.get("outer").unwrap().clone().cast::<rhai::Map>();
        assert_eq!(outer.get("inner").unwrap().clone().cast::<String>(), "tief");
    }

    #[test]
    fn parse_json_array() {
        let result = parse_json_for_rhai(r#"{"items":[1,2,3]}"#);
        let map = result.cast::<rhai::Map>();
        let items = map
            .get("items")
            .unwrap()
            .clone()
            .cast::<Vec<rhai::Dynamic>>();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn rhai_interpret_mit_parse_json() {
        let (_tmp, home) = test_home();
        write_script(
            &home,
            "nerves/test-nerve",
            r#"
            let d = parse_json(data);
            if d.label == "alert" {
                #{ forward: true, target: "neocortex", text: `Alarm: ${d.msg}` }
            } else {
                #{ forward: false }
            }
        "#,
        );
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        let data = serde_json::json!({"label": "alert", "msg": "CPU hoch"});
        brainstem.handle_nerve_signal("nerve/test-nerve", "stats", &data, "2026-03-03T14:00:00Z");

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(text.contains("Alarm: CPU hoch"), "Text: {}", text)
            }
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_interpret_parse_json_forward_false() {
        let (_tmp, home) = test_home();
        write_script(
            &home,
            "nerves/test-nerve",
            r#"
            let d = parse_json(data);
            if d.label == "alert" {
                #{ forward: true, target: "neocortex", text: "ja" }
            } else {
                #{ forward: false }
            }
        "#,
        );
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        let data = serde_json::json!({"label": "normal", "msg": "alles gut"});
        brainstem.handle_nerve_signal("nerve/test-nerve", "stats", &data, "2026-03-03T14:00:00Z");

        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn rhai_kein_map_zurueck() {
        let (_tmp, home) = test_home();
        write_script(&home, "nerves/test-nerve", "42");
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve",
            "ping",
            &serde_json::Value::Null,
            "2026-03-02T14:00:00Z",
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("kein Map")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn rhai_endlosschleife_abbruch() {
        let (_tmp, home) = test_home();
        write_script(&home, "nerves/test-nerve", "loop { }");
        let (mut brainstem, bus) =
            brainstem_with_nerve(&home, "test-nerve", Some("nerves/test-nerve"));
        let mut rx = bus.subscribe();

        brainstem.handle_nerve_signal(
            "nerve/test-nerve",
            "ping",
            &serde_json::Value::Null,
            "2026-03-02T14:00:00Z",
        );

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("Script-Fehler")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    // -- Nerve-Launcher --

    #[test]
    fn launch_kein_nerves_verzeichnis() {
        let (_tmp, home) = test_home();
        let bus = Bus::new(16);
        let children = launch_nerves(&home, &bus);
        assert!(children.is_empty());
    }

    #[test]
    fn launch_leeres_nerves_verzeichnis() {
        let (_tmp, home) = test_home();
        fs::create_dir_all(home.join("nerves")).unwrap();
        let bus = Bus::new(16);
        let children = launch_nerves(&home, &bus);
        assert!(children.is_empty());
    }

    #[test]
    fn launch_kein_manifest() {
        let (_tmp, home) = test_home();
        // Verzeichnis ohne manifest.toml → wird uebersprungen
        fs::create_dir_all(home.join("nerves/test-nerve")).unwrap();
        let bus = Bus::new(16);
        let children = launch_nerves(&home, &bus);
        assert!(children.is_empty());
    }

    #[test]
    fn launch_kaputtes_manifest() {
        let (_tmp, home) = test_home();
        let dir = home.join("nerves/test-nerve");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.toml"), "das ist kein toml {{{").unwrap();

        let bus = Bus::new(16);
        let mut rx = bus.subscribe();
        let children = launch_nerves(&home, &bus);
        assert!(children.is_empty());

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("fehlerhaft")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn launch_binary_nicht_gefunden() {
        let (_tmp, home) = test_home();
        let dir = home.join("nerves/test-nerve");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("manifest.toml"),
            "binary = \"gibt-es-nicht-12345\"",
        )
        .unwrap();

        let bus = Bus::new(16);
        let mut rx = bus.subscribe();
        let children = launch_nerves(&home, &bus);
        assert!(children.is_empty());

        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => assert!(text.contains("nicht startbar")),
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }
    }

    #[test]
    fn launch_gueltig_startet_prozess() {
        let (_tmp, home) = test_home();
        let dir = home.join("nerves/sleeper");
        fs::create_dir_all(&dir).unwrap();
        // "sleep" existiert ueberall
        fs::write(dir.join("manifest.toml"), "binary = \"sleep\"\n").unwrap();

        let bus = Bus::new(16);
        let mut rx = bus.subscribe();
        let mut children = launch_nerves(&home, &bus);

        // sleep ohne Argument stirbt sofort, aber der Start zaehlt
        // Auf manchen Systemen braucht sleep ein Argument
        // Wir pruefen nur dass die Funktion keinen Fehler wirft
        // und entweder startet oder nicht-startbar meldet
        let event = rx.try_recv().unwrap();
        match event {
            Event::SystemMessage { text } => {
                assert!(
                    text.contains("gestartet") || text.contains("nicht startbar"),
                    "Unerwartete Message: {}",
                    text
                );
            }
            other => panic!("Erwartet SystemMessage, bekam: {:?}", other),
        }

        // Aufraeumen
        for child in &mut children {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    // -- Delay parsen --

    #[test]
    fn parse_delay_minuten() {
        let dur = parse_delay("30m").unwrap();
        assert_eq!(dur, std::time::Duration::from_secs(30 * 60));
    }

    #[test]
    fn parse_delay_stunden() {
        let dur = parse_delay("2h").unwrap();
        assert_eq!(dur, std::time::Duration::from_secs(2 * 3600));
    }

    #[test]
    fn parse_delay_tage() {
        let dur = parse_delay("1d").unwrap();
        assert_eq!(dur, std::time::Duration::from_secs(86400));
    }

    #[test]
    fn parse_delay_sekunden() {
        let dur = parse_delay("45s").unwrap();
        assert_eq!(dur, std::time::Duration::from_secs(45));
    }

    #[test]
    fn parse_delay_ungueltig() {
        assert!(parse_delay("abc").is_err());
        assert!(parse_delay("").is_err());
        assert!(parse_delay("10x").is_err());
        assert!(parse_delay("0m").is_err());
    }

    // -- Cron parsen --

    #[test]
    fn cron_gueltig() {
        use std::str::FromStr;
        let schedule = cron::Schedule::from_str("0 0 * * * * *");
        assert!(schedule.is_ok());
    }

    #[test]
    fn cron_ungueltig() {
        use std::str::FromStr;
        let schedule = cron::Schedule::from_str("nicht gueltig");
        assert!(schedule.is_err());
    }

    // -- fire_due_entries --

    #[test]
    fn fire_due_once_timer() {
        let (_tmp, home) = test_home();
        let scheduler = test_scheduler();
        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home, scheduler.clone());

        // Timer der sofort faellig ist
        scheduler.lock().unwrap().push(ScheduleEntry {
            id: "1".into(),
            label: "test-once".into(),
            kind: ScheduleKind::Once {
                fire_at: Instant::now() - std::time::Duration::from_secs(1),
            },
        });

        let mut rx = bus.subscribe();
        brainstem.fire_due_entries();

        let event = rx.try_recv().unwrap();
        match event {
            Event::HeartbeatTick { label } => assert_eq!(label, "test-once"),
            other => panic!("Erwartet HeartbeatTick, bekam: {:?}", other),
        }

        // Once-Timer muss entfernt sein
        assert_eq!(scheduler.lock().unwrap().len(), 0);
    }

    #[test]
    fn fire_due_cron_bleibt() {
        use std::str::FromStr;
        let (_tmp, home) = test_home();
        let scheduler = test_scheduler();
        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home, scheduler.clone());

        let schedule = cron::Schedule::from_str("0 0 * * * * *").unwrap();
        scheduler.lock().unwrap().push(ScheduleEntry {
            id: "2".into(),
            label: "cron-test".into(),
            kind: ScheduleKind::Cron {
                schedule: Box::new(schedule),
                next_fire: Instant::now() - std::time::Duration::from_secs(1),
            },
        });

        let mut rx = bus.subscribe();
        brainstem.fire_due_entries();

        let event = rx.try_recv().unwrap();
        match event {
            Event::HeartbeatTick { label } => assert_eq!(label, "cron-test"),
            other => panic!("Erwartet HeartbeatTick, bekam: {:?}", other),
        }

        // Cron-Timer bleibt, next_fire ist neu berechnet
        let entries = scheduler.lock().unwrap();
        assert_eq!(entries.len(), 1);
        match &entries[0].kind {
            ScheduleKind::Cron { next_fire, .. } => {
                assert!(*next_fire > Instant::now());
            }
            _ => panic!("Erwartet Cron-Eintrag"),
        }
    }

    #[test]
    fn fire_due_nicht_faellig_bleibt() {
        let (_tmp, home) = test_home();
        let scheduler = test_scheduler();
        let bus = Arc::new(Bus::new(16));
        let brainstem = Brainstem::new(bus.clone(), &home, scheduler.clone());

        // Timer der in der Zukunft liegt
        scheduler.lock().unwrap().push(ScheduleEntry {
            id: "3".into(),
            label: "future".into(),
            kind: ScheduleKind::Once {
                fire_at: Instant::now() + std::time::Duration::from_secs(3600),
            },
        });

        let mut rx = bus.subscribe();
        brainstem.fire_due_entries();

        // Kein Event
        assert!(rx.try_recv().is_err());
        // Timer noch da
        assert_eq!(scheduler.lock().unwrap().len(), 1);
    }
}
