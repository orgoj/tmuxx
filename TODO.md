# TODO - tmuxcc

## 🛠 Opravy (Fixes)
- [ ] **Logika Session Tree**:
    - [ ] Opravit ořezávání zobrazení u posledního agenta v seznamu (musí být vidět všechny řádky window/pane, nejen první).
    - [ ] Přidat možnost zakázat cyklické procházení (z poslední položky na první a naopak) v `defaults.toml`.
    - [ ] Zpřesnit chování při filtrování: pokud se objeví noví agenti, první musí být automaticky označen. Pokud není vidět nic, pravý panel musí být prázdný.
    - [ ] Implementovat funkční klávesy Home/End pro skok na začátek/konec seznamu.
- [ ] **Správa Session**:
    - [ ] Implementovat funkci pro přejmenování aktuální tmux session.
    - [ ] Prověřit a opravit logiku "Kill Session" (vykazuje nestabilní chování).
    - [ ] Přidat příkaz pro uzavření celé session (vhodné zejména pro úklid po SSH připojeních).
- [ ] **Modal/Help Scrolling**: Opravit zavírání Help okna šipkami. V readonly režimu šipky nesmí hýbat kurzorem, ale pouze scrollovat text.
- [ ] **Preview Scrolling**: Implementovat plynulý scroll v preview oblasti s automatickým scrollováním na konec po zalomení textu.

## 💡 Drobnosti (Tweaks)
- [ ] **TODO Layout**: Přidat možnost zobrazit TODO sekci na plnou šířku (pokud je aktivní, pravý panel s aktivitou se nebude vykreslovat).
- [ ] **Session Collapse**: Možnost sbalit session v tree view (ponechat jen indikátory stavu). Vyžaduje logiku pro výběr celého session uzlu.
- [ ] **CLI Argumenty**: Přidat přímý argument `--filter <PATTERN>` (nyní nutno přes `--set filter_pattern=...`).
- [ ] **SSH Detection**: Výzkum spolehlivé detekce AI agentů běžících uvnitř SSH session.

## 🚀 Větší funkce (Features)
- [ ] **Notifikační systém**: Desktopové a terminálové upozornění na události vyžadující pozornost (approval, error).
- [ ] **Externí TODO Generátor**: Podpora pro externí programy (např. `beads`), které budou generovat obsah TODO okna dynamicky.
- [ ] **Focus (f) - Outside Tmux**: Automatické otevírání nového okna terminálu (Kitty, Alacritty) s připojením k session, pokud `tmuxcc` běží mimo tmux.
- [ ] **Action Menu**: Komplexní systém konfigurovatelných akcí (proměnné, bash pipeline). Viz [TODO-MENU.md](TODO-MENU.md).

---

## ✅ Hotovo (Completed)
- [x] **Plně modulární konfigurace**: Všechny defaulty jsou v `defaults.toml`, žádné hardcoded ikony v kódu.
- [x] **Univerzální Summary Parser**: Plně konfigurovatelné parsování výstupu pomocí regexů.
- [x] **Konfigurovatelný Highlight**: Syntax highlighting v náhledu definovaný v TOML.
- [x] **Rozšířená detekce procesů**: Detekce přes procesní strom a obsah paneu.
- [x] **Per-agent Keybindings**: Vlastní klávesy pro akce definované u každého agenta.
