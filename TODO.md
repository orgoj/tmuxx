# TODO - Tmuxx

## 🚀 Větší funkce (Features)

### Global Highlight Rules
**Účel:** Globální pravidla pro zvýraznění error/fail/exception ve všech agentech

**Změny:**
1. `src/app/config.rs` - přidat do `Config` (~řádek 95):
   ```rust
   /// Global highlight rules applied to all agents
   #[serde(default)]
   pub global_highlight_rules: Vec<HighlightRule>,
   ```

2. `src/ui/components/pane_preview.rs` - v renderování (~řádek 200):
   ```rust
   // Merge agent-specific + global rules
   let all_rules: Vec<_> = agent_config
       .highlight_rules.iter()
       .chain(state.config.global_highlight_rules.iter())
       .collect();
   ```

3. `src/config/defaults.toml`:
   ```toml
   [[global_highlight_rules]]
   pattern = "(?i)error"
   color = "red"
   modifiers = ["bold"]
   
   [[global_highlight_rules]]
   pattern = "(?i)fail(ed|ure)?"
   color = "red"
   
   [[global_highlight_rules]]
   pattern = "(?i)(traceback|exception|panic)"
   color = "yellow"
   modifiers = ["bold"]
   ```

---

### Notification System
**Účel:** Desktop notifikace když agent čeká na approval příliš dlouho

**Změny:**
1. `src/app/config.rs` - přidat do `Config`:
   ```rust
   /// Command to run for notifications (placeholders: {title}, {message}, {agent})
   /// Example: "notify-send '{title}' '{message}'"
   #[serde(default)]
   pub notification_command: Option<String>,
   
   /// Delay before sending notification (ms)
   #[serde(default = "default_notification_delay")]
   pub notification_delay_ms: u64,
   
   fn default_notification_delay() -> u64 { 60000 }  // 1 minute
   ```

2. `src/agents/types.rs` - přidat do `MonitoredAgent`:
   ```rust
   /// When approval was first detected (for notification timing)
   pub approval_since: Option<std::time::Instant>,
   /// Whether notification was already sent for current approval
   pub notification_sent: bool,
   ```

3. `src/monitor/task.rs` - v update loop přidat:
   ```rust
   // Check notification timeout
   if agent.status.needs_attention() {
       if agent.approval_since.is_none() {
           agent.approval_since = Some(Instant::now());
       }
       if !agent.notification_sent {
           if let Some(since) = agent.approval_since {
               if since.elapsed().as_millis() > config.notification_delay_ms as u128 {
                   send_notification(&config, &agent);
                   agent.notification_sent = true;
               }
           }
       }
   } else {
       agent.approval_since = None;
       agent.notification_sent = false;
   }
   ```

4. `src/monitor/task.rs` - nová funkce:
   ```rust
   fn send_notification(config: &Config, agent: &MonitoredAgent) {
       if let Some(cmd) = &config.notification_command {
           let expanded = cmd
               .replace("{title}", "tmuxx")
               .replace("{agent}", &agent.name)
               .replace("{message}", &format!("{} needs attention", agent.name));
           let _ = std::process::Command::new("bash")
               .args(["-c", &expanded])
               .spawn();
       }
   }
   ```

---

### External TODO Generator
**Účel:** TODO panel plněný externím příkazem (beads, taskwarrior, etc.)

**Změny:**
1. `src/app/config.rs` - přidat do `Config`:
   ```rust
   /// Command to generate TODO content (stdout becomes TODO panel)
   #[serde(default)]
   pub todo_command: Option<String>,
   
   /// How often to refresh TODO from command (ms)
   #[serde(default = "default_todo_refresh")]
   pub todo_refresh_interval_ms: u64,
   
   fn default_todo_refresh() -> u64 { 30000 }  // 30 seconds
   ```

2. `src/app/state.rs` - přidat do `AppState`:
   ```rust
   pub todo_last_refresh: Option<std::time::Instant>,
   ```

3. `src/ui/app.rs` - v main loop přidat refresh:
   ```rust
   // Refresh TODO from command if configured
   if let Some(cmd) = &state.config.todo_command {
       let should_refresh = state.todo_last_refresh
           .map(|t| t.elapsed().as_millis() > state.config.todo_refresh_interval_ms as u128)
           .unwrap_or(true);
       if should_refresh {
           if let Ok(output) = std::process::Command::new("bash")
               .args(["-c", cmd])
               .output() {
               state.current_todo = Some(String::from_utf8_lossy(&output.stdout).to_string());
               state.todo_last_refresh = Some(std::time::Instant::now());
           }
       }
   }
   ```

