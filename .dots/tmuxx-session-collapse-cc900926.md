---
title: Session Collapse
status: open
priority: 4
issue-type: task
created-at: "2026-01-29T16:48:42.310204+01:00"
---

Účel: Sbalení sessions v tree view pro přehlednost. Ovládání přes bindovatelné akce.

Změny:
1. 'src/app/state.rs' - přidat do 'AppState':
   pub collapsed_sessions: std::collections::HashSet<String>,

2. 'src/app/actions.rs' - přidat akce:
   TreeExpand, TreeCollapse, TreeToggle

3. 'src/ui/components/agent_tree.rs' - renderování:
   Pokud je session collapsed, vykreslit indikátor '▶' a přeskočit agenty.
   Pokud není, vykreslit '▼'. Zobrazovat sumární '⚠' (needs_attention) i u sbalené session.

4. 'src/config/defaults.toml':
   Mapovat '+' na TreeExpand, '-' na TreeCollapse a '*' na TreeToggle.
