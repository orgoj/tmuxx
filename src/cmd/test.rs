use anyhow::Result;
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

    let config = Config::load_merged();
    let mut total_success = 0;
    let mut total_fail = 0;

    // Check for subdirectories to run recursively
    let mut subdirs: Vec<PathBuf> = fs::read_dir(&args.dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();

    subdirs.sort();

    if !subdirs.is_empty() {
        println!("📂 Found {} test suites (subdirectories)", subdirs.len());
        for dir in subdirs {
            let (s, f) = run_suite_for_dir(&dir, &config)?;
            total_success += s;
            total_fail += f;
        }
    } else {
        // Run in single directory mode
        let (s, f) = run_suite_for_dir(&args.dir, &config)?;
        total_success += s;
        total_fail += f;
    }

    println!(
        "\n📊 Total Results: {} Passed, {} Failed",
        total_success, total_fail
    );

    if total_fail > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_suite_for_dir(dir: &std::path::Path, config: &Config) -> Result<(usize, usize)> {
    let dirname = dir.file_name().unwrap_or_default().to_string_lossy();

    // Determine Agent ID
    let agent_id = if dirname == "shell" || dirname.ends_with("shell") {
        "generic_shell"
    } else {
        &dirname
    };

    println!("\n🔍 Test Suite: {} (Agent: {})", dirname, agent_id);

    // Find Agent Config
    let agent_config = match config.agents.iter().find(|a| a.id == agent_id) {
        Some(c) => c.clone(),
        None => {
            println!(
                "⚠️ Skipping suite '{}': Agent '{}' not found in config",
                dirname, agent_id
            );
            return Ok((0, 0));
        }
    };

    // Initialize Parser
    let parser = UniversalParser::new(agent_config);

    // Iterate over fixtures
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .collect();

    files.sort();

    if files.is_empty() {
        println!("   (No test files found)");
        return Ok((0, 0));
    }

    let mut success_count = 0;
    let mut fail_count = 0;

    for path in files {
        let filename = path.file_name().unwrap().to_string_lossy();
        let content = fs::read_to_string(&path)?;

        // Parse expected status
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() < 3 || parts[0] != "case" {
            println!("⚠️ Skipping invalid filename format: {}", filename);
            continue;
        }

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

        let actual_status = parser.parse_status(&content);
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
            "  📄 {:<40} Expected: {:<15} Got: {:<15} -> {}",
            filename,
            status_part,
            actual_status.short_text(),
            result_str
        );

        if !is_match {
            println!("     Actual Status: {:?}", actual_status);
        }
    }

    Ok((success_count, fail_count))
}
