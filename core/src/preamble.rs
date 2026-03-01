// Preamble: System-Prompt aus Teilen zusammenbauen.
//
// Boot-Sequence:
// 1. soul.md - Wer bin ich?
// 2. user.md - Mit wem rede ich?
// 3. context/*.md - Was weiss ich noch?

use std::fs;
use std::path::{Path, PathBuf};

/// Laedt eine Datei oder gibt einen leeren String zurueck.
fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Laedt alle .md Dateien aus einem Verzeichnis, alphabetisch sortiert.
pub fn load_context_files(dir: &Path) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "md"))
            .collect();

        paths.sort();

        for path in paths {
            let name = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let content = read_file(&path);
            if !content.is_empty() {
                files.push((name, content));
            }
        }
    }

    files
}

/// Zaehlt die Context-Dateien (fuer Boot-Info).
pub fn count_context_files(home: &Path) -> usize {
    load_context_files(&home.join("memory/context")).len()
}

/// Baut den System-Prompt zusammen (Boot-Sequence):
/// 1. soul.md - Wer bin ich?
/// 2. user.md - Mit wem rede ich?
/// 3. context/*.md - Was weiss ich noch?
pub fn load_preamble(home: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();

    let soul = read_file(&home.join("memory/soul.md"));
    if !soul.is_empty() {
        parts.push(soul);
    }

    let user = read_file(&home.join("memory/user.md"));
    if !user.is_empty() {
        parts.push(user);
    }

    let context_files = load_context_files(&home.join("memory/context"));
    for (name, content) in &context_files {
        parts.push(format!("# Kontext: {}\n\n{}", name, content));
    }

    parts.join("\n\n---\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_home() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();
        fs::create_dir_all(home.join("memory/context")).unwrap();
        (tmp, home)
    }

    // ==========================================================
    // load_preamble() / load_context_files()
    // ==========================================================

    #[test]
    fn preamble_mit_soul_und_user() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "Ich bin AIUX.").unwrap();
        fs::write(home.join("memory/user.md"), "Bruce ist cool.").unwrap();

        let preamble = load_preamble(&home);
        assert!(preamble.contains("Ich bin AIUX."));
        assert!(preamble.contains("Bruce ist cool."));
        assert!(preamble.contains("---")); // Trenner
    }

    #[test]
    fn preamble_nur_soul() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "Ich bin AIUX.").unwrap();

        let preamble = load_preamble(&home);
        assert_eq!(preamble, "Ich bin AIUX.");
    }

    #[test]
    fn preamble_ohne_dateien() {
        let (_tmp, home) = test_home();
        let preamble = load_preamble(&home);
        assert!(preamble.is_empty());
    }

    #[test]
    fn preamble_mit_context_dateien() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "Soul.").unwrap();
        fs::write(home.join("memory/context/a.md"), "AAA").unwrap();
        fs::write(home.join("memory/context/b.md"), "BBB").unwrap();

        let preamble = load_preamble(&home);
        assert!(preamble.contains("# Kontext: a"));
        assert!(preamble.contains("AAA"));
        assert!(preamble.contains("# Kontext: b"));
        assert!(preamble.contains("BBB"));
    }

    #[test]
    fn preamble_leere_dateien_werden_ignoriert() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/soul.md"), "").unwrap();
        fs::write(home.join("memory/context/leer.md"), "").unwrap();

        let preamble = load_preamble(&home);
        assert!(preamble.is_empty());
    }

    #[test]
    fn context_files_nur_md() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/context/notiz.md"), "Inhalt").unwrap();
        fs::write(home.join("memory/context/bild.png"), "binary").unwrap();
        fs::write(home.join("memory/context/readme.txt"), "text").unwrap();

        let files = load_context_files(&home.join("memory/context"));
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "notiz");
    }

    #[test]
    fn context_files_sortiert() {
        let (_tmp, home) = test_home();
        fs::write(home.join("memory/context/c.md"), "C").unwrap();
        fs::write(home.join("memory/context/a.md"), "A").unwrap();
        fs::write(home.join("memory/context/b.md"), "B").unwrap();

        let files = load_context_files(&home.join("memory/context"));
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn context_files_leeres_verzeichnis() {
        let (_tmp, home) = test_home();
        let files = load_context_files(&home.join("memory/context"));
        assert!(files.is_empty());
    }

    #[test]
    fn context_files_verzeichnis_existiert_nicht() {
        let (_tmp, home) = test_home();
        let files = load_context_files(&home.join("memory/gibts_nicht"));
        assert!(files.is_empty());
    }
}
