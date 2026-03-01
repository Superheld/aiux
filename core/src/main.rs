// aiux-core: Embodied AI Agent
//
// main.rs ist nur der Verdrahter:
// Bus erstellen, Module anschliessen, laufen lassen.

mod bus;
mod config;
mod core;
mod events;
mod memory;
mod repl;

use std::sync::Arc;

use crate::bus::Bus;
use crate::config::Config;
use crate::core::Core;
use crate::repl::Repl;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let bus = Arc::new(Bus::new(1024));
    let home = core::find_home();

    // Config laden
    let config = Config::load(&home)?;

    // Core: das Gehirn
    let core = Core::new(bus.clone(), home, config);
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
