# TODO - Tmuxx

- [ ] git status k sesson
  - u session vypisu to pouziji pres template
  - melo by to jit vypnout (pak ty promenne pro templated od git budou mit asi -)
  - musi to byt bezpecne a nesmi delat .git lock zamykani
  - udelat promenne pro vse co jde jednoduse a rychle z git zjistit
  - musi to mi cachovane hodnoty - aktualizace asi jen kdyz se prepnu do session, at to neni casto

- [ ] run command in windows dir
  - zepta se na command - input line dialog
  - default binding r (nebo ctrl-r)

- [ ]  input line dialog history
  - oddelena pro kazde pouziti (prompt/command)
  - persistentni ukladani do souboru vedle user config asi pro kazde pouziti jen txt, co command to radek

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
