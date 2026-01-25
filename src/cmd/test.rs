use anyhow::{Context, Result};
use crossterm::style::Stylize;
use std::fs;
use std::path::PathBuf;

use crate::agents::AgentStatus;
use crate::app::config::Config;
use crate::parsers::AgentParser;
use crate::parsers::UniversalParser;

pub struct TestArgs {
    pub dir: PathBuf,
}

pub async fn run_test(args: TestArgs) -> Result<()> {
    println!("🧪 Running Regression Tests in {}", args.dir.display());

    // 1. Load Configuration
    let config = Config::load_merged();

    // 2. Find Claude Agent
    let agent_config = config
        .agents
        .iter()
        .find(|a| a.id == "claude")
        .context("Agent 'claude' not found in config")?
        .clone();

    // 3. Initialize Parser
    let parser = UniversalParser::new(agent_config.clone());

    // 4. Iterate over fixtures
    let mut files: Vec<PathBuf> = fs::read_dir(&args.dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .collect();

    files.sort();

    let mut success_count = 0;
    let mut fail_count = 0;

    for path in files {
        let filename = path.file_name().unwrap().to_string_lossy();
        let content = fs::read_to_string(&path)?;

        // Parse expected status from filename: string between first and second underscore
        // Format: case_<STATUS>_<DESC>.txt
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() < 3 || parts[0] != "case" {
            println!("⚠️ Skipping invalid filename format: {}", filename);
            continue;
        }

        // Handle composite statuses like "awaiting_approval" which contain underscore
        // Strategy: Try to match known status strings first
        let status_part = if filename.contains("awaiting_approval") {
            "awaiting_approval"
        } else if filename.contains("awaiting_input") {
            "awaiting_input"
        } else {
            parts[1]
        };

        let expected_status_enum = match status_part {
            "idle" => AgentStatus::Idle,
            "processing" => AgentStatus::Processing {
                activity: "".to_string(),
            },
            "awaiting_approval" => AgentStatus::AwaitingApproval {
                approval_type: crate::agents::ApprovalType::Other("".to_string()),
                details: "".to_string(),
            },
            "error" => AgentStatus::Error {
                message: "".to_string(),
            },
            _ => {
                println!("⚠️ Unknown status type in filename: {}", status_part);
                continue;
            }
        };

        println!("\n📄 Checking {}", filename.bold());

        let actual_status = parser.parse_status(&content);

        // Compare Enums (Variant matching only, ignoring inner data like uptime/details)
        let is_match =
            std::mem::discriminant(&actual_status) == std::mem::discriminant(&expected_status_enum);

        let result_str = if is_match {
            success_count += 1;
            "PASS".green()
        } else {
            fail_count += 1;
            "FAIL".red()
        };

        println!(
            "  Expected: {:<20} Got: {:<20} -> {}",
            status_part,
            actual_status.short_text(),
            result_str
        );

        if !is_match {
            // Print detail for debugging
            println!("  Actual Status: {:?}", actual_status);
        }
    }

    println!(
        "\n📊 Results: {} Passed, {} Failed",
        success_count, fail_count
    );

    if fail_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
