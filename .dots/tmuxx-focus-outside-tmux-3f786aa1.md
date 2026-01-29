---
title: Focus Outside Tmux
status: open
priority: 5
issue-type: task
created-at: "2026-01-29T16:48:42.322670+01:00"
---

Účel: Klávesa 'f' (Focus) funguje i když tmuxx běží mimo tmux pomocí terminal_wrapperu.

Změny:
1. 'src/ui/app.rs' - handling 'f' key:
   Pokud TmuxClient::is_inside_tmux() == false:
   Použít 'state.config.terminal_wrapper' (default: 'xterm -e "{cmd}"').
   Sestavit cmd: 'tmux attach -t "{session_name}"'.
   Nahradit '{cmd}' ve wrapperu a spustit přes std::process::Command::spawn().
