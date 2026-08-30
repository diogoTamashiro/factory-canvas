# ADR 0003 — CAD documents, blueprints, and versioned data

- **Status:** Accepted
- **Date:** 2026-08-21
- **Decider:** Diogo

## Context

Factory Canvas began as a 2D spatial-occupancy editor for a base. The product is evolving into an offline factory CAD tool: the player must be able to design a complete factory, focus on subsets, configure the product for each machine, and save reusable production modules.

Game data — PACs, constructible entities, products, ports, regions, and mechanics — will be maintained and versioned by Diogo. It must not be coupled to the canvas, the user document, or production rules that have not yet been confirmed.

The public repository also does not store private reference data. The public format must therefore describe contracts and schemas without depending on private files.

## Decision

- separate the factory document (`FactoryDocument`) from the reusable module definition (`BlueprintDocument`);
- persist both as readable local JSON, each with `schema_version` and explicit migrations;
- use a modular game-data package with a manifest and a SemVer `data_version` controlled by Diogo;
- unify machines, conveyors, power poles, and future components as positionable constructible entities;
- store the product choice on each positioned entity, while the static definition declares only capabilities;
- save blueprints as independent copies in relative coordinates, with no live link to the original factory or blueprint;
- treat every physical port exposed at a selection boundary as a nameable blueprint interface without asserting a confirmed connection or flow;
- keep recipe validation, conveyor connectivity, throughput, active regional rules, and the solver outside the first implementation of these documents.

The detailed contract is in [`docs/data-model.md`](../data-model.md).

## Phase 3 implementation note — 2026-08-29

The first executable runtime-catalog schema uses a strict manifest with `schema_version`, `catalog_id`, SemVer `data_version`, `display_name`, `default_base_id`, and fixed `modules` entries for regions, bases, buildables, and products. Both embedded public data and a directory package pass through the same all-or-nothing decoder.

This is the currently implemented subset of the modular-package decision. Port types, rules, document persistence, migrations, and blueprint data remain planned; they are not accepted silently as runtime catalog fields in schema v1.

## Consequences

### Positive

- the CAD tool can evolve independently of game-data collection;
- blueprints are portable and repeatable without reusing factory IDs;
- the format is auditable, migratable, and suitable for offline work;
- the catalog, user document, and interface retain explicit boundaries;
- conveyors can use the same spatial system without creating a parallel model.

### Negative

- the future migration from static `BlockTemplate`/`BlockInstance` values to data IDs requires its own phase and compatibility tests;
- versioned JSON introduces responsibility for migrations and atomic saves;
- blueprint interfaces initially represent exposed ports, not actual connectivity;
- data versions must be maintained carefully to preserve document provenance.

## Alternatives considered

### One document for factories and modules

Rejected because a blueprint would stop being an independent reusable unit and would retain IDs/state from the source factory.

### SQLite as the primary document format

Rejected for the first version because separate JSON is more readable, portable, easy to version, and sufficient for the offline local library. SQLite could later be used as an index of recent items or for search, without replacing the portable documents.

### Connect ports directly in the first model

Rejected because the game uses player-placed conveyors. The first version needs to represent spatial entities and physical ports without inventing topology, flow, or compatibility rules that have not yet been confirmed.

## Review

Review this ADR when conveyor connectivity or the first `schema_version` migration requires an incompatible change to the document contract.
