# ADR 0002 — Rename the product to Factory Canvas

- **Status:** Accepted
- **Date:** 2026-08-19
- **Decider:** Diogo

## Context

The project used the provisional names `softFactory` and Graph Planner. The former can suggest a software factory, while the latter can be mistaken for a graphing or mathematical analysis tool.

The current product is a native, offline 2D visual editor for arranging factory blocks on a canvas. The name needs to communicate the industrial domain and its focus on visual planning without depending directly on a game brand.

## Decision

- the product is renamed **Factory Canvas**;
- the local directory and GitHub repository use the `factory-canvas` slug;
- the Cargo package and default executable use `factory-canvas`;
- the application displays `Factory Canvas — Arknights: Endfield` as its window title;
- references to the previous name in product documents are updated;
- ADR 0001 preserves the historical record of the earlier naming decision.

## Consequences

### Positive

- the name directly communicates a visual tool for factories;
- it does not suggest that the project builds software;
- it is not tied to the technical term graph;
- the slug is simple and consistent across the folder, repository, package, and executable.

### Negative

- links, clones, and scripts that pointed to the old repository or folder need to use `factory-canvas`;
- the Rust crate changes from `graph_planner` to `factory_canvas`;
- the legacy SQLite file `softfactory.db` remains for compatibility, but its name no longer appears in the interface.
