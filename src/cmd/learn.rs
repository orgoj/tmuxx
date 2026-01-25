use anyhow::{Result, anyhow};
use std::io::{self, Write};
use crate::tmux::TmuxClient;
use crate::app::config::{AgentDefinition, StateRule, MatchMode};

pub struct LearnArgs {
    pub target_pane: Option<String>,
    pub agent_name: Option<String>,
}

pub async fn run_learn(args: LearnArgs) -> Result<()> {
    let client = TmuxClient::new();
    
    // 1. Select Pane
    let panes = client.list_panes()?;
    let target = if let Some(target_id) = args.target_pane {
        panes.into_iter().find(|p| p.target() == target_id || p.title == target_id)
            .ok_or_else(|| anyhow!("Pane '{}' not found", target_id))?
    } else {
        println!("Available Panes:");
        for (i, p) in panes.iter().enumerate() {
            println!("{}: {} ({}) - {}", i, p.target(), p.title, p.command);
        }
        print!("Select pane index: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let idx: usize = input.trim().parse()?;
        panes.get(idx).ok_or_else(|| anyhow!("Invalid index"))?.clone()
    };
    
    println!("Analyzing pane: {} ({})", target.target(), target.command);
    
    // 2. Identify Agent Process
    let name = args.agent_name.unwrap_or_else(|| {
        // Guess name from command
        target.command.clone()
    });
    
    // Capture command hierarchy for detection
    let mut detection_patterns = Vec::new();
    detection_patterns.push(regex::escape(&target.command));
    for child in &target.child_commands {
        detection_patterns.push(regex::escape(child));
    }
    // Also include ancestors if significant? 
    // Usually the immediate command or a child is the agent.
    
    println!("Detected command patterns: {:?}", detection_patterns);

    // 3. Capture Content & Determine State
    let content = client.capture_pane(&target.target())?;
    println!("\n--- Pane Content (Last 5 lines) ---");
    let lines: Vec<&str> = content.lines().collect();
    for line in lines.iter().rev().take(5).rev() {
        println!("| {}", line);
    }
    println!("-----------------------------------\n");
    
    print!("What is the current state? [P]rocessing, [I]nput, [E]rror, [D]one: ");
    io::stdout().flush()?;
    let mut state_input = String::new();
    io::stdin().read_line(&mut state_input)?;
    let state_char = state_input.trim().to_lowercase();
    
    let state = match state_char.chars().next() {
        Some('p') => "processing",
        Some('i') => "awaiting_input",
        Some('e') => "error",
        Some('d') => "completed",
        _ => "processing",
    };
    
    // 4. Propose Pattern
    let last_line = lines.last().unwrap_or(&"");
    let proposed_pattern = if state == "awaiting_input" {
        // Escape the last line and anchor it
        format!("{}$", regex::escape(last_line.trim()))
    } else {
        // Just match something specific?
        // For generic states, it's harder.
        // Let's rely on user
        String::new()
    };
    
    println!("Proposed pattern for '{}': {}", state, proposed_pattern);
    print!("Enter pattern (regex) [Press Enter to accept proposed]: ");
    io::stdout().flush()?;
    let mut pattern_input = String::new();
    io::stdin().read_line(&mut pattern_input)?;
    let final_pattern = if pattern_input.trim().is_empty() {
        proposed_pattern
    } else {
        pattern_input.trim().to_string()
    };
    
    // 5. Generate Definition
    let definition = AgentDefinition {
        name: name.clone(),
        match_patterns: detection_patterns,
        priority: 10,
        state_rules: vec![
            StateRule {
                state: state.to_string(),
                pattern: final_pattern,
                mode: MatchMode::Regex,
            }
        ],
        approval_keys: Some("y".to_string()),
        rejection_keys: Some("n".to_string()),
    };
    
    // Output TOML
    println!("\n--- Generated Configuration ---");
    let toml_str = toml::to_string_pretty(&definition)?;
    println!("\n[[agent_definitions]]\n{}", toml_str);
    
    println!("\n--- Generated Test Case (TODO) ---");
    // TODO: Write to a file in /tmp or append to config?
    
    println!("Do you want to append this to your config.toml? [y/N] ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().eq_ignore_ascii_case("y") {
        let config_path = crate::app::Config::default_path().unwrap();
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(config_path)?;
            
        writeln!(file, "\n[[agent_definitions]]")?;
        writeln!(file, "{}", toml_str)?;
        println!("Configuration saved.");
    }

    Ok(())
}
