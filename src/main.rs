use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tmuxcc::app::Config;
use tmuxcc::ui::run_app;

#[derive(Parser)]
#[command(name = "tmuxcc")]
#[command(author, version, about, long_about = None)]
#[command(
    about = "AI Agent Dashboard for tmux - manage Claude Code, OpenCode, Codex CLI, Gemini CLI in one place"
)]
struct Cli {
    /// Poll interval in milliseconds
    #[arg(short, long, default_value = "500", value_name = "MS")]
    poll_interval: u64,

    /// Number of lines to capture from pane
    #[arg(short, long, default_value = "100", value_name = "LINES")]
    capture_lines: u32,

    /// Path to config file
    #[arg(short = 'f', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Output debug logs to tmuxcc.log
    #[arg(short, long)]
    debug: bool,

    /// Show config file path
    #[arg(long)]
    show_config_path: bool,

    /// Debug: show loaded config and bindings before starting
    #[arg(long)]
    debug_config: bool,

    /// Generate default config file
    #[arg(long)]
    init_config: bool,

    /// Set config options (can be used multiple times)
    /// Example: --set show_detached_sessions=false
    #[arg(long = "set", value_name = "KEY=VALUE")]
    config_overrides: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Show config path and exit
    if cli.show_config_path {
        if let Some(path) = Config::default_path() {
            println!("{}", path.display());
        } else {
            println!("Config directory not found");
        }
        return Ok(());
    }

    // Initialize config file and exit
    if cli.init_config {
        let config = Config::default();
        if let Err(e) = config.save() {
            eprintln!("Failed to create config file: {}", e);
            std::process::exit(1);
        }
        if let Some(path) = Config::default_path() {
            println!("Config file created: {}", path.display());
        }
        return Ok(());
    }

    // Setup logging
    if cli.debug {
        let log_file = std::fs::File::create("tmuxcc.log")?;
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(log_file)
            .with_ansi(false);

        tracing_subscriber::registry()
            .with(file_layer)
            .with(tracing_subscriber::filter::LevelFilter::DEBUG)
            .init();
    }

    // Load config (from file or CLI args)
    let mut config = if let Some(config_path) = &cli.config {
        Config::load_from(config_path).unwrap_or_else(|e| {
            eprintln!("Failed to load config file: {}", e);
            std::process::exit(1);
        })
    } else {
        Config::load()
    };

    // CLI args override config file
    config.poll_interval_ms = cli.poll_interval;
    config.capture_lines = cli.capture_lines;

    // Apply --set overrides
    for override_str in &cli.config_overrides {
        let (key, value) = override_str.split_once('=').ok_or_else(|| {
            anyhow::anyhow!("Invalid --set format: '{}'. Use KEY=VALUE", override_str)
        })?;
        if let Err(e) = config.apply_override(key.trim(), value.trim()) {
            eprintln!("Error applying config override: {}", e);
            std::process::exit(1);
        }
    }

    // Debug: show loaded config and bindings
    if cli.debug_config {
        println!("=== Loaded Config ===");
        if let Some(path) = Config::default_path() {
            if path.exists() {
                println!("Config file: {}", path.display());
            } else {
                println!(
                    "Config file: {} (not found, using defaults)",
                    path.display()
                );
            }
        }
        println!("\nKey bindings:");
        for (key, action) in &config.key_bindings.bindings {
            println!("  [{}] -> {:?}", key, action);
        }
        println!("\nPress Enter to continue...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }

    // Run the application
    run_app(config).await
}
