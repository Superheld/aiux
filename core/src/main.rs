// aiux-core: Embodied AI Agent
//
// main.rs ist nur der Verdrahter:
// Bus erstellen, Module anschliessen, laufen lassen.

mod agent;
mod brainstem;
mod bus;
mod config;
mod history;
mod home;
mod mqtt;
mod repl;
mod tools;

use std::sync::{Arc, Mutex};

use crate::brainstem::Brainstem;
use crate::bus::Bus;
use crate::config::Config;
use crate::agent::Core;
use crate::mqtt::MqttBridge;
use crate::repl::Repl;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let bus = Arc::new(Bus::new(1024));
    let home = home::find_home();

    // Config laden
    let config = Config::load(&home)?;

    // MQTT-Bridge: Verbindung zur Aussenwelt (optional)
    let mqtt_host = config.mqtt_host.clone();
    let mqtt_port = config.mqtt_port;

    // SharedScheduler: geteilter Zustand fuer Timer/Cron
    let scheduler = Arc::new(Mutex::new(Vec::new()));

    // Brainstem: Reflexe und Nerve-Verarbeitung
    let brainstem = Brainstem::new(bus.clone(), &home, scheduler.clone());
    tokio::spawn(async move { brainstem.run().await });

    // Core: das Gehirn (konsumiert config + home)
    let core = Core::new(bus.clone(), home, config, scheduler);

    if let Some(host) = mqtt_host {
        let port = mqtt_port.unwrap_or(1883);
        let bridge = MqttBridge::new(bus.clone(), &host, port);
        tokio::spawn(async move { bridge.run().await });
    }
    let boot_info = core.boot_info();

    if !boot_info.has_soul {
        eprintln!("Warnung: Keine soul.md gefunden. Agent hat keine Persoenlichkeit.");
    }

    // REPL: die Kommandozeile
    let repl = Arc::new(Repl::new(bus.clone()));
    repl.print_boot_info(&boot_info);

    // Alles parallel laufen lassen
    let repl_output = {
        let repl = repl.clone();
        tokio::spawn(async move { repl.run_output().await })
    };
    let repl_input = {
        let repl = repl.clone();
        tokio::spawn(async move { repl.run_input().await })
    };
    let core_task = tokio::spawn(async move { core.run().await });

    // Warten bis einer fertig ist (normalerweise REPL input bei quit)
    tokio::select! {
        _ = repl_input => {}
        _ = repl_output => {}
        result = core_task => {
            if let Ok(Err(e)) = result {
                eprintln!("Core-Fehler: {}", e);
            }
        }
    }

    Ok(())
}
