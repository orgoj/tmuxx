//! Persistent history for Command Palette and other input dialogs.
//!
//! History is stored in plain text files (one entry per line) at:
//! `~/.config/tmuxx/history/<use_case>.txt`

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Get history file path for a given use-case.
///
/// Returns `None` if the config directory cannot be determined.
pub fn history_path(use_case: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|p| {
        p.join("tmuxx")
            .join("history")
            .join(format!("{}.txt", use_case))
    })
}

/// Load history from file (newest first).
///
/// Returns an empty Vec if the file doesn't exist or cannot be read.
pub fn load_history(use_case: &str, max_items: usize) -> Vec<String> {
    let Some(path) = history_path(use_case) else {
        return Vec::new();
    };

    let Ok(file) = fs::File::open(&path) else {
        return Vec::new();
    };

    BufReader::new(file)
        .lines()
        .take(max_items)
        .filter_map(|l| l.ok())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Save history to file (overwrites).
///
/// Creates the directory structure if it doesn't exist.
pub fn save_history(use_case: &str, items: &[String], max_items: usize) -> std::io::Result<()> {
    let Some(path) = history_path(use_case) else {
        return Ok(());
    };

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(&path)?;
    for item in items.iter().take(max_items) {
        writeln!(file, "{}", item)?;
    }
    Ok(())
}

/// Add item to history (prepends, deduplicates, truncates).
///
/// This modifies the history in-place:
/// 1. Removes any existing duplicates of the new item
/// 2. Prepends the new item to the front
/// 3. Truncates to max_items
pub fn add_to_history(items: &mut Vec<String>, new_item: String, max_items: usize) {
    // Remove duplicates
    items.retain(|i| i != &new_item);
    // Prepend new item
    items.insert(0, new_item);
    // Truncate to max
    items.truncate(max_items);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_add_to_history_basic() {
        let mut history = vec!["old1".to_string(), "old2".to_string()];
        add_to_history(&mut history, "new".to_string(), 10);
        assert_eq!(history, vec!["new", "old1", "old2"]);
    }

    #[test]
    fn test_add_to_history_deduplicates() {
        let mut history = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        add_to_history(&mut history, "b".to_string(), 10);
        assert_eq!(history, vec!["b", "a", "c"]);
    }

    #[test]
    fn test_add_to_history_truncates() {
        let mut history = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        add_to_history(&mut history, "new".to_string(), 3);
        assert_eq!(history, vec!["new", "1", "2"]);
    }

    #[test]
    fn test_history_path() {
        let path = history_path("test_case");
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("tmuxx"));
        assert!(path.to_string_lossy().contains("history"));
        assert!(path.to_string_lossy().ends_with("test_case.txt"));
    }

    #[test]
    fn test_save_and_load_history() {
        // Use a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let history_dir = temp_dir.path().join("history");
        fs::create_dir_all(&history_dir).unwrap();
        let history_file = history_dir.join("test.txt");

        // Write test history
        let items = vec!["cmd1".to_string(), "cmd2".to_string(), "cmd3".to_string()];
        let mut file = fs::File::create(&history_file).unwrap();
        for item in &items {
            writeln!(file, "{}", item).unwrap();
        }
        drop(file);

        // Read it back
        let loaded: Vec<String> = BufReader::new(fs::File::open(&history_file).unwrap())
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| !l.is_empty())
            .collect();

        assert_eq!(loaded, items);
    }

    #[test]
    fn test_load_nonexistent_file() {
        // load_history should return empty vec for nonexistent files
        let history = load_history("nonexistent_test_case_12345", 100);
        assert!(history.is_empty());
    }
}
