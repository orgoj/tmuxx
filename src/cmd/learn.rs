use crate::app::config::{AgentConfig, AgentKeys, MatcherConfig, StateRule};
use crate::tmux::TmuxClient;
use anyhow::{anyhow, Result};
use std::io::{self, Write};

pub struct LearnArgs {
    pub target_pane: Option<String>,
    pub agent_name: Option<String>,
}

pub async fn run_learn(args: LearnArgs) -> Result<()> {
    let client = TmuxClient::new();

    // 1. Select Pane
    let panes = client.list_panes()?;
    let target = if let Some(target_id) = args.target_pane {
        panes
            .into_iter()
            .find(|p| p.target() == target_id || p.title == target_id)
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
        panes
            .get(idx)
            .ok_or_else(|| anyhow!("Invalid index"))?
            .clone()
    };

    println!("Analyzing pane: {} ({})", target.target(), target.command);

    // 2. Identify Agent Process
    let name = args.agent_name.unwrap_or_else(|| {
        // Guess name from command
        target.command.clone()
    });

    // 3. Capture Content & Determine State
    let content = client.capture_pane(&target.target())?;
    println!("\n--- Pane Content (Last 5 lines) ---");
    let lines: Vec<&str> = content.lines().collect();
    for line in lines.iter().rev().take(5).rev() {
        println!("| {}", line);
    }
    println!("-----------------------------------\n");

    print!("What is the current state? [W]orking, [I]dle, [E]rror, [A]pproval: ");
    io::stdout().flush()?;
    let mut state_input = String::new();
    io::stdin().read_line(&mut state_input)?;
    let state_char = state_input.trim().to_lowercase();

    let (status, kind) = match state_char.chars().next() {
        Some('w') => ("working", Some(crate::app::config::RuleType::Working)),
        Some('i') => ("idle", Some(crate::app::config::RuleType::Idle)),
        Some('e') => ("error", Some(crate::app::config::RuleType::Error)),
        Some('a') => ("approval", Some(crate::app::config::RuleType::Approval)),
        _ => ("working", Some(crate::app::config::RuleType::Working)),
    };

    // 4. Propose Pattern
    let last_line = lines.last().unwrap_or(&"");
    let proposed_pattern = if status == "ready" || status == "done" {
        format!("{}$", regex::escape(last_line.trim()))
    } else {
        String::new()
    };

    println!("Proposed pattern for '{}': {}", status, proposed_pattern);
    print!("Enter pattern (regex) [Press Enter to accept proposed]: ");
    io::stdout().flush()?;
    let mut pattern_input = String::new();
    io::stdin().read_line(&mut pattern_input)?;
    let final_pattern = if pattern_input.trim().is_empty() {
        proposed_pattern
    } else {
        pattern_input.trim().to_string()
    };

    // 5. Generate Definition (AgentConfig)
    let definition = AgentConfig {
        id: name.to_lowercase().replace(' ', "-"),
        name: name.clone(),
        color: Some("cyan".to_string()),
        background_color: None,
        priority: 10,
        matchers: vec![MatcherConfig::Command {
            pattern: regex::escape(&target.command),
        }],
        state_rules: vec![StateRule {
            status: status.to_string(),
            kind,
            pattern: final_pattern,
            approval_type: None,
            last_lines: None,
            splitter: None,
            refinements: Vec::new(),
        }],
        subagent_rules: None,
        process_indicators: Vec::new(),
        title_indicators: None,
        default_status: None,
        default_type: None,
        keys: AgentKeys::default(),
        layout: None,
        summary_rules: None,
        highlight_rules: Vec::new(),
    };

    // Output TOML
    println!("\n--- Generated Configuration ---");
    let toml_str = toml::to_string_pretty(&definition)?;
    println!("\n[[agents]]\n{}", toml_str);

    println!("Do you want to append this to your config.toml? [y/N] ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().eq_ignore_ascii_case("y") {
        let config_path = crate::app::Config::default_path().unwrap();
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(config_path)?;

        writeln!(file, "\n[[agents]]")?;
        writeln!(file, "{}", toml_str)?;
        println!("Configuration saved.");
    }

    Ok(())
}
