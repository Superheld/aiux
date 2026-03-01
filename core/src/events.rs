// Event-Typen fuer den internen Event-Bus.
//
// Alles kommt als Event rein, alles geht als Event raus.
// Aktuell nur die Basis-Events fuer REPL <-> Core Kommunikation.
// Spaeter kommen NerveEvent, SchedulerTick etc. dazu.

/// Alle Events die ueber den Bus laufen.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Event {
    // -- Input --
    /// User hat etwas eingegeben
    UserInput { text: String },

    // -- Response (Streaming) --
    /// Ein einzelnes Token aus der LLM-Antwort
    ResponseToken { text: String },
    /// Antwort komplett - full_text enthaelt den ganzen Text
    ResponseComplete { full_text: String },

    // -- System --
    /// Sauber herunterfahren
    Shutdown,

    // -- REPL-Befehle --
    /// History loeschen
    ClearHistory,
}
