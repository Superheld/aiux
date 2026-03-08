// agent/ - Alle Agents leben hier.
//
// Jeder Agent ist eine eigene Datei.
// mod.rs ist nur der Einstiegspunkt (Rust-Konvention fuer Verzeichnis-Module).

mod hippocampus;
pub mod neocortex;

pub use neocortex::{BootInfo, Core};
