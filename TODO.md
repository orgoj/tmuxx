# TODO - tmuxcc

## Completed Tasks ✅

### Custom Agent Patterns & Detection
**Status:** ✅ HOTOVO (2026-01-23)
**Implementace:**
- ✅ CustomAgentParser s regex matching
- ✅ Wildcard pattern `*` pro všechny panes
- ✅ Config integration (agent_patterns v TOML)
- ✅ AgentType::Custom(String) variant
- ✅ Priority: built-in parsers first, pak custom patterns
- ✅ Documentation v README.md
- ✅ Testováno: wildcard detekuje všechny panes

**Soubory změněny:**
- `src/parsers/custom.rs` - nový CustomAgentParser
- `src/parsers/mod.rs` - ParserRegistry.with_config()
- `src/agents/types.rs` - AgentType::Custom variant
- `src/ui/app.rs` - předání config do registry
- `src/ui/components/agent_tree.rs` - Custom color handling

### Cross-Session Focus (klávesa 'f' uvnitř tmux)
**Status:** ✅ HOTOVO (2026-01-23)
**Implementace:**
- ✅ Detekce current vs target session
- ✅ Same-session: select-window + select-pane
- ✅ Cross-session: tmux switch-client
- ✅ Error když běží mimo tmux
- ✅ Testováno: z ct-test do cc-tmuxcc funguje

**Soubory změněny:**
- `src/tmux/client.rs` - focus_pane() s cross-session support

---

## Priority Tasks

### 1. Focus klávesa 'f' - Outside Tmux Support
**Status:** ✅ VYŘEŠENO JEDNODUŠŠÍM ZPŮSOBEM (2026-01-23)

**Co funguje:**
- ✅ Inside tmux, same session - funguje
- ✅ Inside tmux, cross-session - funguje (switch-client)
- ✅ Outside tmux - vyřešeno **wrapper scriptem** (jednodušší než terminal launcher)

**Řešení:** Wrapper script `scripts/tmuxcc-wrapper.sh`
- Automaticky zajišťuje že tmuxcc VŽDY běží uvnitř tmux session `tmuxcc`
- Pokud session neexistuje, vytvoří ji
- Pokud běžíš inside tmux: switch-client do tmuxcc session
- Pokud běžíš outside tmux: attach do tmuxcc session
- Eliminuje problém "outside tmux" zcela

**Použití:**
```bash
# Symlink do ~/bin
ln -sf $(pwd)/scripts/tmuxcc-wrapper.sh ~/bin/tcc

# Spustit wrapper místo přímého tmuxcc
tcc
```

**Poznámka:** Původní plán (Step 6) s platform-specific terminal launcherem je ZBYTEČNÝ.
Wrapper script je jednodušší, spolehlivější, a cross-platform.

**Soubory:**
- `scripts/tmuxcc-wrapper.sh` - wrapper script
- `README.md` - dokumentace použití


### 2. Preview session špatně zobrazuje konec - chybí Claude prompty
**Status:** 🐛 Bug
**Problém:** Session preview nezobrazuje konec pane obsahu → nejsou vidět approval prompty/menu
**Poznámka:** Možná je to tím že neřeší šířku textu - zalomují se řádky na screen v okně a pak se tam nevejde konec

**Akce:**
- [ ] Debug: zjistit proč preview nezachytává konec pane
- [ ] Možná: capture_lines není dost? Nebo špatný offset?
- [ ] Ověřit teorii o šířce textu a zalamování
- [ ] Fix: zobrazit správně poslední řádky s prompty
- [ ] Test: ověřit že vidíme "Do you want to allow this edit? [y/n]"


### 3. Modální input dialog s text editorem
**Status:** ✅ Library selected - Ready to implement
**Akce:**
- [ ] Přidat tui-textarea do Cargo.toml
- [ ] Prostudovat popup_placeholder.rs example z knihovny
- [ ] Implementovat modální popup dialog s TextArea
- [ ] Propojit s event handling (Esc zavře, Enter odešle)
- [ ] Nahradit současný input buffer tímto řešením
- [ ] Test: otevřít popup, zadat text, odeslat

**Problém:** Současný input buffer má chyby, potřebujeme modální dialog s kvalitním editorem
**Řešení:** Použít **tui-textarea** knihovnu (by rhysd)

**Vybraná knihovna: tui-textarea**
- Repo: https://github.com/rhysd/tui-textarea
- Docs: https://docs.rs/tui-textarea
- Podporuje ratatui 0.29 ✅
- Má popup example! (examples/popup_placeholder.rs)
- Features: multi-line, undo/redo, selection, search, Emacs shortcuts

**Instalace:**
```toml
tui-textarea = "*"
```

### 4. Statusline u session + přesunout input do modálního dialogu
**Status:** 🎨 UI Enhancement
**Problém:** Input buffer zabírá místo kde by mohla být statusline pro session
**Řešení:**
- Odstranit always-visible input buffer z layoutu
- Přidat statusline pro vybranou session (status, kontext %, aktivita)
- Input přesunout do modálního dialogu (viz úkol #1)
**Akce:**
- [ ] Navrhnout layout: kde bude statusline, co zobrazí
- [ ] Implementovat statusline pro session (podobně jako header)
- [ ] Odstranit input buffer z main layoutu
- [ ] Propojit s modálním input dialogem z úkolu #1

---

## Notes
- Před implementací VŽDY hledat hotové knihovny přes web search
- Používat rtfmbro MCP pro dokumentaci knihovny
- Nepsát věci od nuly když existují kvalitní knihovny
