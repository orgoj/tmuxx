---
title: External TODO Generator
status: open
priority: 2
issue-type: task
created-at: "2026-01-29T21:24:44.450499+01:00"
---

Účel: TODO panel plněný externím příkazem (beads, taskwarrior, atd.) běžícím asynchronně v pozadí.

Změny:
1. 'src/app/config.rs' - přidat do 'Config':
   pub todo_command: Option<String>,
   #[serde(default = "default_todo_refresh")]
   pub todo_refresh_interval_ms: u64,
   fn default_todo_refresh() -> u64 { 30000 }

2. 'src/monitor/task.rs':
   - Přidat do 'MonitorUpdate' pole: 'pub external_todo: Option<String>'.
   - V 'MonitorTask' sledovat 'last_todo_refresh: Instant'.
   - V 'poll_agents' nebo přímo v 'run' smyčce: pokud je nastaven 'todo_command' a uplynul interval, spustit příkaz přes 'tokio::process::Command'.
   - Výstup (stdout) předat v 'MonitorUpdate'.

3. 'src/ui/app.rs':
   - V 'run_loop' při přijetí 'MonitorUpdate' aktualizovat 'state.current_todo', pokud update obsahuje 'external_todo'.
   - Zajistit, aby 'state.refresh_project_todo()' nepřebíjelo tento externí obsah, pokud je aktivní.
