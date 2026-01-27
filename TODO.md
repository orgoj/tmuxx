# TODO - Tmuxx

## 🛠 Opravy (Fixes)
- [x] prompt popup dialog nezobrazuje jak vybrat prompt Enter a Alt+Enter s hintem
- [x] v popupdialogu (prompr / menu) mid ve spodu boz, ktery bude zobrazovat cely command a nebo prompt
- [x] ve status bar maji byt jen errory cervene, ted je tam skoro vsechno cervene , normalni hlaseni maji byt zlene, opravit
- [ ] config reload ? jestli je to jednoduche, jinak presunout pozdeji (binding command)

## 💡 Drobnosti (Tweaks)
- [x] **TODO Layout**: Přidat možnost zobrazit TODO sekci na plnou šířku (pokud je aktivní, pravý panel s aktivitou se nebude vykreslovat). Defautl on.
- [ ] **Notifikační systém**: Desktopové a terminálové upozornění na události vyžadující pozornost (approval, error). Mozna jen volani cmd na poslani notifikace a s definovatelnym spozdenim (1min). Pro kazde window zapsat cas vzniku aproval a kdyz to prekroci ten cas tak posilat notifikaci.
- [ ] **SSH Detection**: Výzkum spolehlivé detekce AI agentů běžících uvnitř SSH session.
  - [ ] pro zacatek jen nejaky idikator i windows ze je v process ssh, to by mozna stacil config
  - [ ] pak tento ukol dej nakonec a musime udelat nejak lepsi praci s ssh aby jsme umeli detekovat remote agenta v ssh
- [ ] **Vylepšený init-config**: `--init-config` by měl zapsat `defaults.toml` včetně komentářů (z `include_str!`), ne jen serializovaný struct.

## 🚀 Větší funkce (Features)
- [ ] **Externí TODO Generátor**: Podpora pro externí programy (např. `beads`), které budou generovat obsah TODO okna dynamicky.
- [ ] **Action Menu**: Komplexní systém konfigurovatelných akcí (proměnné, bash pipeline). Zozsirni stavajici definice.
- [ ] **Session Collapse**: Možnost sbalit session v tree view (ponechat jen indikátory stavu). Vyžaduje logiku pro výběr celého session uzlu.
- [ ] **Focus (f) - Outside Tmux**: Automatické otevírání nového okna terminálu (Kitty, Alacritty) s připojením k session, pokud `tmuxx` běží mimo tmux.

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
