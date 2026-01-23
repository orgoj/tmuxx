# tmuxcc Vision & Future Ideas

## 🎯 Current State: Multi-AI Agent Dashboard

**What we have:**
- Multi-AI support: Claude Code, OpenCode, Codex CLI, Gemini CLI
- Real-time agent monitoring across all tmux sessions
- Approval management with batch operations
- Subagent tracking (Claude Code Task tool)
- Multi-agent selection and batch approvals
- Hierarchical tree view (Session/Window/Pane)
- Context awareness (remaining context %)
- Configurable via TOML

**What's working well:**
- AgentParser trait provides clean extensibility for new agents
- Process detection via multiple strategies (command, title, child processes)
- Clean separation: tmux layer, parser layer, application layer

---

## 🚀 Major Features Roadmap

### 1. Advanced Multi-AI Support (Partially Done ✅)

**Current:**
- ✅ Claude Code (full detection, subagents, context tracking)
- ✅ OpenCode (basic detection)
- ✅ Codex CLI (basic detection)
- ✅ Gemini CLI (basic detection)
- ✅ AgentParser trait for extensibility

**Future:**
- 🎯 Pi AI support
- 🎯 Aider support
- 🎯 Generic "Any AI agent" detection with config templates
- 🎯 Agent-specific features (each AI has unique capabilities)
- 🎯 Plugin system for community-contributed agent parsers

### 2. Standalone Panel Architecture

- ✅ Currently: Works as tmux TUI application
- 🎯 Target: Can run independently outside tmux context
- 🎯 Central control for ALL tmux sessions from single interface
- 🎯 Multiple view modes: popup (current), split pane, full window

### 3. Advanced Configuration System

**Current:**
- ✅ TOML config with poll interval, capture lines
- ✅ Custom agent patterns (empty = defaults)

**Future multi-level hierarchy:**
```
Global config (~/.config/tmuxcc/)
  ↓
Project-specific config (./.tmuxcc.toml)
  ↓
Agent-type config (claude/gemini/pi specific)
  ↓
Session-name specific config (pattern matching)
```

**Configurable aspects (planned):**
- Custom key bindings
- Visual themes and colors
- Agent-specific approval workflows
- Filters and search patterns
- Preview layout and size

### 4. Integrated AI Processing

**Vision: AI-powered workflows within tmuxcc itself**

**AI Integration Points:**
- Screen capture → AI analysis → suggested actions
- Content extraction → AI summarization → routing
- Multi-session batch operations with AI guidance
- Context-aware command suggestions based on agent state
- Automatic error detection and resolution suggestions

**Example workflow:**
```
1. Capture content from agent showing error
2. Send to Claude Haiku with "Analyze this error"
3. Display suggested fix in tmuxcc
4. User can paste fix to agent with one keystroke
```

### 5. Configurable Command Pipelines

**Powerful pipe system for custom workflows**

**Example 1: Screen → AI → Editor → Paste**
```yaml
pipes:
  - name: "ai-screen-edit"
    trigger: "Ctrl+e"
    steps:
      - capture: selected_pane
      - ai: claude-haiku
        prompt: "Refactor this code..."
      - editor: wait  # Open in $EDITOR
      - paste: selected_pane
```

**Example 2: Multi-session batch operation**
```yaml
pipes:
  - name: "batch-test"
    steps:
      - select: filter="status:idle"
      - send_keys: "npm test"
      - wait: 5s
      - capture: all
      - report: summary
```

### 6. Advanced Session Management

**Features:**
- 🎯 Quick control of multiple sessions simultaneously
- 🎯 Search and filter across all sessions
- 🎯 Batch operations (send command to multiple sessions)
- 🎯 Session grouping and tagging
- 🎯 Context-aware session detection (project type, Git status)
- 🎯 Session templates and quick-start configs

### 7. Menu System (Future)

**Fully configurable, context-aware menus:**
- Global menus (always available)
- Project-specific menus (detected by path)
- Agent-type menus (Claude vs Gemini vs Pi)
- Session-specific menus (per session name pattern)
- Dynamic menus based on session state

---

## 🎨 Architecture Implications

### Configuration Management (Planned)
- Multi-level config merging (global → project → agent → session)
- Schema validation with clear error messages
- Hot reload support (watch config file changes)
- Config profiles/presets for different workflows

### Plugin System (Future)
- AI backend plugins (new agent parsers)
- Tool detector plugins (extend detection logic)
- Command pipeline plugins (custom workflow steps)
- Menu provider plugins (custom UI extensions)

