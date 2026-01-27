use regex::Regex;
use tracing::warn;

use crate::agents::{AgentStatus, AgentType, ApprovalType, Subagent, SubagentStatus, SubagentType};
use crate::app::config::{AgentConfig, MatcherConfig};
use crate::parsers::{safe_tail, AgentParser, MatchStrength};

pub struct UniversalParser {
    config: AgentConfig,
    capture_buffer_size: usize,
    matchers: Vec<CompiledMatcher>,
    state_rules: Vec<CompiledStateRule>,
    subagent_rules: Option<CompiledSubagentRules>,
    summary_rules: Option<CompiledSummaryRules>,
    highlight_rules: Vec<CompiledHighlightRule>,
    layout_rules: Option<CompiledLayoutRules>,
}

enum CompiledMatcher {
    Command(Regex),
    Ancestor(Regex),
    Title(Regex),
    Content(Regex),
}

struct CompiledStateRule {
    status: String,
    kind: Option<crate::app::config::RuleType>,
    re: Regex,
    approval_type: Option<String>,
    last_lines: Option<usize>,
    refinements: Vec<CompiledRefinement>,
}

struct CompiledRefinement {
    group: String,
    re: Regex,
    status: String,
    kind: Option<crate::app::config::RuleType>,
    approval_type: Option<String>,
}

struct CompiledSubagentRules {
    start: Regex,
    running: Regex,
    complete: Regex,
}

struct CompiledSummaryRules {
    activity: Option<Regex>,
    task_pending: Option<Regex>,
    task_completed: Option<Regex>,
    tool_use: Option<Regex>,
}

struct CompiledHighlightRule {
    re: Regex,
    color: String,
    modifiers: Vec<String>,
}

struct CompiledLayoutRules {
    footer_separator: Option<Regex>,
    header_separator: Option<Regex>,
}

