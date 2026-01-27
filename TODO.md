# TODO - Tmuxx

## 🛠 Opravy (Fixes)
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
- [ ] **Action Menu**: Komplexní systém konfigurovatelných akcí (proměnné, bash pipeline).

### Configuration
- [ ] **Vylepšený init-config**: `--init-config` (nebo `--write-config`) by měl zapsat `defaults.toml` včetně komentářů (z `include_str!`), ne jen serializovaný struct.

## 🔮 Nápady a Roadmap (Ideas)

### AI Integrace
- [ ] **AI-Powered Workflows**: Analýza obrazovky pomocí AI a navrhování akcí.
  - Příklad: Capture screen -> Send to Claude -> Show fix -> Paste to pane.
- [ ] **Context-aware Suggestions**: Návrhy příkazů na základě stavu agenta.

### Notifikace a Hooky
- [ ] **Desktop Notifications**: `notify-send` nebo nativní notifikace při chybě/požadavku na schválení.
- [ ] **Hook System**: Spouštění skriptů při událostech (např. `approval_needed`, `agent_error`).
- [ ] **Event Filtering**: Notifikovat jen akční události, ne informační.

### Konfigurace a Rozšíření
- [ ] **Config Hot Reload**: Automatické načtení změn v `config.toml`.
- [ ] **Plugin System**: Možnost přidávat nové parsery agentů jako externí moduly/skripty.
- [ ] **Profiles**: Rychlé přepínání mezi sadami nastavení (např. "Work", "Home").

### Pokročilá Detekce
- [ ] **Process Tree Analysis**: Detekce agentů přes analýzu stromu procesů (nejen přímý command).
- [ ] **Parent Process Detection**: Lepší detekce wrapperů.
