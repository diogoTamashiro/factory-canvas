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

```text
catalog/
  manifest.json
  bases.json
  buildables.json
  products.json
  port_types.json
  regions.json
  rules.json
```

`manifest.json` contains:

```json
{
  "schema_version": 1,
  "data_version": "0.1.0",
  "files": [
    "bases.json",
    "buildables.json",
    "products.json",
    "port_types.json",
    "regions.json",
    "rules.json"
  ]
}
```

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
  category: Machine | Conveyor | Power | ...
  footprint
  ports[]
  capabilities
```

The category differentiates future behavior but does not create a second placement, rotation, bounds, or collision system.

## Physical ports

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