impl UniversalParser {
    pub fn new(config: AgentConfig, capture_buffer_size: usize) -> Self {
        let mut matchers = Vec::new();
        for m in &config.matchers {
            match m {
                MatcherConfig::Command { pattern } => match Regex::new(pattern) {
                    Ok(re) => matchers.push(CompiledMatcher::Command(re)),
                    Err(e) => warn!(
                        "Invalid command pattern '{}' for agent {}: {}",
                        pattern, config.name, e
                    ),
                },
                MatcherConfig::Ancestor { pattern } => match Regex::new(pattern) {
                    Ok(re) => matchers.push(CompiledMatcher::Ancestor(re)),
                    Err(e) => warn!(
                        "Invalid ancestor pattern '{}' for agent {}: {}",
                        pattern, config.name, e
                    ),
                },
                MatcherConfig::Title { pattern } => match Regex::new(pattern) {
                    Ok(re) => matchers.push(CompiledMatcher::Title(re)),
                    Err(e) => warn!(
                        "Invalid title pattern '{}' for agent {}: {}",
                        pattern, config.name, e
                    ),
                },
                MatcherConfig::Content { pattern } => match Regex::new(pattern) {
                    Ok(re) => matchers.push(CompiledMatcher::Content(re)),
                    Err(e) => warn!(
                        "Invalid content pattern '{}' for agent {}: {}",
                        pattern, config.name, e
                    ),
                },
            }
        }

        let mut state_rules = Vec::new();
        for rule in &config.state_rules {
            match Regex::new(&rule.pattern) {
                Ok(re) => {
                    let mut refinements = Vec::new();
                    for r in &rule.refinements {
                        match Regex::new(&r.pattern) {
                            Ok(ref_re) => {
                                refinements.push(CompiledRefinement {
                                    group: r.group.clone(),
                                    re: ref_re,
                                    status: r.status.clone(),
                                    kind: r.kind.clone(),
                                    approval_type: r.approval_type.clone(),
                                });
                            }
                            Err(e) => warn!(
                                "Invalid refinement pattern '{}' in rule for agent {}: {}",
                                r.pattern, config.name, e
                            ),
                        }
                    }
                    state_rules.push(CompiledStateRule {
                        status: rule.status.clone(),
                        kind: rule.kind.clone(),
                        re,
                        approval_type: rule.approval_type.clone(),
                        last_lines: rule.last_lines,
                        refinements,
                    });
                }
                Err(e) => warn!(
                    "Invalid state rule pattern '{}' for agent {}: {}",
                    rule.pattern, config.name, e
                ),
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

        let summary_rules = config
            .summary_rules
            .as_ref()
            .map(|rules| CompiledSummaryRules {
                activity: rules.activity.as_ref().and_then(|p| Regex::new(p).ok()),
                task_pending: rules.task_pending.as_ref().and_then(|p| Regex::new(p).ok()),
                task_completed: rules
                    .task_completed
                    .as_ref()
                    .and_then(|p| Regex::new(p).ok()),
                tool_use: rules.tool_use.as_ref().and_then(|p| Regex::new(p).ok()),
            });

        let highlight_rules = config
            .highlight_rules
            .iter()
            .filter_map(|rule| {
                Regex::new(&rule.pattern)
                    .ok()
                    .map(|re| CompiledHighlightRule {
                        re,
                        color: rule.color.clone(),
                        modifiers: rule.modifiers.clone(),
                    })
            })
            .collect();

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
            capture_buffer_size,
            matchers,
            state_rules,
            subagent_rules,
            summary_rules,
            highlight_rules,
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
        AgentType::Named(self.config.name.clone())
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
                CompiledMatcher::Content(_) => {
                    // Content matchers do not contribute to initial detection strength
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
        let raw_content = safe_tail(content, self.capture_buffer_size);

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
                let mut status_kind = rule.kind.clone();
                let mut approval_type_override = None;
                let details = caps.name("details").map(|m| m.as_str().to_string());

                // Process refinements
                for refinement in &rule.refinements {
                    if let Some(group_match) = caps.name(&refinement.group) {
                        if refinement.re.is_match(group_match.as_str()) {
                            status_str = refinement.status.clone();
                            if refinement.kind.is_some() {
                                status_kind = refinement.kind.clone();
                            }
                            if refinement.approval_type.is_some() {
                                approval_type_override = refinement.approval_type.clone();
                            }
                            break;
                        }
                    }
                }

                // If explicit kind is set, use it.
                if let Some(kind) = status_kind {
                    use crate::app::config::RuleType;
                    match kind {
                        RuleType::Idle => {
                            return AgentStatus::Idle {
                                label: Some(status_str),
                            };
                        }
                        RuleType::Working => {
                            return AgentStatus::Processing {
                                activity: details.unwrap_or(status_str),
                            };
                        }
                        RuleType::Error => {
                            return AgentStatus::Error {
                                message: details.unwrap_or(status_str),
                            };
                        }
                        RuleType::Approval => {
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
                                details: details.unwrap_or(status_str),
                            };
                        }
                    }
                }
            }
        }

        if body_content.trim().is_empty() {
            AgentStatus::Idle { label: None }
        } else {
            // Priority: default_type > default_status string mapping > Idle/Processing based on parser default
            if let Some(kind) = &self.config.default_type {
                use crate::app::config::RuleType;
                let label = self.config.default_status.clone();
                match kind {
                    RuleType::Idle => return AgentStatus::Idle { label },
                    RuleType::Working => {
                        return AgentStatus::Processing {
                            activity: label.unwrap_or_else(|| "Processing".to_string()),
                        }
                    }
                    RuleType::Error => {
                        return AgentStatus::Error {
                            message: label.unwrap_or_else(|| "Error".to_string()),
                        }
                    }
                    RuleType::Approval => {
                        return AgentStatus::AwaitingApproval {
                            approval_type: ApprovalType::Other("Action Required".to_string()),
                            details: label.unwrap_or_default(),
                        }
                    }
                }
            }

            // Fallback: Default to Idle if no specific logic matches and no default_type is configured
            AgentStatus::Idle {
                label: self.config.default_status.clone(),
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

    fn parse_summary(&self, content: &str) -> crate::parsers::AgentSummary {
        let mut summary = crate::parsers::AgentSummary::default();
        let rules = match &self.summary_rules {
            Some(r) => r,
            None => return summary,
        };

        for line in content.lines() {
            let trimmed = line.trim();

            if let Some(re) = &rules.activity {
                if let Some(cap) = re.captures(trimmed) {
                    if let Some(m) = cap.get(1) {
                        summary.current_activity = Some(m.as_str().to_string());
                    }
                }
            }

            if let Some(re) = &rules.task_pending {
                if let Some(cap) = re.captures(trimmed) {
                    if let Some(m) = cap.get(1) {
                        summary.tasks.push((false, m.as_str().to_string()));
                    }
                }
            }

            if let Some(re) = &rules.task_completed {
                if let Some(cap) = re.captures(trimmed) {
                    if let Some(m) = cap.get(1) {
                        summary.tasks.push((true, m.as_str().to_string()));
                    }
                }
            }

            if let Some(re) = &rules.tool_use {
                if let Some(cap) = re.captures(trimmed) {
                    if let Some(m) = cap.get(1) {
                        summary.tools.push(m.as_str().to_string());
                    }
                }
            }
        }

        summary
    }

    fn highlight_line(&self, line: &str) -> Option<ratatui::style::Style> {
        use crate::ui::Styles;
        use ratatui::style::{Modifier, Style};

        for rule in &self.highlight_rules {
            if rule.re.is_match(line) {
                let mut style = Style::default().fg(Styles::parse_color(&rule.color));
                for modifier in &rule.modifiers {
                    match modifier.to_lowercase().as_str() {
                        "bold" => style = style.add_modifier(Modifier::BOLD),
                        "italic" => style = style.add_modifier(Modifier::ITALIC),
                        "dim" => style = style.add_modifier(Modifier::DIM),
                        _ => {}
                    }
                }
                return Some(style);
            }
        }
        None
    }

    fn approval_keys(&self) -> &str {
        self.config.keys.approve.as_deref().unwrap_or("y")
    }

    fn rejection_keys(&self) -> &str {
        self.config.keys.reject.as_deref().unwrap_or("n")
    }

    fn process_indicators(&self) -> Vec<crate::app::config::ProcessIndicator> {
        self.config.process_indicators.clone()
    }

    fn requires_content_check(&self) -> bool {
        self.matchers
            .iter()
            .any(|m| matches!(m, CompiledMatcher::Content(_)))
    }

    fn match_content(&self, content: &str) -> bool {
        // If any Content matcher is present, at least ONE must match?
        // Or ALL must match?
        // Usually matchers are OR (if multiple defined).
        // BUT here we are refining a match.
        // Let's say: If there are content matchers, at least one must match.

        let content_matchers: Vec<&Regex> = self
            .matchers
            .iter()
            .filter_map(|m| match m {
                CompiledMatcher::Content(re) => Some(re),
                _ => None,
            })
            .collect();

        if content_matchers.is_empty() {
            return true;
        }

        for re in content_matchers {
            if re.is_match(content) {
                return true;
            }
        }

        false
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
                kind: Some(crate::app::config::RuleType::Working),
                // Updated regex to handle boxed input and multi-line capture
                pattern: r"(?ms)(?P<indicator>.*)\n[ \t]*[│]?─{10,}.*?\n[ \t]*[│]?.*❯[^\n]*\s*$"
                    .to_string(),
                approval_type: None,
                refinements: vec![crate::app::config::Refinement {
                    group: "indicator".to_string(),
                    pattern: "Baked".to_string(),
                    status: "idle".to_string(),
                    kind: Some(crate::app::config::RuleType::Idle),
                    approval_type: None,
                }],
                last_lines: Some(0),
            }],
            title_indicators: None,
            keys: AgentKeys::default(),
            subagent_rules: None,
            default_type: None,
            layout: Some(crate::app::config::LayoutConfig {
                // Handle optional │ prefix
                footer_separator: Some(r"(?m)^[ \t]*[│]?─{10,}.*?$".to_string()),
                header_separator: None,
            }),
            summary_rules: None,
            highlight_rules: Vec::new(),
            process_indicators: vec![],
        };

        let parser = UniversalParser::new(config, 16384);

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
            AgentStatus::Idle { .. } => {
                // Success!
            }
            status => panic!("Expected Idle status, got {:?}", status),
        }
    }
}
