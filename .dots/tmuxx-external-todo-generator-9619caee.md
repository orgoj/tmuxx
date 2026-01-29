---
title: External TODO Generator
status: open
priority: 2
issue-type: task
created-at: "2026-01-29T16:48:42.268397+01:00"
---

Účel: TODO panel plněný externím příkazem (beads, taskwarrior, atd.) běžícím asynchronně.

Změny:
1. 'src/app/config.rs' - přidat do 'Config':
   pub todo_command: Option<String>,
   #[serde(default = "default_todo_refresh")]
   pub todo_refresh_interval_ms: u64,
   fn default_todo_refresh() -> u64 { 30000 }

2. 'src/app/state.rs' - přidat do 'AppState':
   pub todo_last_refresh: Option<std::time::Instant>,

3. 'src/ui/app.rs' - ASYNC refresh v main loop:
   Použít tokio::process::Command a tokio::spawn. Refresh nesmí blokovat TUI.
   Výsledek (stdout) uložit do 'state.current_todo'. Refreshovat pouze pokud uplynul 'todo_refresh_interval_ms'.