---

### Action Menu Variables
**Účel:** Menu položky s input prompty pro proměnné

**Změny:**
1. `src/app/menu_config.rs` - přidat do `MenuItem`:
   ```rust
   /// Variables to prompt for before execution
   /// Key: variable name, Value: prompt text
   #[serde(default)]
   pub variables: std::collections::HashMap<String, String>,
   ```

2. `src/app/state.rs` - přidat nový `PopupType`:
   ```rust
   MenuVariableInput {
       menu_item_path: Vec<usize>,
       variable_name: String,
       collected_vars: std::collections::HashMap<String, String>,
       remaining_vars: Vec<(String, String)>,  // (name, prompt)
   },
   ```

3. `src/ui/app.rs` - při Execute menu item:
   ```rust
   if !menu_item.variables.is_empty() {
       // Start variable collection popup
       let vars: Vec<_> = menu_item.variables.iter().collect();
       state.show_popup(PopupType::MenuVariableInput {
           menu_item_path: path.clone(),
           variable_name: vars[0].0.clone(),
           collected_vars: HashMap::new(),
           remaining_vars: vars[1..].iter().map(|(k,v)| (k.to_string(), v.to_string())).collect(),
       });
   } else {
       // Execute immediately
   }
   ```

---

### Session Collapse
**Účel:** Sbalení sessions v tree view pro přehlednost

**Změny:**
1. `src/app/state.rs` - přidat do `AppState`:
   ```rust
   /// Collapsed sessions (by session name)
   pub collapsed_sessions: std::collections::HashSet<String>,
   ```

2. `src/app/actions.rs` - přidat akci:
   ```rust
   ToggleSessionCollapse(String),  // session name
   ```

3. `src/ui/components/agent_tree.rs` - v renderování:
   ```rust
   // Group agents by session
   for (session, agents) in grouped {
       let is_collapsed = state.collapsed_sessions.contains(&session);
       
       // Render session header with collapse indicator
       let indicator = if is_collapsed { "▶" } else { "▼" };
       let agent_count = agents.len();
       let approval_count = agents.iter().filter(|a| a.status.needs_attention()).count();
       
       spans.push(Span::raw(format!("{} {} ({}", indicator, session, agent_count)));
       if approval_count > 0 {
           spans.push(Span::styled(format!(" ⚠{}", approval_count), Style::default().fg(Color::Yellow)));
       }
       
       if !is_collapsed {
           // Render agents
       }
   }
   ```

4. Key binding - `c` nebo `Enter` na session řádku toggle collapse

---

### Focus Outside Tmux
**Účel:** Klávesa `f` funguje i když tmuxx běží mimo tmux

**Změny v `src/ui/app.rs`** - v handling `f` key:
```rust
KeyAction::Focus => {
    if let Some(agent) = state.selected_agent() {
        if TmuxClient::is_inside_tmux() {
            // Existing: tmux select-pane
            tmux_client.focus_pane(&agent.target)?;
        } else if let Some(wrapper) = &state.config.terminal_wrapper {
            // Outside tmux: open new terminal with tmux attach
            let cmd = format!("tmux attach -t '{}'", agent.session);
            let wrapped = wrapper.replace("{cmd}", &cmd);
            let _ = std::process::Command::new("bash")
                .args(["-c", &wrapped])
                .spawn();
            state.set_status(format!("Opened terminal for {}", agent.session));
        } else {
            state.set_error("Cannot focus: not in tmux and no terminal_wrapper configured".to_string());
        }
    }
}
```

---

## 🔮 Nápady a Roadmap (Ideas)

- detekce zmeny stavu s agent na shell? asi drzet nejaky priznak a mozna je to na error alert, urcite kdyz tam je exit code

### AI Integrace
- **AI-Powered Workflows**: Analýza obrazovky pomocí AI a navrhování akcí
- **Context-aware Suggestions**: Návrhy příkazů na základě stavu agenta

### Hooky a Rozšíření
- **Hook System**: Spouštění skriptů při událostech (`on_approval_needed`, `on_error`, `on_idle`)
  - Config: `hooks: HashMap<String, String>` (event → command)
- **Plugin System**: Externí parsery agentů jako dynamické knihovny nebo skripty
- **Profiles**: Přepínání mezi sadami nastavení (`--profile work`)

### Pokročilá Detekce
- **Process Tree Analysis**: Detekce agentů přes kompletní strom procesů
- **SSH Remote Agents**: Detekce AI agentů běžících v SSH session
  - Vyžaduje: parsing SSH connection info, remote process detection
