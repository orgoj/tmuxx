---
title: Global Highlight Rules
status: closed
priority: 1
issue-type: task
created-at: "\"\\\"\\\\\\\"\\\\\\\\\\\\\\\"2026-01-29T21:24:21.586589+01:00\\\\\\\\\\\\\\\"\\\\\\\"\\\"\""
closed-at: "2026-01-29T21:51:24.591683+01:00"
close-reason: Verified Global Highlight Rules implementation with a new test case and confirmed all tests pass.
---

Účel: Globální pravidla pro zvýraznění error/fail/exception ve všech agentech.

Změny:
1. 'src/app/config.rs' - přidat do 'Config':
   pub global_highlight_rules: Vec<HighlightRule>,

2. 'src/parsers/universal.rs' - v 'UniversalParser::new' (řádek 151+) a v 'highlight_line':
   Upravit konstruktor, aby přijímal 'global_rules: &[HighlightRule]'.
   Tyto pravidla zkompilovat a přidat do 'self.highlight_rules' (jako fallback na konec seznamu).

3. 'src/parsers/mod.rs' - v 'ParserRegistry::with_config':
   Předávat 'config.global_highlight_rules' do 'UniversalParser::new'.

4. 'src/config/defaults.toml' - přidat výchozí globální pravidla:
   [[global_highlight_rules]]
   pattern = "(?i)error"
   color = "red"
   modifiers = ["bold"]
   ...atd. (fail, exception, panic)

5. 'src/ui/components/pane_preview.rs' - vyčistit hardcoded pravidla v 'render_detailed' (~řádek 335), která nyní budou v defaults.toml.
