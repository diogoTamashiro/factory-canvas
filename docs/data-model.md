# Data model v1 — Factory Canvas

> Approved evolution contract for CAD documents and the game-data package. This document does not claim that the model is already implemented in the current domain.

## Layer separation

```text
Game-data package
        │
        ▼
Static definitions ───► Factory document ───► Local blueprints
        │                        │                       │
        └── capabilities          └── entities           └── relative copies
```

- **Game data:** provided and versioned by Diogo.
- **Factory document:** spatial state of one complete base.
- **Blueprint:** production module saved locally and inserted as an independent copy.

## Identifiers

All persisted data IDs use stable ASCII `snake_case` strings independent of display names:

- `BuildableId`, for example `refinery_unit`;
- `ProductId`, for example `processed_xiranite`;
- `PortTypeId`, for example `item`;
- `BaseId` and `RegionId`;
- `BlueprintId`.

`EntityId` remains a monotonic identifier local to the factory. Blueprints use a local `BlueprintEntityId` and never reuse an `EntityId` from the source factory.

## Modular data package

Phase 3 implements runtime catalog schema v1 with a fixed set of four required modules. The versioned public package lives under `catalog/public/`; a complete private package may be loaded from `data/catalog/` at startup in a later integration commit.

```text
catalog/
  manifest.json
  regions.json
  bases.json
  buildables.json
  products.json
```

`manifest.json` contains:

```json
{
  "schema_version": 1,
  "catalog_id": "factory_canvas_public",
  "data_version": "0.1.0",
  "display_name": "Factory Canvas — Public Catalog",
  "default_base_id": "wuling_main",
  "modules": {
    "regions": "regions.json",
    "bases": "bases.json",
    "buildables": "buildables.json",
    "products": "products.json"
  }
}
```

The four module files use strict root wrappers named `regions`, `bases`, `buildables`, and `products`. Their current definitions are:

- regions: `id`, `display_name`;
- bases: `id`, `display_name`, `region_id`, `width`, `height`;
- buildables: `id`, `display_name`, `category`, `symbol`, `footprint`, `production_targets`;
- products: `id`, `display_name`.

Unknown or missing fields, malformed JSON, unsupported schema versions, invalid identifiers, nonpositive or overflowing dimensions, unsafe module paths, missing references, and duplicate IDs are rejected before a `Catalog` snapshot is returned. Module paths must remain relative to the package root; parent traversal, absolute/rooted paths, repeated paths, and symlink escapes are invalid. The loader never combines files from public and private sources.

`port_types.json` and `rules.json` remain possible future extensions. They are not accepted runtime inputs in schema v1.

`data_version` follows SemVer:

- `MAJOR`: incompatible IDs, semantics, or contract;
- `MINOR`: new compatible data or capabilities;
- `PATCH`: compatible data correction.

## Constructible entity

Machines, conveyors, power poles, and future components use the same spatial definition:

```text
BuildableDefinition
  id: BuildableId
  display_name
  category: CategoryId
  symbol
  footprint
  production_targets[]
```

The category differentiates future behavior but does not create a second placement, rotation, bounds, or collision system. Physical ports and capabilities beyond the validated `production_targets` list remain planned extensions.

## Planned physical ports

```text
PortDefinition
  id: PortId local to the buildable
  anchor: { tile, side }
  flow: Input | Output | Bidirectional
  port_type: PortTypeId
```

- `tile` is a relative coordinate inside the unrotated footprint;
- `side` is the physical `North`, `East`, `South`, or `West` face;
- `flow` describes the logical direction;
- `port_type` describes the transported type.

Entity rotation transforms `anchor` and `side` through a single domain geometry rule. In this phase, ports do not validate connections, recipes, rates, or flow.

## Positioned entity

```text
PlacedEntity
  id: EntityId
  buildable_id: BuildableId
  origin
  rotation
  production_target: Option<ProductId>
  future configuration
```

`production_target` is the user's choice on the instance. The static definition says whether the entity can produce and which products it can offer; the first increment does not calculate inputs, outputs, recipes, or throughput.

## Factory document

```text
FactoryDocument
  schema_version
  catalog_data_version
  metadata
  base
  entities[]
  optional viewport/editor metadata
```

The document records `catalog_data_version` for provenance. A version difference initially produces a warning and never overwrites or blocks the document automatically.

## Blueprint document

```text
BlueprintDocument
  schema_version
  catalog_data_version
  blueprint_id
  name
  optional description
  nodes[]
  interfaces[]
  metadata
```

- `nodes[]` uses origins normalized relative to the selection;
- inserting a blueprint creates new entities with new monotonic IDs;
- the selection is literal: no external component is pulled in automatically;
- an interface represents a physical port in the selection that is open outward;
- interfaces can receive a user-defined name but do not assert a connected conveyor, flow, or confirmed compatibility.

## Persistence

Factories and blueprints are separate, readable local JSON files. Each format has its own `schema_version`.

Migration and save follow these steps:

1. read and identify the version;
2. migrate in memory when supported;
3. validate the complete document;
4. serialize to a temporary file on the same volume;
5. synchronize when applicable;
6. rename atomically;
7. report success only after the rename.

Unknown, incompatible, or invalid files do not replace the currently open document.

## Outside model v1

- conveyor connectivity graph;
- adjacent-port validation;
- recipes and production balance;
- throughput and bottlenecks;
- active regional rules;
- live link between a blueprint and an inserted instance;
- solver and auto-layout.
