// REPL: Die Kommandozeile des Agents.
//
// Liest von stdin, publiziert UserInput Events.
// Empfaengt ResponseToken Events und gibt sie auf stdout aus.
// Spaeter ersetzbar durch Gateway, TUI, oder anderes Frontend.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use crate::bus::Bus;
use crate::agent::BootInfo;
use crate::bus::events::Event;

/// REPL - Read-Eval-Print-Loop als Event-Teilnehmer.
pub struct Repl {
    bus: Arc<Bus>,
}

impl Repl {
    pub fn new(bus: Arc<Bus>) -> Self {
        Self { bus }
    }

    /// Boot-Info anzeigen.
    pub fn print_boot_info(&self, info: &BootInfo) {
        println!("AIUX v0.1.0");
        if info.has_soul { println!("  [+] soul.md"); }
        if info.has_user { println!("  [+] user.md"); }
        if info.has_shortterm { println!("  [+] shortterm.md"); }
        println!("  [+] Tools: soul, user, memory");
        if info.history_count > 0 {
            println!("  [+] {} History-Nachrichten", info.history_count);
        }
        println!("Zum Beenden: /quit | /clear = History loeschen\n");
        print!("Du: ");
        io::stdout().flush().ok();
    }

    /// Input-Schleife: stdin lesen und Events publishen.
    /// Laeuft in einem eigenen Task (blocking wegen stdin).
    pub async fn run_input(self: Arc<Self>) {
        // stdin ist blocking - in einem eigenen Thread laufen lassen
        let bus = self.bus.clone();
        tokio::task::spawn_blocking(move || {
            let stdin = io::stdin();
            loop {
                // Prompt wird von run_output gedruckt (nach Boot-Info und nach jeder Antwort),
                // damit er nicht mit der Antwort kollidiert.
                let mut input = String::new();
                match stdin.lock().read_line(&mut input) {
                    Ok(0) | Err(_) => break, // EOF oder Fehler -> beenden
                    _ => {}
                }
                let input = input.trim().to_string();

                if input.is_empty() {
                    continue;
                }

                if input == "/quit" || input == "/exit" {
                    println!("\nTschuess.");
                    bus.publish(Event::Shutdown);
                    break;
                }

                if input == "/clear" {
                    bus.publish(Event::ClearHistory);
                    continue;
                }

                bus.publish(Event::UserInput { text: input });
            }
        })
        .await
        .ok();
    }

    /// Output-Schleife: Response-Events empfangen und auf stdout ausgeben.
    pub async fn run_output(&self) {
        let mut receiver = self.bus.subscribe();

        loop {
            match receiver.recv().await {
                Ok(Event::UserInput { .. }) => {
                    // Antwort-Label mit Leerzeile davor
                    print!("\nAIUX: ");
                    io::stdout().flush().ok();
                }
                Ok(Event::ResponseToken { text }) => {
                    print!("{}", text);
                    io::stdout().flush().ok();
                }
                Ok(Event::ResponseComplete { .. }) => {
                    // Leerzeile nach Antwort, dann neuer Prompt
                    println!("\n");
                    print!("Du: ");
                    io::stdout().flush().ok();
                }
                Ok(Event::ClearHistory) => {
                    println!("History geloescht.\n");
                    print!("Du: ");
                    io::stdout().flush().ok();
                }
                Ok(Event::ToolCall { name }) => {
                    println!("\n[tool: {}]", name);
                    io::stdout().flush().ok();
                }
                Ok(Event::SystemMessage { text }) => {
                    println!("\n[{}]", text);
                    io::stdout().flush().ok();
                }
                Ok(Event::Compacting) => {
                    print!("\n[kompaktifiziere...] ");
                    io::stdout().flush().ok();
                }
                Ok(Event::Compacted) => {
                    println!("fertig.\n");
                    print!("Du: ");
                    io::stdout().flush().ok();
                }
                Ok(Event::NerveSignal { source, event, .. }) => {
                    println!("\n[nerve: {} → {}]", source, event);
                    io::stdout().flush().ok();
                }
                Ok(Event::Shutdown) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("REPL: {} Events verpasst", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}
