use regex::Regex;
use tracing::warn;

use crate::agents::{AgentStatus, AgentType, ApprovalType, Subagent};
use crate::app::config::{AgentConfig, HighlightRule, MatcherConfig};
use crate::parsers::{safe_tail, AgentParser, AgentSummary, MatchStrength};

/// Split content on structural separator area (the Claude/Pi prompt sandwich)
/// This looks from the bottom and identifies the start of the UI chrome.
fn split_on_separator_line(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (content.to_string(), String::new());
    }

    // Search from the bottom for the LAST separator line that marks the prompt area
    for i in (0..lines.len()).rev() {
        let trimmed = lines[i].trim();
        if trimmed.len() >= 40 && trimmed.chars().all(|c| c == '─') {
            // We found a separator. Now we need to find the TOP of the prompt sandwich.
            let mut split_idx = i;
            let mut j = i;
            // Look up to 3 lines above to find the start of a small prompt sandwich
            while j > 0 && j > i.saturating_sub(3) {
                j -= 1;
                let upper_trimmed = lines[j].trim();
                if upper_trimmed.len() >= 40 && upper_trimmed.chars().all(|c| c == '─') {
                    split_idx = j;
                }
            }

            let body = lines[..split_idx].join("\n");
            let prompt = lines[split_idx..].join("\n");
            return (body, prompt);
        }
    }
    (content.to_string(), String::new())
}

/// Split content on last ╭─ powerline box start
fn split_on_powerline(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();
    for i in (0..lines.len()).rev() {
        if lines[i].starts_with("╭─") {
            let body = lines[..i].join("\n");
            let prompt = lines[i..].join("\n");
            return (body, prompt);
        }
    }
    (content.to_string(), String::new())
}

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

/// Built-in splitter types
#[derive(Debug, Clone, PartialEq, Eq)]
enum Splitter {
    /// No splitter, use regex pattern as-is
    None,
    /// Split on first line that is only ─ characters (40+)
    SeparatorLine,
    /// Split on ╭─ powerline box start
    PowerlineBox,
}

struct CompiledStateRule {
    status: String,
    kind: Option<crate::app::config::RuleType>,
    re: Option<Regex>,
    splitter: Splitter,
    approval_type: Option<String>,
    last_lines: Option<usize>,
    refinements: Vec<CompiledRefinement>,
}

/// Where to apply the refinement pattern
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum MatchLocation {
    #[default]
    Anywhere,
    LastLine,
    LastBlock,
    FirstLineOfLastBlock,
}

struct CompiledRefinement {
    group: String,
    re: Regex,
    status: String,
    kind: Option<crate::app::config::RuleType>,
    approval_type: Option<String>,
    location: MatchLocation,
}

