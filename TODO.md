# TODO - Tmuxx

- [ ] git status k sesson
  - u session vypisu to pouziji pres template
  - melo by to jit vypnout (pak ty promenne pro templated od git budou mit asi -)
  - musi to byt bezpecne a nesmi delat .git lock zamykani
  - udelat promenne pro vse co jde jednoduse a rychle z git zjistit
  - musi to mi cachovane hodnoty - aktualizace asi jen kdyz se prepnu do session, at to neni casto

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

## Ideas

### Zrusitelne notifikace

Trackovat notification id pro windows a to umozni jeji zruseni. Bude treba prikaz, ktery vraci  id, trackovat jej a pouzivat pro dalsi notifikace.

> notify-send --help
Usage:
  notify-send [OPTION…] <SUMMARY> [BODY] - create a notification

Help Options:
  -?, --help                        Show help options

Application Options:
  -u, --urgency=LEVEL               Specifies the urgency level (low, normal, critical).
  -t, --expire-time=TIME            Specifies the timeout in milliseconds at which to expire the notification.
  -a, --app-name=APP_NAME           Specifies the app name for the icon
  -i, --icon=ICON                   Specifies an icon filename or stock icon to display.
  -c, --category=TYPE[,TYPE...]     Specifies the notification category.
  -e, --transient                   Create a transient notification
  -h, --hint=TYPE:NAME:VALUE        Specifies basic extra data to pass. Valid types are boolean, int, double, string, byte and variant.
  -p, --print-id                    Print the notification ID.
  -r, --replace-id=REPLACE_ID       The ID of the notification to replace.
  -w, --wait                        Wait for the notification to be closed before exiting.
  -A, --action=[NAME=]Text...       Specifies the actions to display to the user. Implies --wait to wait for user input. May be set multiple times. The name of the action is output to stdout. If NAME is not specified, the numerical index of the option is used (starting with 0).
  -v, --version                     Version of the package.
