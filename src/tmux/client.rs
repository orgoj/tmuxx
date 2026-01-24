use anyhow::{Context, Result};
use std::process::Command;

use super::pane::PaneInfo;
use crate::app::{Config, KillMethod};

/// Client for interacting with tmux
pub struct TmuxClient {
    /// Number of lines to capture from pane
    capture_lines: u32,
    /// Whether to show detached tmux sessions
    show_detached_sessions: bool,
}

impl TmuxClient {
    /// Creates a new TmuxClient from a Config
    pub fn from_config(config: &Config) -> Self {
        Self {
            capture_lines: config.capture_lines,
            show_detached_sessions: config.show_detached_sessions,
        }
    }

    /// Creates a new TmuxClient with default settings
    pub fn new() -> Self {
        Self {
            capture_lines: 100,
            show_detached_sessions: true,
        }
    }

    /// Creates a new TmuxClient with custom capture lines
    pub fn with_capture_lines(capture_lines: u32) -> Self {
        Self {
            capture_lines,
            show_detached_sessions: true,
        }
    }

    /// Check if tmux is available and running
    pub fn is_available(&self) -> bool {
        Command::new("tmux")
            .arg("list-sessions")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Lists all panes across all attached sessions
    pub fn list_panes(&self) -> Result<Vec<PaneInfo>> {
        // Use tab separator to handle spaces in titles/paths
        // Include session_attached to filter out detached sessions
        let output = Command::new("tmux")
            .args([
                "list-panes",
                "-a",
                "-F",
                "#{session_attached}\t#{session_name}:#{window_index}.#{pane_index}\t#{window_name}\t#{pane_current_command}\t#{pane_pid}\t#{pane_title}\t#{pane_current_path}",
            ])
            .output()
            .context("Failed to execute tmux list-panes")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("tmux list-panes failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let panes: Vec<PaneInfo> = stdout
            .lines()
            .filter_map(|line| {
                let (attached_str, rest) = line.split_once('\t')?;

                // Filter based on config setting
                if !self.show_detached_sessions {
                    // Skip detached sessions (attached_str == "0")
                    if attached_str != "1" {
                        return None;
                    }
                }

                // Parse pane info
                PaneInfo::parse(rest)
            })
            .collect();

        Ok(panes)
    }

    /// Captures the content of a specific pane
    pub fn capture_pane(&self, target: &str) -> Result<String> {
        let start_line = format!("-{}", self.capture_lines);

        let output = Command::new("tmux")
            .args(["capture-pane", "-p", "-t", target, "-S", &start_line])
            .output()
            .context("Failed to execute tmux capture-pane")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("tmux capture-pane failed for {}: {}", target, stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Sends keys to a specific pane
    pub fn send_keys(&self, target: &str, keys: &str) -> Result<()> {
        let output = Command::new("tmux")
            .args(["send-keys", "-t", target, keys])
            .output()
            .context("Failed to execute tmux send-keys")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("tmux send-keys failed for {}: {}", target, stderr);
        }

        Ok(())
    }

    /// Selects (focuses) a specific pane
    pub fn select_pane(&self, target: &str) -> Result<()> {
        let output = Command::new("tmux")
            .args(["select-pane", "-t", target])
            .output()
            .context("Failed to execute tmux select-pane")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("tmux select-pane failed for {}: {}", target, stderr);
        }

        Ok(())
    }

    /// Selects a specific window
    pub fn select_window(&self, target: &str) -> Result<()> {
        // Extract session:window from full target
        let window_target = if let Some(pos) = target.rfind('.') {
            &target[..pos]
        } else {
            target
        };

        let output = Command::new("tmux")
            .args(["select-window", "-t", window_target])
            .output()
            .context("Failed to execute tmux select-window")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "tmux select-window failed for {}: {}",
                window_target,
                stderr
            );
        }

        Ok(())
    }

    /// Kill application in target pane
    pub fn kill_application(&self, target: &str, method: &KillMethod) -> Result<()> {
        match method {
            KillMethod::Sigterm => {
                // Get PID of process in pane
                let output = Command::new("tmux")
                    .args(["display-message", "-t", target, "-p", "#{pane_pid}"])
                    .output()
                    .context("Failed to get pane PID")?;

                if !output.status.success() {
                    anyhow::bail!("Could not get pane PID for {}", target);
                }

                let pid_str = String::from_utf8(output.stdout)?;
                let pid = pid_str
                    .trim()
                    .parse::<i32>()
                    .context("Failed to parse PID")?;

                // Send SIGTERM and check result
                let result = unsafe { libc::kill(pid, libc::SIGTERM) };
                if result == -1 {
                    return Err(anyhow::anyhow!(
                        "Failed to send SIGTERM to PID {}: {}",
                        pid,
                        std::io::Error::last_os_error()
                    ));
                }
                Ok(())
            }
            KillMethod::CtrlCCtrlD => {
                // Send Ctrl-C then Ctrl-D sequence
                self.send_keys(target, "C-c")?;
                std::thread::sleep(std::time::Duration::from_millis(100));
                self.send_keys(target, "C-d")?;
                Ok(())
            }
        }
    }

    /// Check if running inside tmux
    fn is_inside_tmux() -> bool {
        std::env::var("TMUX").is_ok()
    }

    /// Get current tmux session name (if inside tmux)
    pub fn get_current_session(&self) -> Result<Option<String>> {
        if !Self::is_inside_tmux() {
            return Ok(None);
        }

        let output = Command::new("tmux")
            .args(["display-message", "-p", "#S"])
            .output()
            .context("Failed to get current tmux session")?;

        if !output.status.success() {
            return Ok(None);
        }

        let session = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(session))
    }

    /// Focuses on a pane by selecting its window and pane
    ///
    /// Supports cross-session focus when running inside tmux.
    /// If target is in a different session, uses switch-client to change sessions.
    /// If running outside tmux, returns an error.
    pub fn focus_pane(&self, target: &str) -> Result<()> {
        // Extract session from target (e.g., "main:0.1" -> "main")
        let target_session = target
            .split(':')
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid target format: {}", target))?;

        // Check if inside tmux
        if !Self::is_inside_tmux() {
            anyhow::bail!(
                "Cannot focus pane: tmuxcc is not running inside tmux.\n\
                 Run tmuxcc in a tmux pane to use focus (f key)"
            );
        }

        // Get current session
        let current_session = self
            .get_current_session()?
            .ok_or_else(|| anyhow::anyhow!("Could not determine current tmux session"))?;

        if current_session == target_session {
            // Same session: use existing logic
            self.select_window(target)?;
            self.select_pane(target)?;
        } else {
            // Different session: use switch-client
            let output = Command::new("tmux")
                .args(["switch-client", "-t", target])
                .output()
                .context("Failed to execute tmux switch-client")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("tmux switch-client failed for {}: {}", target, stderr);
            }
        }

        Ok(())
    }

    /// Renames a tmux session
    pub fn rename_session(&self, old_name: &str, new_name: &str) -> Result<()> {
        let output = Command::new("tmux")
            .args(["rename-session", "-t", old_name, new_name])
            .output()
            .context("Failed to execute tmux rename-session")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("tmux rename-session failed: {}", stderr);
        }
        Ok(())
    }
}

impl Default for TmuxClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TmuxClient::new();
        assert_eq!(client.capture_lines, 100);

        let custom_client = TmuxClient::with_capture_lines(200);
        assert_eq!(custom_client.capture_lines, 200);
    }
}
