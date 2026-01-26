# TODO - Tmuxx

## 🛠 Opravy (Fixes)
- [ ] **Správa Session**:
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
- [ ] **Focus (f) - Outside Tmux**: Automatické otevírání nového okna terminálu (Kitty, Alacritty) s připojením k session, pokud `tmuxx` běží mimo tmux.
- [ ] **Action Menu**: Komplexní systém konfigurovatelných akcí (proměnné, bash pipeline). Viz [TODO-MENU.md](TODO-MENU.md).
