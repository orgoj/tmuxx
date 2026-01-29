---
title: Global Highlight Rules
status: open
priority: 1
issue-type: task
created-at: "2026-01-29T16:48:42.255206+01:00"
---

Účel: Globální pravidla pro zvýraznění error/fail/exception ve všech agentech. Žádná hardcoded pravidla v kódu.

Změny:
1. 'src/app/config.rs' - přidat do 'Config' (~řádek 95):
   pub global_highlight_rules: Vec<HighlightRule>,

2. 'src/ui/components/pane_preview.rs' - v renderování (~řádek 200) implementovat merge:
   let all_rules: Vec<_> = agent_config.highlight_rules.iter().chain(state.config.global_highlight_rules.iter()).collect();

3. 'src/config/defaults.toml' - přesunout sem veškerou logiku:
   [[global_highlight_rules]]
   pattern = "(?i)error"
   color = "red"
   modifiers = ["bold"]

   [[global_highlight_rules]]
   pattern = "(?i)fail(ed|ure)?"
   color = "red"

   [[global_highlight_rules]]
   pattern = "(?i)(traceback|exception|panic)"
   color = "yellow"
   modifiers = ["bold"]
