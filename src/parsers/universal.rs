use regex::Regex;

use crate::agents::{AgentStatus, AgentType, ApprovalType, Subagent, SubagentStatus, SubagentType};
use crate::app::config::{AgentConfig, MatcherConfig};
use crate::parsers::{safe_tail, AgentParser, MatchStrength};

pub struct UniversalParser {
    config: AgentConfig,
    matchers: Vec<CompiledMatcher>,
    state_rules: Vec<CompiledStateRule>,
    subagent_rules: Option<CompiledSubagentRules>,
    layout_rules: Option<CompiledLayoutRules>,
}

enum CompiledMatcher {
    Command(Regex),
    Ancestor(Regex),
    Title(Regex),
}

struct CompiledStateRule {
    status: String,
    re: Regex,
    approval_type: Option<String>,
    last_lines: Option<usize>,
    refinements: Vec<CompiledRefinement>,
}

struct CompiledRefinement {
    group: String,
    re: Regex,
    status: String,
    approval_type: Option<String>,
}

struct CompiledSubagentRules {
    start: Regex,
    running: Regex,
    complete: Regex,
}

struct CompiledLayoutRules {
    footer_separator: Option<Regex>,
    header_separator: Option<Regex>,
}

impl UniversalParser {
    pub fn new(config: AgentConfig) -> Self {
        let mut matchers = Vec::new();
        for m in &config.matchers {
            match m {
                MatcherConfig::Command { pattern } => {
                    if let Ok(re) = Regex::new(pattern) {
                        matchers.push(CompiledMatcher::Command(re));
                    }
                }
                MatcherConfig::Ancestor { pattern } => {
                    if let Ok(re) = Regex::new(pattern) {
                        matchers.push(CompiledMatcher::Ancestor(re));
                    }
                }
                MatcherConfig::Title { pattern } => {
                    if let Ok(re) = Regex::new(pattern) {
                        matchers.push(CompiledMatcher::Title(re));
                    }
                }
            }
        }

        let mut state_rules = Vec::new();
        for rule in &config.state_rules {
            if let Ok(re) = Regex::new(&rule.pattern) {
                let mut refinements = Vec::new();
                for r in &rule.refinements {
                    if let Ok(ref_re) = Regex::new(&r.pattern) {
                        refinements.push(CompiledRefinement {
                            group: r.group.clone(),
                            re: ref_re,
                            status: r.status.clone(),
                            approval_type: r.approval_type.clone(),
                        });
                    }
                }
                state_rules.push(CompiledStateRule {
                    status: rule.status.clone(),
                    re,
                    approval_type: rule.approval_type.clone(),
                    last_lines: rule.last_lines,
                    refinements,
                });
            }
        }

        let subagent_rules = config.subagent_rules.as_ref().and_then(|rules| {
            if let (Ok(start), Ok(running), Ok(complete)) = (
                Regex::new(&rules.start),
                Regex::new(&rules.running),
                Regex::new(&rules.complete),
            ) {
                Some(CompiledSubagentRules {
                    start,
                    running,
                    complete,
                })
            } else {
                None
            }
        });

        let layout_rules = config.layout.as_ref().map(|rules| CompiledLayoutRules {
            footer_separator: rules
                .footer_separator
                .as_ref()
                .and_then(|p| Regex::new(p).ok()),
            header_separator: rules
                .header_separator
                .as_ref()
                .and_then(|p| Regex::new(p).ok()),
        });

        Self {
            config,
            matchers,
            state_rules,
            subagent_rules,
            layout_rules,
        }
    }
    pub fn extract_body<'a>(&self, content: &'a str) -> &'a str {
        let mut start = 0;
        let mut end = content.len();

        if let Some(layout) = &self.layout_rules {
            // Apply Header Separator (skip everything before match)
            if let Some(re) = &layout.header_separator {
                if let Some(m) = re.find(content) {
                    start = m.end();
                }
            }

            // Apply Footer Separator (skip everything after match)
            if let Some(re) = &layout.footer_separator {
                let search_region = &content[start..];
                // Find the LAST match of the footer separator
                if let Some(m) = re.find_iter(search_region).last() {
                    end = start + m.start();
                }
            }
        }

        if start >= end {
            return "";
        }
        &content[start..end]
    }
}

impl AgentParser for UniversalParser {
    fn agent_name(&self) -> &str {
        &self.config.name
    }

    fn agent_color(&self) -> Option<&str> {
        self.config.color.as_deref()
    }