struct CompiledSubagentRules {
    start: Regex,
    _running: Regex,
    _complete: Regex,
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
    pub fn new(config: AgentConfig, capture_buffer_size: usize, global_rules: &[HighlightRule]) -> Self {
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
            // Parse splitter
            let splitter = match rule.splitter.as_deref() {
                Some("separator_line") => Splitter::SeparatorLine,
                Some("powerline_box") => Splitter::PowerlineBox,
                _ => Splitter::None,
            };

            // Compile regex pattern (optional if splitter is used)
            let re = if rule.pattern.is_empty() {
                None
            } else {
                match Regex::new(&rule.pattern) {
                    Ok(r) => Some(r),
                    Err(e) => {
                        warn!(
                            "Invalid state rule pattern '{}' for agent {}: {}",
                            rule.pattern, config.name, e
                        );
                        continue;
                    }
                }
            };

            // Compile refinements
            let mut refinements = Vec::new();
            for r in &rule.refinements {
                match Regex::new(&r.pattern) {
                    Ok(ref_re) => {
                        let location = match r.location.as_deref() {
                            Some("last_line") => MatchLocation::LastLine,
                            Some("last_block") => MatchLocation::LastBlock,
                            Some("first_line_of_last_block") => MatchLocation::FirstLineOfLastBlock,
                            _ => MatchLocation::Anywhere,
                        };
                        refinements.push(CompiledRefinement {
                            group: r.group.clone(),
                            re: ref_re,
                            status: r.status.clone(),
                            kind: r.kind.clone(),
                            approval_type: r.approval_type.clone(),
                            location,
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
                splitter,
                approval_type: rule.approval_type.clone(),
                last_lines: rule.last_lines,
                refinements,
            });
        }

        let subagent_rules = config.subagent_rules.as_ref().and_then(|rules| {
            if let (Ok(start), Ok(running), Ok(complete)) = (
                Regex::new(&rules.start),
                Regex::new(&rules.running),
                Regex::new(&rules.complete),
            ) {
                Some(CompiledSubagentRules {
                    start,
                    _running: running,
                    _complete: complete,
                })
            } else {
                warn!("Invalid subagent rules for agent {}", config.name);
                None
            }
        });

        let summary_rules = config.summary_rules.as_ref().map(|rules| {
            let activity = rules.activity.as_ref().and_then(|p| Regex::new(p).ok());
            let task_pending = rules.task_pending.as_ref().and_then(|p| Regex::new(p).ok());
            let task_completed = rules
                .task_completed
                .as_ref()
                .and_then(|p| Regex::new(p).ok());
            let tool_use = rules.tool_use.as_ref().and_then(|p| Regex::new(p).ok());
            CompiledSummaryRules {
                activity,
                task_pending,
                task_completed,
                tool_use,
            }
        });

        let mut highlight_rules: Vec<CompiledHighlightRule> = config
            .highlight_rules
            .iter()
            .filter_map(|r| {
                Regex::new(&r.pattern).ok().map(|re| CompiledHighlightRule {
                    re,
                    color: r.color.clone(),
                    modifiers: r.modifiers.clone(),
                })
            })
            .collect();

        // Add global rules as fallback
        for r in global_rules {
            if let Ok(re) = Regex::new(&r.pattern) {
                highlight_rules.push(CompiledHighlightRule {
                    re,
                    color: r.color.clone(),
                    modifiers: r.modifiers.clone(),
                });
            }
        }

        let layout_rules = config.layout.as_ref().map(|l| CompiledLayoutRules {
            footer_separator: l.footer_separator.as_ref().and_then(|p| Regex::new(p).ok()),
            header_separator: l.header_separator.as_ref().and_then(|p| Regex::new(p).ok()),
        });

        UniversalParser {
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
            if let Some(re) = &layout.header_separator {
                if let Some(m) = re.find(content) {
                    start = m.end();
                }
            }
            if let Some(re) = &layout.footer_separator {
                let search_region = &content[start..];
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

    fn agent_id(&self) -> &str {
        &self.config.id
    }

    fn agent_type(&self) -> AgentType {
        AgentType::Named(self.config.id.clone())
    }

    fn agent_color(&self) -> Option<&str> {
        self.config.color.as_deref()
    }

    fn agent_background_color(&self) -> Option<&str> {
        self.config.background_color.as_deref()
    }

    fn match_strength(&self, detection_strings: &[&str]) -> MatchStrength {
        let mut best_strength = MatchStrength::None;
        for matcher in &self.matchers {
            match matcher {
                CompiledMatcher::Command(re) => {
                    if re.is_match(detection_strings[0]) {
                        best_strength = best_strength.max(MatchStrength::Strong);
                    }
                }
                CompiledMatcher::Title(re) => {
                    let title = detection_strings[2];
                    if re.is_match(title) {
                        best_strength = best_strength.max(MatchStrength::Weak);
                    }
                }
                CompiledMatcher::Ancestor(re) => {
                    for cmd in &detection_strings[3..] {
                        if re.is_match(cmd) {
                            best_strength = best_strength.max(MatchStrength::Strong);
                        }
                    }
                }
                CompiledMatcher::Content(re) => {
                    let content = detection_strings[1];
                    if re.is_match(content) {
                        best_strength = best_strength.max(MatchStrength::Strong);
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
        let raw_content = safe_tail(content, self.capture_buffer_size);
        let body_content = self.extract_body(raw_content);

        for rule in &self.state_rules {
            let search_content = if let Some(n) = rule.last_lines {
                let lines: Vec<&str> = body_content.lines().collect();
                if lines.len() > n {
                    let start_idx = lines.len() - n;
                    let suffix = lines[start_idx..].join("\n");
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

            // Apply splitter
            let (body_group, prompt_group) = match &rule.splitter {
                Splitter::SeparatorLine => split_on_separator_line(&search_content),
                Splitter::PowerlineBox => split_on_powerline(&search_content),
                Splitter::None => {
                    if let Some(ref re) = rule.re {
                        if let Some(caps) = re.captures(&search_content) {
                            let body = caps
                                .name("body")
                                .map(|m| m.as_str())
                                .unwrap_or(&search_content);
                            let prompt = caps.name("prompt").map(|m| m.as_str()).unwrap_or("");
                            (body.to_string(), prompt.to_string())
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            };

            let mut status_str = rule.status.clone();
            let mut status_kind = rule.kind.clone();
            let mut approval_type_override = None;
            let mut matched = false;

            if rule.splitter != Splitter::None || rule.re.is_some() {
                matched = true;
            }

            for refinement in &rule.refinements {
                let target_text = if refinement.group == "prompt" {
                    &prompt_group
                } else {
                    &body_group
                };
                let match_text = match &refinement.location {
                    MatchLocation::LastLine => target_text
                        .lines()
                        .rev()
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or(""),
                    MatchLocation::LastBlock => {
                        if let Some(pos) = target_text.rfind("\n\n") {
                            &target_text[pos + 2..]
                        } else {
                            target_text
                        }
                    }
                    MatchLocation::FirstLineOfLastBlock => {
                        let block = if let Some(pos) = target_text.rfind("\n\n") {
                            &target_text[pos + 2..]
                        } else {
                            target_text
                        };
                        block.lines().find(|l| !l.trim().is_empty()).unwrap_or("")
                    }
                    MatchLocation::Anywhere => target_text,
                };

                if refinement.re.is_match(match_text) {
                    status_str = refinement.status.clone();
                    if refinement.kind.is_some() {
                        status_kind = refinement.kind.clone();
                    }
                    if refinement.approval_type.is_some() {
                        approval_type_override = refinement.approval_type.clone();
                    }
                    matched = true;
                    break;
                }
            }

            if !matched {
                continue;
            }

            if let Some(kind) = status_kind {
                use crate::app::config::RuleType;
                match kind {
                    RuleType::Idle => {
                        return AgentStatus::Idle {
                            label: Some(status_str),
                        }
                    }
                    RuleType::Working => {
                        return AgentStatus::Processing {
                            activity: status_str,
                        }
                    }
                    RuleType::Error => {
                        return AgentStatus::Error {
                            message: status_str,
                        }
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
                            details: status_str,
                        };
                    }
                }
            }
        }

        if body_content.trim().is_empty() {
            AgentStatus::Idle { label: None }
        } else {
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
                            details: label.unwrap_or_else(|| "Action Required".to_string()),
                        }
                    }
                }
            }
            AgentStatus::Processing {
                activity: "Processing".to_string(),
            }
        }
    }

    fn parse_subagents(&self, content: &str) -> Vec<Subagent> {
        let mut subagents = Vec::new();
        if let Some(rules) = &self.subagent_rules {
            for caps in rules.start.captures_iter(content) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                let desc = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                subagents.push(Subagent::new(
                    name.to_string(),
                    crate::agents::SubagentType::parse(name),
                    desc.to_string(),
                ));
            }
        }
        subagents
    }

    fn parse_summary(&self, content: &str) -> AgentSummary {
        let mut summary = AgentSummary::default();

        if let Some(rules) = &self.summary_rules {
            let last_chunk = safe_tail(content, 4000);
            if let Some(re) = &rules.activity {
                if let Some(caps) = re.captures_iter(last_chunk).last() {
                    summary.current_activity = caps.get(1).map(|m| m.as_str().to_string());
                }
            }
            if let Some(re) = &rules.task_pending {
                for caps in re.captures_iter(last_chunk) {
                    summary.tasks.push((
                        false,
                        caps.get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    ));
                }
            }
            if let Some(re) = &rules.task_completed {
                for caps in re.captures_iter(last_chunk) {
                    summary.tasks.push((
                        true,
                        caps.get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    ));
                }
            }
            if let Some(re) = &rules.tool_use {
                for caps in re.captures_iter(last_chunk) {
                    summary.tools.push(
                        caps.get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    );
                }
            }
        }
        summary
    }

    fn explain_status(&self, content: &str) -> Option<String> {
        let raw_content = safe_tail(content, self.capture_buffer_size);
        let body_content = self.extract_body(raw_content);
        let mut explanation = String::new();
        explanation.push_str("\n--- DEBUG EXPLANATION ---\n");
        explanation.push_str(&format!(
            "Body content extracted (length: {}):\n",
            body_content.len()
        ));
        explanation.push_str("------------------\n");
        explanation.push_str(body_content);
        explanation.push_str("\n------------------\n");

        for (idx, rule) in self.state_rules.iter().enumerate() {
            let search_content = if let Some(n) = rule.last_lines {
                let lines: Vec<&str> = body_content.lines().collect();
                if lines.len() > n {
                    let start_idx = lines.len() - n;
                    lines[start_idx..].join("\n")
                } else {
                    body_content.to_string()
                }
            } else {
                body_content.to_string()
            };

            let (body_group, prompt_group) = match &rule.splitter {
                Splitter::SeparatorLine => split_on_separator_line(&search_content),
                Splitter::PowerlineBox => split_on_powerline(&search_content),
                Splitter::None => {
                    if let Some(ref re) = rule.re {
                        if let Some(caps) = re.captures(&search_content) {
                            let body = caps
                                .name("body")
                                .map(|m| m.as_str())
                                .unwrap_or(&search_content);
                            let prompt = caps.name("prompt").map(|m| m.as_str()).unwrap_or("");
                            (body.to_string(), prompt.to_string())
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            };

            explanation.push_str(&format!(
                "MATCHED Rule #{} (status: {}, splitter: {:?})\n",
                idx, rule.status, rule.splitter
            ));

            for (r_idx, refinement) in rule.refinements.iter().enumerate() {
                let target_text = if refinement.group == "prompt" {
                    &prompt_group
                } else {
                    &body_group
                };
                let match_text = match &refinement.location {
                    MatchLocation::LastLine => target_text
                        .lines()
                        .rev()
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or(""),
                    MatchLocation::LastBlock => {
                        if let Some(pos) = target_text.rfind("\n\n") {
                            &target_text[pos + 2..]
                        } else {
                            target_text
                        }
                    }
                    MatchLocation::FirstLineOfLastBlock => {
                        let block = if let Some(pos) = target_text.rfind("\n\n") {
                            &target_text[pos + 2..]
                        } else {
                            target_text
                        };
                        block.lines().find(|l| !l.trim().is_empty()).unwrap_or("")
                    }
                    MatchLocation::Anywhere => target_text,
                };

                if refinement.re.is_match(match_text) {
                    explanation.push_str(&format!(
                        "  MATCHED Refinement #{} (status: {}, location: {:?}, pattern: {})\n",
                        r_idx, refinement.status, refinement.location, refinement.re
                    ));
                    break;
                }
            }
            return Some(explanation);
        }
        Some(explanation)
    }

    fn highlight_line(&self, line: &str) -> Option<ratatui::style::Style> {
        for rule in &self.highlight_rules {
            if rule.re.is_match(line) {
                let color = crate::ui::Styles::parse_color(&rule.color);
                let mut style = ratatui::style::Style::default().fg(color);
                for modifier in &rule.modifiers {
                    match modifier.to_lowercase().as_str() {
                        "bold" => style = style.add_modifier(ratatui::style::Modifier::BOLD),
                        "italic" => style = style.add_modifier(ratatui::style::Modifier::ITALIC),
                        "dim" => style = style.add_modifier(ratatui::style::Modifier::DIM),
                        "reversed" => {
                            style = style.add_modifier(ratatui::style::Modifier::REVERSED)
                        }
                        _ => {}
                    }
                }
                return Some(style);
            }
        }
        None
    }

    fn process_indicators(&self) -> Vec<crate::app::config::ProcessIndicator> {
        self.config.process_indicators.clone()
    }

    fn approval_keys(&self) -> &str {
        // Return first key for display in approval prompt
        self.config
            .keys
            .approve
            .first()
            .map(|s| s.as_str())
            .unwrap_or("y")
    }

    fn rejection_keys(&self) -> &str {
        // Return first key for display in approval prompt
        self.config
            .keys
            .reject
            .first()
            .map(|s| s.as_str())
            .unwrap_or("n")
    }

    fn is_ai(&self) -> bool {
        self.config.is_ai
    }
}
