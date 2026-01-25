mod universal;

pub use universal::UniversalParser;

use crate::agents::{AgentStatus, AgentType, Subagent};
use crate::app::Config;
use crate::tmux::PaneInfo;

/// Safely get the last N characters of a string (handles multi-byte chars)
pub(crate) fn safe_tail(s: &str, max_chars: usize) -> &str {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s;
    }
    let skip = char_count - max_chars;
    let byte_idx = s.char_indices().nth(skip).map(|(idx, _)| idx).unwrap_or(0);
    &s[byte_idx..]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchStrength {
    None = 0,
    Weak = 1,   // Title-based match
    Strong = 2, // Command/Ancestor/Process-tree match
}

/// Trait for parsing agent output
pub trait AgentParser: Send + Sync {
    /// Returns the name of the agent
    fn agent_name(&self) -> &str;

    /// Returns the AgentType for this parser
    fn agent_type(&self) -> AgentType;

    /// Checks if any of the detection strings match this agent and with what strength
    fn match_strength(&self, detection_strings: &[&str]) -> MatchStrength;

    /// Legacy method for compatibility (calls match_strength)
    fn matches(&self, detection_strings: &[&str]) -> bool {
        self.match_strength(detection_strings) > MatchStrength::None
    }

    /// Parses the pane content and returns the agent status
    fn parse_status(&self, content: &str) -> AgentStatus;

    /// Parses subagents from the content (default: empty)
    fn parse_subagents(&self, content: &str) -> Vec<Subagent> {
        let _ = content;
        Vec::new()
    }

    /// Parses context remaining percentage from content (default: None)
    fn parse_context_remaining(&self, content: &str) -> Option<u8> {
        let _ = content;
        None
    }

    /// Returns the key(s) to send for approval
    fn approval_keys(&self) -> &str {
        "y"
    }

    /// Returns the key(s) to send for rejection
    fn rejection_keys(&self) -> &str {
        "n"
    }
}

/// Registry of all available parsers
pub struct ParserRegistry {
    parsers: Vec<Box<dyn AgentParser>>,
}

impl ParserRegistry {
    /// Creates a new registry with all default parsers
    pub fn new() -> Self {
        Self::with_config(&Config::default())
    }

    /// Creates a registry with custom patterns from config
    ///
    /// Built-in parsers (ClaudeCode, OpenCode, etc.) are registered first,
    /// so they take priority over custom patterns when multiple patterns match.
    pub fn with_config(config: &Config) -> Self {
        let mut parsers: Vec<Box<dyn AgentParser>> = Vec::new();
        // Initialize parsers from config agents
        for agent_config in &config.agents {
            parsers.push(Box::new(UniversalParser::new(agent_config.clone())));
        }

        Self { parsers }
    }

    /// Finds a parser that matches the given pane info
    pub fn find_parser_for_pane(&self, pane: &PaneInfo) -> Option<&dyn AgentParser> {
        let detection_strings = pane.detection_strings();

        // We want the HIGHEST MatchStrength, and within those, the HIGHEST priority (index in parsers list)
        // Note: self.parsers is already sorted by priority DESC.

        let mut best_parser: Option<&dyn AgentParser> = None;
        let mut best_strength = MatchStrength::None;

        for parser in &self.parsers {
            let strength = parser.match_strength(&detection_strings);
            if strength > best_strength {
                best_strength = strength;
                best_parser = Some(parser.as_ref());

                // Shortcut: if we have a Strong match and we are sorted by priority,
                // the first Strong match is the best.
                if strength == MatchStrength::Strong {
                    return best_parser;
                }
            }
        }

        best_parser
    }

    /// Returns all registered parsers
    pub fn all_parsers(&self) -> impl Iterator<Item = &dyn AgentParser> {
        self.parsers.iter().map(|p| p.as_ref())
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_registry() {
        let registry = ParserRegistry::new();

        // Test finding parsers with various detection strings
        let claude_pane = PaneInfo {
            session: "main".to_string(),
            window: 0,
            window_name: "code".to_string(),
            pane: 0,
            command: "node".to_string(),
            title: "Claude Code".to_string(),
            path: "/home/user/project".to_string(),
            pid: 1234,
            cmdline: "/usr/bin/claude".to_string(),
            child_commands: Vec::new(),
        };
        assert!(registry.find_parser_for_pane(&claude_pane).is_some());

        let opencode_pane = PaneInfo {
            session: "main".to_string(),
            window: 0,
            window_name: "code".to_string(),
            pane: 1,
            command: "opencode".to_string(),
            title: "".to_string(),
            path: "/home/user/project".to_string(),
            pid: 1235,
            cmdline: "opencode".to_string(),
            child_commands: Vec::new(),
        };
        assert!(registry.find_parser_for_pane(&opencode_pane).is_some());

        // Test detection via child processes
        let child_claude_pane = PaneInfo {
            session: "main".to_string(),
            window: 0,
            window_name: "code".to_string(),
            pane: 2,
            command: "zsh".to_string(),
            title: "~".to_string(),
            path: "/home/user/project".to_string(),
            pid: 1236,
            cmdline: "-zsh".to_string(),
            child_commands: vec!["claude -c".to_string(), "claude".to_string()],
        };
        assert!(registry.find_parser_for_pane(&child_claude_pane).is_some());
    }
}
