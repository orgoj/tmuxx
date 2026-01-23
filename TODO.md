# TODO - tmuxcc

## Priority Tasks


### 1. Fix klávesy 'f' - neotvírá tmux session
**Status:** 🐛 Bug
**Problém:** Klávesa `f` má fokusovat/přepnout do vybrané tmux session, ale nefunguje
**Akce:**
- [ ] Debug: zjistit proč `f` key handler nefunguje
- [ ] Otestovat tmux send-keys/attach mechanismus
- [ ] Opravit a ověřit že funguje focus na vybranou session


### 2. Preview session špatně zobrazuje konec - chybí Claude prompty
- nove zjisteni - asi je to tim ze neresi sirku textu zalomuji se radky na screen v okne a pak se tam nevejde konec - over toto jako prvni vec
**Status:** 🐛 Bug
**Problém:** Session preview nezobrazuje konec pane obsahu → nejsou vidět approval prompty/menu
**Akce:**
- [ ] Debug: zjistit proč preview nezachytává konec pane
- [ ] Možná: capture_lines není dost? Nebo špatný offset?
- [ ] Fix: zobrazit správně poslední řádky s prompty
- [ ] Test: ověřit že vidíme "Do you want to allow this edit? [y/n]"

**Akce:**
- [ ] Přidat tui-textarea do Cargo.toml
- [ ] Prostudovat popup_placeholder.rs example z knihovny
- [ ] Implementovat modální popup dialog s TextArea
- [ ] Propojit s event handling (Esc zavře, Enter odešle)
- [ ] Nahradit současný input buffer tímto řešením
- [ ] Test: otevřít popup, zadat text, odeslat



### 3. Modální input dialog s text editorem
**Status:** ✅ Library selected - Ready to implement
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