    fn agent_background_color(&self) -> Option<&str> {
        self.config.background_color.as_deref()
    }

    fn agent_type(&self) -> AgentType {
        // Universal parser always uses Custom type with name from config
        // This avoids hardcoding specific logic for specific agents in the code
        AgentType::Custom(self.config.name.clone())
    }

    fn match_strength(&self, detection_strings: &[&str]) -> MatchStrength {
        if detection_strings.len() < 3 {
            return MatchStrength::None;
        }

        let command = detection_strings[0];
        let title = detection_strings[1];
        let cmdline = detection_strings[2];

        let mut best_strength = MatchStrength::None;

        for matcher in &self.matchers {
            match matcher {
                CompiledMatcher::Command(re) => {
                    // Command matching: check current pane command/cmdline OR any child process
                    if re.is_match(command) || re.is_match(cmdline) {
                        best_strength = best_strength.max(MatchStrength::Strong);
                    }
                    // Also check children (everything after index 2)
                    for cmd in &detection_strings[3..] {
                        if re.is_match(cmd) {
                            best_strength = best_strength.max(MatchStrength::Strong);
                        }
                    }
                }
                CompiledMatcher::Title(re) => {
                    if re.is_match(title) {
                        best_strength = best_strength.max(MatchStrength::Weak);
                    }
                }
                CompiledMatcher::Ancestor(re) => {
                    // Ancestor matching: verify if any of the last few strings (ancestors) match
                    for cmd in &detection_strings[3..] {
                        if re.is_match(cmd) {
                            best_strength = best_strength.max(MatchStrength::Strong);
                        }
                    }
                }
            }
        }
        best_strength
    }

    fn matches(&self, detection_strings: &[&str]) -> bool {
        self.match_strength(detection_strings) > MatchStrength::None
    }

    fn parse_status(&self, content: &str) -> AgentStatus {
        // Look at a large enough chunk to see prompts and context.
        // For prompts anchored to the absolute end, we MUST include the end.
        let raw_content = safe_tail(content, 16384); // Increased buffer for safer context

        // 1. Isolate Body (strip header/footer if configured)
        let body_content = self.extract_body(raw_content);

        for rule in &self.state_rules {
            let search_content = if let Some(n) = rule.last_lines {
                // Efficiency: Only look at the last N lines
                let lines: Vec<&str> = body_content.lines().collect();
                if lines.len() > n {
                    let start_idx = lines.len() - n;
                    // Note: This is an approximation as it reconstructs the string,
                    // but for state rules it's usually exactly what's needed.
                    let suffix = lines[start_idx..].join("\n");
                    // Important: if the original body_content ended with a newline,
                    // join("\n") might lose it, so we peek back.
                    if body_content.ends_with('\n') && !suffix.ends_with('\n') {
                        let mut s = suffix;
                        s.push('\n');
                        s
                    } else {
                        suffix
                    }
                } else {
                    body_content.to_string()
                }
            } else {
                body_content.to_string()
            };

            if let Some(caps) = rule.re.captures(&search_content) {
                let mut status_str = rule.status.clone();
                let mut approval_type_override = None;
                let details = caps.name("details").map(|m| m.as_str().to_string());

                // Process refinements
                for refinement in &rule.refinements {
                    if let Some(group_match) = caps.name(&refinement.group) {
                        if refinement.re.is_match(group_match.as_str()) {
                            status_str = refinement.status.clone();
                            if refinement.approval_type.is_some() {
                                approval_type_override = refinement.approval_type.clone();
                            }
                            break;
                        }
                    }
                }

                match status_str.as_str() {
                    "processing" | "running" | "working" => {
                        return AgentStatus::Processing {
                            activity: details.unwrap_or_else(|| "Processing".to_string()),
                        };
                    }
                    "awaiting_approval" => {
                        let final_approval_type = approval_type_override
                            .as_deref()
                            .or(rule.approval_type.as_deref());
                        let approval_type = match final_approval_type {
                            Some("edit") => ApprovalType::FileEdit,
                            Some("create") => ApprovalType::FileCreate,
                            Some("delete") => ApprovalType::FileDelete,
                            Some("shell") => ApprovalType::ShellCommand,
                            Some("mcp") => ApprovalType::McpTool,
                            _ => ApprovalType::Other("Action Required".to_string()),
                        };
                        return AgentStatus::AwaitingApproval {
                            approval_type,
                            details: details.unwrap_or_default(),
                        };
                    }
                    "error" => {
                        return AgentStatus::Error {
                            message: details.unwrap_or_else(|| "Error detected".to_string()),
                        };
                    }
                    "awaiting_input" | "idle" => {
                        return AgentStatus::Idle;
                    }
                    s if s.starts_with("tui:") => {
                        return AgentStatus::Tui {
                            name: s[4..].to_string(),
                        };
                    }
                    _ => {
                        return AgentStatus::Processing {
                            activity: rule.status.clone(),
                        };
                    }
                }
            }
        }

        if body_content.trim().is_empty() {
            AgentStatus::Idle
        } else {
            let def = self.config.default_status.as_deref().unwrap_or("idle");
            match def {
                "processing" | "running" | "working" => AgentStatus::Processing {
                    activity: "Processing".to_string(),
                },
                _ => AgentStatus::Idle,
            }
        }
    }

