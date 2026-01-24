//! Session pattern matching with auto-detection of pattern type.
//!
//! Supports three pattern types:
//! - Fixed: exact string match (default)
//! - Glob: shell-style wildcards (* and ?)
//! - Regex: full regular expressions (wrapped in /.../)

use anyhow::{anyhow, Result};
use glob::Pattern as GlobPattern;
use regex::Regex;

/// Pattern for matching tmux session names
#[derive(Debug, Clone)]
pub enum SessionPattern {
    /// Exact string match
    Fixed(String),
    /// Glob pattern (supports * and ?)
    Glob(GlobPattern),
    /// Regular expression
    Regex(Regex),
}

impl SessionPattern {
    /// Parse a pattern string with auto-detection of pattern type.
    ///
    /// Detection rules:
    /// - `/pattern/` → Regex
    /// - Contains `*` or `?` → Glob
    /// - Otherwise → Fixed (exact match)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// SessionPattern::parse("cc-prod")       // Fixed match
    /// SessionPattern::parse("test-*")        // Glob pattern
    /// SessionPattern::parse("/^ssh-\\d+$/")  // Regex pattern
    /// ```
    pub fn parse(pattern: &str) -> Result<Self> {
        // Check for regex: /pattern/
        if pattern.starts_with('/') && pattern.ends_with('/') && pattern.len() > 2 {
            let inner = &pattern[1..pattern.len() - 1];
            let re = Regex::new(inner)
                .map_err(|e| anyhow!("Invalid regex pattern '{}': {}", inner, e))?;
            return Ok(SessionPattern::Regex(re));
        }

        // Check for glob: contains * or ?
        if pattern.contains('*') || pattern.contains('?') {
            let glob = GlobPattern::new(pattern)
                .map_err(|e| anyhow!("Invalid glob pattern '{}': {}", pattern, e))?;
            return Ok(SessionPattern::Glob(glob));
        }

        // Default: fixed (exact match)
        Ok(SessionPattern::Fixed(pattern.to_string()))
    }

    /// Check if a session name matches this pattern
    pub fn matches(&self, session: &str) -> bool {
        match self {
            SessionPattern::Fixed(s) => s == session,
            SessionPattern::Glob(g) => g.matches(session),
            SessionPattern::Regex(r) => r.is_match(session),
        }
    }

    /// Get a description of the pattern type for debugging
    #[allow(dead_code)]
    pub fn pattern_type(&self) -> &'static str {
        match self {
            SessionPattern::Fixed(_) => "fixed",
            SessionPattern::Glob(_) => "glob",
            SessionPattern::Regex(_) => "regex",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_pattern() {
        let pattern = SessionPattern::parse("cc-prod").unwrap();
        assert!(matches!(pattern, SessionPattern::Fixed(_)));
        assert!(pattern.matches("cc-prod"));
        assert!(!pattern.matches("cc-prod-2"));
        assert!(!pattern.matches("test-cc-prod"));
        assert!(!pattern.matches("CC-PROD")); // Case sensitive
    }

    #[test]
    fn test_glob_pattern_asterisk() {
        let pattern = SessionPattern::parse("test-*").unwrap();
        assert!(matches!(pattern, SessionPattern::Glob(_)));
        assert!(pattern.matches("test-1"));
        assert!(pattern.matches("test-abc"));
        assert!(pattern.matches("test-"));
        assert!(!pattern.matches("prod-test"));
        assert!(!pattern.matches("Test-1")); // Case sensitive
    }

    #[test]
    fn test_glob_pattern_question() {
        let pattern = SessionPattern::parse("cc-?-prod").unwrap();
        assert!(matches!(pattern, SessionPattern::Glob(_)));
        assert!(pattern.matches("cc-1-prod"));
        assert!(pattern.matches("cc-a-prod"));
        assert!(!pattern.matches("cc-12-prod")); // ? matches exactly one char
        assert!(!pattern.matches("cc--prod"));
    }

    #[test]
    fn test_glob_pattern_combined() {
        let pattern = SessionPattern::parse("*-test-?").unwrap();
        assert!(pattern.matches("abc-test-1"));
        assert!(pattern.matches("-test-x"));
        assert!(!pattern.matches("abc-test-12"));
    }

    #[test]
    fn test_regex_pattern() {
        let pattern = SessionPattern::parse("/^ssh-\\d+$/").unwrap();
        assert!(matches!(pattern, SessionPattern::Regex(_)));
        assert!(pattern.matches("ssh-123"));
        assert!(pattern.matches("ssh-0"));
        assert!(!pattern.matches("ssh-abc"));
        assert!(!pattern.matches("my-ssh-1"));
        assert!(!pattern.matches("ssh-123-extra"));
    }

    #[test]
    fn test_regex_pattern_partial() {
        // Regex without anchors matches anywhere
        let pattern = SessionPattern::parse("/test/").unwrap();
        assert!(pattern.matches("test"));
        assert!(pattern.matches("my-test-session"));
        assert!(pattern.matches("testing"));
    }

    #[test]
    fn test_regex_case_insensitive() {
        let pattern = SessionPattern::parse("/(?i)^prod$/").unwrap();
        assert!(pattern.matches("prod"));
        assert!(pattern.matches("PROD"));
        assert!(pattern.matches("Prod"));
    }

    #[test]
    fn test_invalid_regex() {
        let result = SessionPattern::parse("/[invalid/");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid regex"));
    }

    #[test]
    fn test_invalid_glob() {
        // "[invalid" without * or ? is treated as fixed string (valid)
        let result = SessionPattern::parse("[invalid");
        assert!(result.is_ok());

        // With * it becomes a glob, and "[" without closing "]" is invalid
        let result = SessionPattern::parse("*[invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid glob"));
    }

    #[test]
    fn test_edge_cases() {
        // Empty pattern
        let pattern = SessionPattern::parse("").unwrap();
        assert!(pattern.matches(""));
        assert!(!pattern.matches("anything"));

        // Single slash (not regex)
        let pattern = SessionPattern::parse("/").unwrap();
        assert!(matches!(pattern, SessionPattern::Fixed(_)));
        assert!(pattern.matches("/"));

        // Double slash (empty regex)
        let result = SessionPattern::parse("//");
        // Empty regex is valid and matches everything
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_type() {
        assert_eq!(
            SessionPattern::parse("exact").unwrap().pattern_type(),
            "fixed"
        );
        assert_eq!(
            SessionPattern::parse("glob-*").unwrap().pattern_type(),
            "glob"
        );
        assert_eq!(
            SessionPattern::parse("/regex/").unwrap().pattern_type(),
            "regex"
        );
    }
}
