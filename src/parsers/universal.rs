use regex::Regex;

use crate::agents::{AgentStatus, AgentType, ApprovalType, Subagent, SubagentStatus, SubagentType};
use crate::app::config::{AgentConfig, MatcherConfig};
use crate::parsers::{safe_tail, AgentParser, MatchStrength};

pub struct UniversalParser {
    config: AgentConfig,
    matchers: Vec<CompiledMatcher>,
    state_rules: Vec<CompiledStateRule>,
    subagent_rules: Option<CompiledSubagentRules>,
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
    refinements: Vec<CompiledRefinement>,
}

struct CompiledRefinement {
    group: String,
    re: Regex,
    status: String,
}

struct CompiledSubagentRules {
    start: Regex,
    running: Regex,
    complete: Regex,
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
                        });
                    }
                }
                state_rules.push(CompiledStateRule {
                    status: rule.status.clone(),
                    re,
                    approval_type: rule.approval_type.clone(),
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

        Self {
            config,
            matchers,
            state_rules,
            subagent_rules,
        }
    }
}

impl AgentParser for UniversalParser {
    fn agent_name(&self) -> &str {
        &self.config.name
    }

    fn agent_type(&self) -> AgentType {
        // Map ID to hardcoded type if it matches
        match self.config.id.as_str() {
            "claude" => AgentType::ClaudeCode,
            "gemini" => AgentType::GeminiCli,
            "opencode" => AgentType::OpenCode,
            "codex_cli" => AgentType::CodexCli,
            _ => AgentType::Custom(self.config.name.clone()),
        }
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
        let recent_content = safe_tail(content, 8192);

        for rule in &self.state_rules {
            if let Some(caps) = rule.re.captures(recent_content) {
                let mut status_str = rule.status.clone();
                let details = caps.name("details").map(|m| m.as_str().to_string());

                // Process refinements
                for refinement in &rule.refinements {
                    if let Some(group_match) = caps.name(&refinement.group) {
                        if refinement.re.is_match(group_match.as_str()) {
                            status_str = refinement.status.clone();
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
                        let approval_type = match rule.approval_type.as_deref() {
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
                    _ => {
                        return AgentStatus::Processing {
                            activity: rule.status.clone(),
                        };
                    }
                }
            }
        }

        if content.trim().is_empty() {
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
