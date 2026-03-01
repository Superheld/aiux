// agent/ - Alle Agents leben hier.
//
// Jeder Agent ist eine eigene Datei.
// mod.rs ist nur der Einstiegspunkt (Rust-Konvention fuer Verzeichnis-Module).

pub mod cortex;
mod hippocampus;

pub use cortex::{BootInfo, Core};