### Performance Considerations
- ✅ Process cache with 500ms refresh (current)
- ✅ Efficient polling with configurable intervals
- 🎯 Async operations for AI calls (future)
- 🎯 Batched tmux commands (reduce overhead)
- 🎯 Smart preview updates (only when visible)

---

## 🔮 Long-term Ideas

### Potential Extensions
- Remote tmux session management (SSH to other machines)
- Session recording and playback
- AI-driven session recommendations
- Visual workflow builder (TUI config editor)
- Export/share workflows and configs
- Community config repository

### Integration Possibilities
- Git status/operations (show branch, dirty state)
- Docker container management
- Process monitoring (CPU, memory per agent)
- Log analysis and filtering
- Multi-machine orchestration

### Quality of Life
- Session search with fuzzy matching
- Bookmarks/favorites for frequent sessions
- History of approvals (audit trail)
- Statistics dashboard (agent usage, approvals over time)
- Notification system (desktop notifications for approvals)

---

## 📝 Naming & Branding

**Current:** tmuxcc (tmux + Claude Code / Control Center)
- Works well as fork name
- Reflects tmux integration
- "cc" suggests control/command center

**If rebranding later:**
- Should reflect multi-AI nature
- Convey power/flexibility
- Easy to remember and type

---

## 📚 Documentation Needs

**Current:**
- README.md with basic usage
- CLAUDE.md for development guidance

**Future needs:**
- User guide for advanced features
- Configuration reference (all options)
- Pipe system guide with cookbook examples
- Plugin development guide
- Agent detector development guide
- Video tutorials / screencasts

---

## ⚠️ Backward Compatibility

**DECISION: Minimal Compatibility Concerns**

This is a fork for personal/team use:
- ✅ Can break things during development
- ✅ Fast iteration without constraints
- ✅ Clean slate for new features
- ⚠️ Consider upgrade path when adding major features
- ⚠️ Config file changes should have migration helper

---

## 🎯 Active Development Ideas

### Hierarchical Configuration System
**Priority:** High
**Status:** Planned

**Vision:** Config loading z více úrovní s automatickým mergem

**Hierarchie:**
```
~/.config/tmuxcc/config.toml    (global config)
  ↓
~/.tmuxcc.toml                   (user-level override)
  ↓
/path/to/project/.tmuxcc.toml   (project-specific)
  ↓
/path/to/project/subdir/.tmuxcc.toml (session-specific based on cwd)
```

**Potřebné funkce:**
```rust
// Config resolver - dáš mu adresář, vrátí merged config
fn resolve_config(session_cwd: &Path) -> Config {
    // 1. Načti global config
    // 2. Walk up from session_cwd a merguj všechny .tmuxcc.toml
    // 3. Merge priority: nejbližší k session_cwd má přednost
}
```

**Use case:**
- Různé projekty mají různé polling intervaly
- Projekt může definovat custom agent patterns
- Subdirectory může mít specifické nastavení

**Implementation notes:**
- Watch file changes pro hot reload
- Clear merge priority (child overrides parent)
- Validation při načítání každého levelu

---

### Konfigurovatelné Menu Akcí per Session
**Priority:** High
**Status:** Planned (relates to AI pipelines vision)

**Vision:** Definovat custom akce pro konkrétní sessions v configu

**Config format (návrh):**
```toml
[[session_menu]]
pattern = "^cc-.*"  # Regex pro session name
name = "Claude Code Menu"

[[session_menu.action]]
key = "t"
label = "Run tests"
command = "npm test"

[[session_menu.action]]
key = "b"
label = "Build project"
command = "cargo build --release"
interactive = true  # Čekat na input před executí?

[[session_menu.action]]
key = "e"
label = "AI Error Analysis"
pipeline = "ai-analyze-error"  # Reference to AI pipeline
```

**Features:**
- Pattern matching na session names
- External command execution (send-keys to session)
- Input prompts před executí
- AI pipeline integration (budoucnost)
- Dynamické menu based on config

**Architecture:**
- Menu provider trait
- Config-driven menu builder
- Action executor (tmux send-keys wrapper)

---

## 🎯 Next Priorities (User Will Define)

User will assign tasks incrementally. Current foundation is solid:
- Multi-agent monitoring ✅
- Approval management ✅
- Subagent tracking ✅
- Configurable behavior ✅

Immediate improvements in progress (see TODO.md):
- Modální input dialog with proper text editor
- Fix 'f' key focus functionality
- Fix preview showing end of pane content
- Add statusline for session info

---

*This document captures the long-term vision inspired by tmuxclai-arch.*
*Implementation will be gradual and user-directed.*
*Update this document as features are completed and vision evolves.*
