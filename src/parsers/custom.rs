use crate::agents::{AgentStatus, AgentType, Subagent};
use crate::parsers::AgentParser;
use regex::Regex;

/// Custom agent parser based on user-configured patterns
///
/// This parser allows users to define their own detection patterns via config.
/// Supports wildcard "*" pattern to match all panes.
pub struct CustomAgentParser {
    pattern: Regex,
    agent_type_name: String,
}

impl CustomAgentParser {
    /// Creates a new CustomAgentParser from a pattern string and agent type name
    ///
    /// # Arguments
    /// * `pattern_str` - Regex pattern or "*" for wildcard (matches everything)
    /// * `agent_type` - Display name for this agent type
    ///
    /// Returns None if the pattern is invalid regex
    pub fn new(pattern_str: &str, agent_type: &str) -> Option<Self> {
        // Support wildcard "*" as match-all pattern
        let regex_pattern = if pattern_str == "*" {
            ".*".to_string() // Match everything
        } else {
            pattern_str.to_string()
        };

        Regex::new(&regex_pattern).ok().map(|pattern| Self {
            pattern,
            agent_type_name: agent_type.to_string(),
        })
    }
}

impl AgentParser for CustomAgentParser {
    fn agent_name(&self) -> &str {
        &self.agent_type_name
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Custom(self.agent_type_name.clone())
    }

    fn matches(&self, detection_strings: &[&str]) -> bool {
        // Match against ANY detection string (command, title, cmdline, children)
        // This allows flexible matching against various process attributes
        detection_strings.iter().any(|s| self.pattern.is_match(s))
    }

    fn parse_status(&self, _content: &str) -> AgentStatus {
        // Custom agents default to Idle status
        // Could be enhanced in the future to parse custom status patterns
        AgentStatus::Idle
    }

    fn parse_subagents(&self, _content: &str) -> Vec<Subagent> {
        // Custom agents don't track subagents by default
        Vec::new()
    }

    fn parse_context_remaining(&self, _content: &str) -> Option<u8> {
        // Custom agents don't track context
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_pattern() {
        let parser = CustomAgentParser::new("*", "All Panes").unwrap();
        assert!(parser.matches(&["node"]));
        assert!(parser.matches(&["bash"]));
        assert!(parser.matches(&["anything"]));
    }

    #[test]
    fn test_specific_pattern() {
        let parser = CustomAgentParser::new("node", "Node Agent").unwrap();
        assert!(parser.matches(&["node"]));
        assert!(parser.matches(&["/usr/bin/node"]));
        assert!(!parser.matches(&["bash"]));
    }

    #[test]
    fn test_regex_pattern() {
        let parser = CustomAgentParser::new("python.*", "Python Agent").unwrap();
        assert!(parser.matches(&["python3"]));
        assert!(parser.matches(&["python3.11"]));
        assert!(!parser.matches(&["node"]));
    }

    #[test]
    fn test_invalid_regex() {
        let parser = CustomAgentParser::new("[invalid(regex", "Bad");
        assert!(parser.is_none());
    }

    #[test]
    fn test_matches_any_detection_string() {
        let parser = CustomAgentParser::new("claude", "Claude").unwrap();
        // Should match if ANY detection string matches
        assert!(parser.matches(&["bash", "/usr/bin/claude", "~"]));
        assert!(!parser.matches(&["bash", "/usr/bin/node", "~"]));
    }
}