    fn parse_subagents(&self, content: &str) -> Vec<Subagent> {
        let rules = match &self.subagent_rules {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut subagents = Vec::new();
        let mut id_counter = 0;

        // Task starts
        for cap in rules.start.captures_iter(content) {
            let type_name = cap.get(1).map(|m| m.as_str()).unwrap_or("Task");
            let desc = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            id_counter += 1;
            subagents.push(Subagent::new(
                format!("{}-{}", self.config.id, id_counter),
                SubagentType::parse(type_name),
                desc.to_string(),
            ));
        }

        // Running
        for cap in rules.running.captures_iter(content) {
            let type_name = &cap[1];
            let desc = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let existing = subagents
                .iter()
                .any(|s| s.subagent_type.display_name().to_lowercase() == type_name.to_lowercase());
            if !existing {
                id_counter += 1;
                subagents.push(Subagent::new(
                    format!("{}-{}", self.config.id, id_counter),
                    SubagentType::parse(type_name),
                    desc.to_string(),
                ));
            }
        }

        // Complete
        for cap in rules.complete.captures_iter(content) {
            let type_name = &cap[1];
            for subagent in &mut subagents {
                if subagent.subagent_type.display_name().to_lowercase() == type_name.to_lowercase()
                {
                    subagent.status = SubagentStatus::Completed;
                }
            }
        }

        subagents
    }

    fn approval_keys(&self) -> &str {
        self.config.keys.approve.as_deref().unwrap_or("y")
    }

    fn rejection_keys(&self) -> &str {
        self.config.keys.reject.as_deref().unwrap_or("n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::AgentStatus;
    use crate::app::config::{AgentConfig, AgentKeys};

    #[test]
    fn test_parse_status_with_footer_stripping() {
        let config = AgentConfig {
            id: "claude".to_string(),
            name: "Claude".to_string(),
            color: Some("magenta".to_string()),
            background_color: None,
            priority: 100,
            default_status: Some("idle".to_string()),
            matchers: vec![],
            state_rules: vec![crate::app::config::StateRule {
                status: "processing".to_string(),
                // Updated regex to handle boxed input and multi-line capture
                pattern: r"(?ms)(?P<indicator>.*)\n[ \t]*[│]?─{10,}.*?\n[ \t]*[│]?.*❯[^\n]*\s*$"
                    .to_string(),
                approval_type: None,
                refinements: vec![crate::app::config::Refinement {
                    group: "indicator".to_string(),
                    pattern: "Baked".to_string(),
                    status: "idle".to_string(),
                    approval_type: None,
                }],
                last_lines: Some(0),
            }],
            title_indicators: None,
            keys: AgentKeys::default(),
            subagent_rules: None,
            layout: Some(crate::app::config::LayoutConfig {
                // Handle optional │ prefix
                footer_separator: Some(r"(?m)^[ \t]*[│]?─{10,}.*?$".to_string()),
                header_separator: None,
            }),
        };

        let parser = UniversalParser::new(config);

        // Simulated output from ct-test (Boxed style)
        let content = "\
Some previous content...
│  Remaining Work (9 tests)                                                                                        │
│                                                                                                                  │
│✻ Baked for 24m 53s                                                                                               │
│                                                                                                                  │
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
❯ 
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  Model: Sonnet 4.5 | Style: default | Ctx: 0 | Ctx(u): 0.0% | Session: 24m                                      
  ⏵⏵ accept edits on (shift+Tab to cycle) · 7 files +112 -62                                                     
";

        // Verify the status detection
        let status = parser.parse_status(content);

        // Should catch 'Baked' in the indicator group
        match status {
            AgentStatus::Idle => {
                // Success!
            }
            status => panic!("Expected Idle status, got {:?}", status),
        }
    }
}
