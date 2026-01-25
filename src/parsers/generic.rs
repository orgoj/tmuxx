use regex::Regex;
use tracing::{debug, error};

use crate::agents::{AgentStatus, AgentType};
use crate::app::config::{AgentDefinition, MatchMode};
use crate::parsers::{AgentParser, safe_tail};

pub struct GenericAgentParser {
    definition: AgentDefinition,
    match_regexes: Vec<Regex>,
    state_regexes: Vec<(String, Regex, MatchMode)>,
}

impl GenericAgentParser {
    pub fn new(definition: AgentDefinition) -> Option<Self> {
        // Compile match regexes
        let mut match_regexes = Vec::new();
        for pattern in &definition.match_patterns {
            match Regex::new(pattern) {
                Ok(re) => match_regexes.push(re),
                Err(e) => {
                    error!("Invalid match pattern '{}' for agent '{}': {}", pattern, definition.name, e);
                    return None;
                }
            }
        }

        // Compile state regexes
        let mut state_regexes = Vec::new();
        for rule in &definition.state_rules {
            if rule.mode == MatchMode::Regex {
                match Regex::new(&rule.pattern) {
                    Ok(re) => state_regexes.push((rule.state.clone(), re, rule.mode)),
                    Err(e) => {
                        error!("Invalid state pattern '{}' for agent '{}': {}", rule.pattern, definition.name, e);
                        // Continue processing other rules? Or fail? Let's skip invalid ones.
                    }
                }
            } else {
                // For contains/ends_with, we don't strictly need a regex, but we store the pattern
                // We'll use a dummy regex or handle it differently.
                // Actually, to keep types consistent, let's just store the string in the regex field if we made a struct,
                // but since we are using tuples, let's just use an empty regex for non-regex modes or compile matching logic.
                // Simpler: use Regex for everything if possible, or build a unified matcher.
                // For "contains", regex is `Regex::new(&regex::escape(pattern))`
                // For "ends_with", regex is `Regex::new(&format!("{}$", regex::escape(pattern)))`
                
                let pattern_str = match rule.mode {
                    MatchMode::Contains => regex::escape(&rule.pattern),
                    MatchMode::EndsWith => format!("{}$", regex::escape(&rule.pattern)),
                    MatchMode::Regex => rule.pattern.clone(),
                };

                match Regex::new(&pattern_str) {
                    Ok(re) => state_regexes.push((rule.state.clone(), re, rule.mode)),
                    Err(e) => {
                        error!("Invalid generated pattern for rule '{}': {}", rule.pattern, e);
                    }
                }
            }
        }

        Some(Self {
            definition,
            match_regexes,
            state_regexes,
        })
    }
}

impl AgentParser for GenericAgentParser {
    fn agent_name(&self) -> &str {
        &self.definition.name
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Custom(self.definition.name.clone())
    }

    fn matches(&self, detection_strings: &[&str]) -> bool {
        for s in detection_strings {
            for re in &self.match_regexes {
                if re.is_match(s) {
                    return true;
                }
            }
        }
        false
    }

    fn parse_status(&self, content: &str) -> AgentStatus {
        let recent_content = safe_tail(content, 2000); 
        
        for (state_name, re, _) in &self.state_regexes {
            if re.is_match(recent_content) {
                return map_state_name_to_status(state_name);
            }
        }
        
        if content.trim().is_empty() {
            return AgentStatus::Idle;
        }

        AgentStatus::Processing { activity: "Processing".to_string() }
    }

    fn approval_keys(&self) -> &str {
        self.definition.approval_keys.as_deref().unwrap_or("y")
    }

    fn rejection_keys(&self) -> &str {
        self.definition.rejection_keys.as_deref().unwrap_or("n")
    }
}

fn map_state_name_to_status(name: &str) -> AgentStatus {
    use crate::agents::ApprovalType;
    match name.to_lowercase().as_str() {
        "idle" => AgentStatus::Idle,
        "processing" | "running" | "thinking" => AgentStatus::Processing { activity: "Processing".to_string() },
        "awaiting_input" | "input" | "ask" | "question" => AgentStatus::Idle, // Map input wait to Idle
        "awaiting_approval" | "confirm" | "approval" => AgentStatus::AwaitingApproval { 
            approval_type: ApprovalType::Other("Generic Approval".to_string()),
            details: "User confirmation required".to_string()
        },
        "error" | "failed" => AgentStatus::Error { message: "Error detected".to_string() },
        "completed" | "done" | "success" => AgentStatus::Idle,
        _ => {
            debug!("Unknown state name '{}', mapping to Processing", name);
            AgentStatus::Processing { activity: "Unknown State".to_string() }
        },
    }
}
