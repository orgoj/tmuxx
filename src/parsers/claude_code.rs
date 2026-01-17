use regex::Regex;

use crate::agents::{AgentStatus, AgentType, ApprovalType, Subagent, SubagentStatus, SubagentType};

use super::{safe_tail, AgentParser};

/// Check if a string looks like a version number (e.g., "2.1.11")
/// Claude Code's pane_current_command often shows version number
fn is_version_like(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Version pattern: digits and dots only, at least one dot
    let has_dot = s.contains('.');
    let all_valid = s.chars().all(|c| c.is_ascii_digit() || c == '.');
    has_dot && all_valid && s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}

/// Parser for Claude Code CLI output
pub struct ClaudeCodeParser {
    // Approval patterns
    file_edit_pattern: Regex,
    file_create_pattern: Regex,
    file_delete_pattern: Regex,
    bash_pattern: Regex,
    mcp_pattern: Regex,
    general_approval_pattern: Regex,

    // Status patterns
    thinking_pattern: Regex,
    tool_use_pattern: Regex,
    idle_pattern: Regex,

    // Subagent patterns
    task_start_pattern: Regex,
    task_running_pattern: Regex,
    task_complete_pattern: Regex,
}

impl ClaudeCodeParser {
    pub fn new() -> Self {
        Self {
            // Approval patterns - detect pending approval prompts
            file_edit_pattern: Regex::new(
                r"(?i)(Edit|Write|Modify)\s+.*?\?.*?\[y/n\]|Do you want to (edit|write|modify)"
            ).unwrap(),
            file_create_pattern: Regex::new(
                r"(?i)Create\s+.*?\?.*?\[y/n\]|Do you want to create"
            ).unwrap(),
            file_delete_pattern: Regex::new(
                r"(?i)Delete\s+.*?\?.*?\[y/n\]|Do you want to delete"
            ).unwrap(),
            bash_pattern: Regex::new(
                r"(?i)(Run|Execute)\s+(command|bash|shell).*?\[y/n\]|Do you want to run"
            ).unwrap(),
            mcp_pattern: Regex::new(
                r"(?i)MCP\s+tool.*?\[y/n\]|Do you want to use.*?MCP"
            ).unwrap(),
            general_approval_pattern: Regex::new(
                r"\[y/n\]|\[Y/n\]|\[yes/no\]"
            ).unwrap(),

            // Status patterns
            thinking_pattern: Regex::new(
                r"(?i)(Thinking|Processing|Analyzing|Working)"
            ).unwrap(),
            tool_use_pattern: Regex::new(
                r"(?i)(Using|Calling|Invoking)\s+(tool|function)|Tool:|Read|Write|Edit|Bash|Glob|Grep"
            ).unwrap(),
            idle_pattern: Regex::new(
                r"(?i)Ready|Waiting for input|>\s*$"
            ).unwrap(),

            // Subagent patterns for Claude Code's Task tool
            // Match: ⏺ Task(...subagent_type="Explore"...description="..."...)
            task_start_pattern: Regex::new(
                r#"(?m)[⏺⠿⠇⠋⠙⠸⠴⠦⠧⠖⠏]\s*Task\s*\([^)]*subagent_type\s*[:=]\s*["']?(\w[\w-]*)["']?[^)]*description\s*[:=]\s*["']([^"']+)["']"#
            ).unwrap(),
            // Match running spinner indicators with agent type
            task_running_pattern: Regex::new(
                r"(?m)^[^│]*[▶►⠿⠇⠋⠙⠸⠴⠦⠧⠖⠏]\s*(\w+)(?:\s*agent)?:?\s*(.*)$"
            ).unwrap(),
            // Match completed indicators
            task_complete_pattern: Regex::new(
                r"(?m)[✓✔]\s*(\w+).*?(?:completed|finished|done|returned)"
            ).unwrap(),
        }
    }

    fn detect_approval(&self, content: &str) -> Option<(ApprovalType, String)> {
        // Only check the last portion of content for pending approvals
        let recent = safe_tail(content, 2000);

        // Check for user question with choices first (AskUserQuestion)
        if let Some((choices, question)) = self.extract_user_question(recent) {
            if !choices.is_empty() {
                return Some((
                    ApprovalType::UserQuestion {
                        choices,
                        multi_select: false, // Could detect from prompt
                    },
                    question,
                ));
            }
        }

        if self.file_edit_pattern.is_match(recent) {
            let details = self.extract_file_path(recent).unwrap_or_default();
            return Some((ApprovalType::FileEdit, details));
        }

        if self.file_create_pattern.is_match(recent) {
            let details = self.extract_file_path(recent).unwrap_or_default();
            return Some((ApprovalType::FileCreate, details));
        }

        if self.file_delete_pattern.is_match(recent) {
            let details = self.extract_file_path(recent).unwrap_or_default();
            return Some((ApprovalType::FileDelete, details));
        }

        if self.bash_pattern.is_match(recent) {
            let details = self.extract_command(recent).unwrap_or_default();
            return Some((ApprovalType::ShellCommand, details));
        }

        if self.mcp_pattern.is_match(recent) {
            return Some((ApprovalType::McpTool, "MCP tool call".to_string()));
        }

        if self.general_approval_pattern.is_match(recent) {
            return Some((ApprovalType::Other("Pending approval".to_string()), String::new()));
        }

        None
    }

