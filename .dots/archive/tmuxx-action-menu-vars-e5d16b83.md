---
title: Action Menu Variables
status: closed
priority: 3
issue-type: task
created-at: "\"2026-01-29T16:48:42.303139+01:00\""
closed-at: "2026-01-29T22:58:50.405358+01:00"
---

Účel: Menu položky s input prompty pro proměnné, sjednoceno s existujícím systémem expansion.

Změny:
1. 'src/app/menu_config.rs' - přidat do 'MenuItem':
   pub variables: std::collections::HashMap<String, String>, // Key: var name, Value: prompt text

2. 'src/app/state.rs' - přidat nový 'PopupType':
   MenuVariableInput {
       menu_item_path: Vec<usize>,
       variable_name: String,
       collected_vars: std::collections::HashMap<String, String>,
       remaining_vars: Vec<(String, String)>,
   }

3. 'src/ui/app.rs' - při Execute menu item:
   Pokud !menu_item.variables.is_empty(), spustit sekvenci popupů.
   Po sesbírání všech hodnot provést expanzi v příkazu (stejná logika jako '{session}') a spustit.
