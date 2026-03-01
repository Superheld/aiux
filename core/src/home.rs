// Home: Findet das home/ Verzeichnis des Agents.
//
// Lokaler Entwicklungsmodus: home/ im Projektroot.
// Deployed: /home/claude/ auf dem Zielsystem.

use std::path::PathBuf;

/// Findet das home/ Verzeichnis.
pub fn find_home() -> PathBuf {
    let local = PathBuf::from("home");
    if local.join("memory/soul.md").exists() {
        return local;
    }

    let deployed = PathBuf::from("/home/claude");
    if deployed.join("memory/soul.md").exists() {
        return deployed;
    }

    local
}