    /// Extract user question with numbered choices
    /// Only detects choices at the END of content (active prompt waiting for input)
    fn extract_user_question(&self, content: &str) -> Option<(Vec<String>, String)> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return None;
        }

        // Find the last prompt marker (❯ or >) - anything after this is user input area
        let last_prompt_idx = lines.iter().rposition(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('❯') || (trimmed.starts_with('>') && trimmed.len() < 3)
        });

        // If there's a prompt marker, only look BEFORE it for choices
        // (Choices after the prompt are past responses, not active questions)
        let search_end = last_prompt_idx.unwrap_or(lines.len());

        // Only check the last 25 lines before the prompt
        let search_start = search_end.saturating_sub(25);
        let check_lines = &lines[search_start..search_end];

        if check_lines.is_empty() {
            return None;
        }

        let mut choices = Vec::new();
        let mut question = String::new();
        let mut first_choice_idx = None;
        let mut last_choice_idx = None;

        // Pattern for numbered choices: "1. Option text" or "  1. Option text"
        let choice_pattern = Regex::new(r"^\s*(\d+)\.\s+(.+)$").ok()?;

        for (i, line) in check_lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip lines that are clearly not choices (table borders, etc.)
            if trimmed.starts_with('│') || trimmed.starts_with('├') ||
               trimmed.starts_with('└') || trimmed.starts_with('┌') ||
               trimmed.starts_with('─') || trimmed.starts_with('✻') {
                if !choices.is_empty() {
                    // Non-choice content after we started - reset
                    choices.clear();
                    first_choice_idx = None;
                    last_choice_idx = None;
                }
                continue;
            }

            if let Some(cap) = choice_pattern.captures(line) {
                if let Ok(num) = cap[1].parse::<u32>() {
                    let choice_text = cap[2].trim();

                    // Accept sequential numbers starting from 1
                    if num as usize == choices.len() + 1 {
                        // Clean up choice text - remove trailing description markers
                        let label = choice_text
                            .split('（')  // Japanese parenthesis
                            .next()
                            .unwrap_or(choice_text)
                            .trim();

                        choices.push(label.to_string());

                        if first_choice_idx.is_none() {
                            first_choice_idx = Some(i);
                        }
                        last_choice_idx = Some(i);
                    } else if !choices.is_empty() {
                        // Non-sequential number after we started - reset
                        choices.clear();
                        first_choice_idx = None;
                        last_choice_idx = None;
                    }
                }
            } else if !choices.is_empty() {
                // Non-choice line after choices started
                // Allow empty lines and very short lines
                if !trimmed.is_empty() && trimmed.len() > 30 {
                    // Longer content after choices - not an active question prompt
                    choices.clear();
                    first_choice_idx = None;
                    last_choice_idx = None;
                }
            }
        }

        // Choices must be near the end of check_lines (within last 8 lines)
        if let Some(last_idx) = last_choice_idx {
            if check_lines.len() - last_idx > 8 {
                return None; // Choices too far from end/prompt
            }
        }

        // Look for question text before the first choice
        if let Some(first_idx) = first_choice_idx {
            for j in (0..first_idx).rev() {
                let prev = check_lines[j].trim();
                if prev.is_empty() {
                    continue;
                }
                // Question usually ends with ? or ？
                if prev.ends_with('?') || prev.ends_with('？') || prev.contains('?') || prev.contains('？') {
                    question = prev.to_string();
                    break;
                }
                // If we find a non-empty line that's not a question, use it anyway
                if question.is_empty() {
                    question = prev.to_string();
                }
                // Only look back a few lines
                if first_idx - j > 5 {
                    break;
                }
            }
        }

        if choices.len() >= 2 {
            Some((choices, question))
        } else {
            None
        }
    }

    fn extract_file_path(&self, content: &str) -> Option<String> {
        let path_pattern = Regex::new(r"(?m)(?:file|path)[:\s]+([^\s\n]+)|([./][\w/.-]+\.\w+)").ok()?;
        path_pattern
            .captures(content)
            .and_then(|c| c.get(1).or(c.get(2)))
            .map(|m| m.as_str().to_string())
    }

    fn extract_command(&self, content: &str) -> Option<String> {
        let cmd_pattern = Regex::new(r"(?m)(?:command|run)[:\s]+`([^`]+)`|```(?:bash|sh)?\n([^`]+)```").ok()?;
        cmd_pattern
            .captures(content)
            .and_then(|c| c.get(1).or(c.get(2)))
            .map(|m| m.as_str().trim().to_string())
    }
}

impl Default for ClaudeCodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentParser for ClaudeCodeParser {
    fn agent_name(&self) -> &str {
        "Claude Code"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::ClaudeCode
    }

