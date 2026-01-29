# TODO - Tmuxx

## 🔮 Nápady a Roadmap (Ideas)

### AI Integrace
- **AI-Powered Workflows**: Analýza obrazovky pomocí AI a navrhování akcí
- **Context-aware Suggestions**: Návrhy příkazů na základě stavu agenta

### Ostatní
- stav start
- detekce zmeny stavu s agent na shell? asi drzet nejaky priznak a mozna je to na error alert, urcite kdyz tam je exit code
- nejaku box s tlacitky definovatelnymi (promty/commandy do aktivniho okna) - pro ovladani jen klikanim mysi
- cli rozhrani - json vystupo stavu terminalu - aby se dalo pouzit ve scriptech ta detekce stavu

### Hooky a Rozšíření
- **Hook System**: Spouštění skriptů při událostech (`on_approval_needed`, `on_error`, `on_idle`)
  - Config: `hooks: HashMap<String, String>` (event → command)
