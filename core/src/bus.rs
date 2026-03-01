// Interner Event-Bus auf Basis von tokio::sync::broadcast.
//
// Ein Sender, beliebig viele Empfaenger.
// Kein MQTT, kein Netzwerk - rein in-process.

use tokio::sync::broadcast;

use crate::events::Event;

/// Der Event-Bus. Verteilt Events an alle Subscriber.
#[derive(Debug)]
pub struct Bus {
    sender: broadcast::Sender<Event>,
}

impl Bus {
    /// Neuer Bus mit gegebener Kapazitaet.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Event an alle Subscriber senden.
    pub fn publish(&self, event: Event) {
        // Fehler ignorieren wenn niemand zuhoert
        let _ = self.sender.send(event);
    }

    /// Neuen Receiver holen. Jeder Subscriber braucht seinen eigenen.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}