    fn matches(&self, detection_strings: &[&str]) -> bool {
        detection_strings.iter().any(|s| {
            let lower = s.to_lowercase();
            // Match by name
            lower.contains("claude") || lower.contains("anthropic")
            // Match by Claude Code icon (✳) in title
            || s.contains('✳')
            // Match by version number pattern (e.g., "2.1.11" as command)
            || is_version_like(s)
        })
    }

    fn parse_status(&self, content: &str) -> AgentStatus {
        // Check for approval prompts first (highest priority)
        if let Some((approval_type, details)) = self.detect_approval(content) {
            return AgentStatus::AwaitingApproval {
                approval_type,
                details,
            };
        }

        // Check recent content for activity indicators
        let recent = safe_tail(content, 500);

        if self.thinking_pattern.is_match(recent) {
            return AgentStatus::Processing {
                activity: "Thinking...".to_string(),
            };
        }

        if self.tool_use_pattern.is_match(recent) {
            return AgentStatus::Processing {
                activity: "Using tools...".to_string(),
            };
        }

        if self.idle_pattern.is_match(recent) {
            return AgentStatus::Idle;
        }

        AgentStatus::Unknown
    }

    fn parse_subagents(&self, content: &str) -> Vec<Subagent> {
        let mut subagents = Vec::new();
        let mut id_counter = 0;

        // Find task starts
        for cap in self.task_start_pattern.captures_iter(content) {
            let subagent_type = SubagentType::from_str(&cap[1]);
            let description = cap[2].to_string();
            id_counter += 1;

            subagents.push(Subagent::new(
                format!("subagent-{}", id_counter),
                subagent_type,
                description,
            ));
        }

        // Find running indicators
        for cap in self.task_running_pattern.captures_iter(content) {
            let type_name = &cap[1];
            let desc = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Check if we already have this subagent
            let existing = subagents.iter().any(|s| {
                s.subagent_type.display_name().to_lowercase() == type_name.to_lowercase()
            });

            if !existing {
                id_counter += 1;
                subagents.push(Subagent::new(
                    format!("subagent-{}", id_counter),
                    SubagentType::from_str(type_name),
                    desc.to_string(),
                ));
            }
        }

        // Mark completed ones
        for cap in self.task_complete_pattern.captures_iter(content) {
            let type_name = &cap[1];
            for subagent in &mut subagents {
                if subagent.subagent_type.display_name().to_lowercase() == type_name.to_lowercase() {
                    subagent.status = SubagentStatus::Completed;
                }
            }
        }

        subagents
    }

    fn approval_keys(&self) -> &str {
        "y"
    }

    fn rejection_keys(&self) -> &str {
        "n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let parser = ClaudeCodeParser::new();
        // Match via command
        assert!(parser.matches(&["claude", "", ""]));
        assert!(parser.matches(&["Claude", "", ""]));
        // Match via cmdline
        assert!(parser.matches(&["node", "", "/usr/bin/claude -c"]));
        // Match via title with Claude Code text
        assert!(parser.matches(&["2.1.11", "Claude Code", ""]));
        // Match via ✳ icon in title
        assert!(parser.matches(&["node", "✳ Some Task", ""]));
        assert!(parser.matches(&["2.1.11", "✳ CLI取得の改善", ""]));
        // Match via version number as command (Claude Code shows version)
        assert!(parser.matches(&["2.1.11", "Some Title", ""]));
        assert!(parser.matches(&["1.0.0", "", ""]));
        // No match
        assert!(!parser.matches(&["opencode", "OpenCode", "opencode"]));
        assert!(!parser.matches(&["fish", "~", "fish"]));
    }

    #[test]
    fn test_is_version_like() {
        assert!(is_version_like("2.1.11"));
        assert!(is_version_like("1.0.0"));
        assert!(is_version_like("0.1"));
        assert!(!is_version_like("fish"));
        assert!(!is_version_like("node"));
        assert!(!is_version_like(""));
        assert!(!is_version_like("2"));  // No dot
    }

    #[test]
    fn test_parse_approval_file_edit() {
        let parser = ClaudeCodeParser::new();
        let content = "Do you want to edit src/main.rs? [y/n]";
        let status = parser.parse_status(content);

        match status {
            AgentStatus::AwaitingApproval { approval_type, .. } => {
                assert_eq!(approval_type, ApprovalType::FileEdit);
            }
            _ => panic!("Expected AwaitingApproval status"),
        }
    }

    #[test]
    fn test_parse_thinking() {
        let parser = ClaudeCodeParser::new();
        let content = "Thinking about the problem...";
        let status = parser.parse_status(content);

        match status {
            AgentStatus::Processing { activity } => {
                assert!(activity.contains("Thinking"));
            }
            _ => panic!("Expected Processing status"),
        }
    }

    #[test]
    fn test_parse_subagents() {
        let parser = ClaudeCodeParser::new();
        let content = r#"
            Task subagent_type="Explore" description="searching codebase"
            ▶ Plan: designing API
            ✓ Explore completed
        "#;

        let subagents = parser.parse_subagents(content);
        assert!(!subagents.is_empty());
    }
}
