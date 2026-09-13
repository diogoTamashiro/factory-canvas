# Data model v1 — Factory Canvas

> Runtime `Catalog` schema v1, `BlockInstance.production_target`, `FactoryDocument`, `BlueprintDocument`, and their local persistence (including the local blueprint library) are implemented. Physical ports and blueprint insertion remain contracts for Phase 5.

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

The runtime catalog implements typed `CatalogId`, `RegionId`, `BaseId`, `BuildableId`, `ProductId`, and `CategoryId` strings. Each value is nonempty ASCII `snake_case`, begins with a lowercase letter, and has no repeated or trailing underscore. IDs are independent of display names. The public package includes values such as `factory_canvas_public`, `wuling`, `wuling_main`, `refinery_unit`, and `production_i`; it intentionally defines no `ProductId` values yet.

`PortTypeId` and `PortId` are planned port types, not yet accepted catalog-schema-v1 or document fields. `BlueprintId` and `BlueprintEntityId` are implemented document-level identifiers — not catalog-schema-v1 fields either, but real types used by `BlueprintDocument` and the blueprint library today.

`EntityId` remains a monotonic identifier local to the factory. Blueprints use a local `BlueprintEntityId` and never reuse an `EntityId` from the source factory.

## Modular data package

Phase 3 implements runtime catalog schema v1 as one manifest and four required modules relative to a package root:

```text
<package-root>/
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

All manifest, module-wrapper, item, and footprint objects reject unknown fields. Every shown field is required, except `buildables`' optional `icon`. The four module files use strict root wrappers named `regions`, `bases`, `buildables`, and `products`. Their definitions are:

- regions: `id`, `display_name`;
- bases: `id`, `display_name`, `region_id`, `width`, `height`;
- buildables: `id`, `display_name`, `category`, `symbol`, `footprint: { width, height }`, `production_targets`, optional `icon`;
- products: `id`, `display_name`.

Validation is all-or-nothing. `schema_version` must be `1`, `data_version` must be valid SemVer, typed IDs must follow their grammar and be unique within each kind, and all catalog, region, base, buildable, and product display names must be nonblank. A buildable symbol must contain one to four characters after trimming. Base and footprint dimensions must be in `1..=65535`. `default_base_id`, each base's `region_id`, and every `production_targets` entry must resolve. One buildable cannot repeat a production target. A buildable's `icon`, when present, is a relative path string resolved against the dedicated `assets/icons/` directory at presentation time — the catalog loader stores it verbatim without touching the filesystem; see [`README.md`](../README.md#custom-buildable-icons-and-machine-data) for the full customization workflow and safety contract.

Module paths must be nonempty, unique, and relative to the package root. Rooted paths, Windows prefixes or alternate-stream separators, `.` and `..` components, empty components, NUL bytes, and modules that resolve outside the canonical package root through a symlink are rejected. The loader returns a `Catalog` only after decoding and validating the complete candidate; failures return a typed `CatalogLoadError`.

`port_types.json` and `rules.json` remain possible future extensions. They are not accepted runtime inputs in schema v1.

`data_version` follows SemVer:

- `MAJOR`: incompatible IDs, semantics, or contract;
- `MINOR`: new compatible data or capabilities;
- `PATCH`: compatible data correction.

### Startup source selection

`catalog/public/` is tracked and embedded in the executable at build time. It is the minimal compatibility fallback. The ignored `data/catalog/` directory can contain a complete private package that is loaded once at startup.

- a complete valid private package becomes the active `Catalog`;
- a missing private `manifest.json` silently selects the embedded public catalog;
- any other private read, schema, path, or integrity failure selects the complete public catalog and leaves a persistent sanitized warning.

Public and private modules are never mixed, and a partial candidate never replaces the active snapshot. User-facing diagnostics do not echo raw JSON, full private paths, private identifiers, or private values. The tracked public package contains four bases and three buildables but intentionally keeps `products` and every `production_targets` list empty. It proves schema and runtime compatibility without asserting product data from the game.

There is no hot reload. Close the app before editing a package and restart it afterward. Semantic validation is all-or-nothing, but the manifest and four modules are separate filesystem reads rather than one atomic snapshot. Changes to `catalog/public/` require a rebuild so the executable embeds them again.

### Updating a package

Edit package data only while the app is closed:

1. add a region to the regions module before a base refers to its `RegionId`;
2. add a base with a unique `BaseId`, an existing `RegionId`, and valid dimensions;
3. add a product with a unique `ProductId` before declaring it in a buildable's `production_targets`;
4. add a buildable with a unique `BuildableId`, a `CategoryId`, a one-to-four-character symbol, valid footprint dimensions, and only existing product IDs;
5. keep existing IDs stable, update manifest `data_version` for the change, save the complete five-file package, and then restart the app so the startup loader validates it.

The ignored private package remains local. If the tracked public package changes, rebuild the executable instead of only restarting it. These checks prove structure and referential integrity, not whether the data is accurate in the game.

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
  icon: Option<relative path>
```

The category differentiates future behavior but does not create a second placement, rotation, bounds, or collision system. `icon` is purely presentational: an optional path resolved against `assets/icons/`, never validated against the filesystem by the domain or the catalog loader, and never affecting placement, rotation, bounds, collision, or production behavior. Physical ports and capabilities beyond the validated `production_targets` list remain planned extensions.

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

When ports are introduced, entity rotation will transform `anchor` and `side` through one domain geometry rule. The planned first port increment will not validate connections, recipes, rates, or flow.

## Implemented positioned entity

```text
BlockInstance
  id: EntityId
  buildable_id: BuildableId
  origin
  rotation
  production_target: Option<ProductId>
```

`production_target` is the user's choice on the instance. `set_production_target` accepts `None` or a `ProductId` that exists in the active `Catalog` and appears in that `BuildableDefinition.production_targets` list. A rejection preserves the instance. Movement and rotation preserve the configured target, and `FactoryLayout::place` revalidates it against the current or destination catalog and buildable before mutating the layout.

The target is configuration plus referential validation. It does not calculate or verify recipes, rates, inputs, outputs, ports, connectivity, throughput, regional rules, or the accuracy of game data.

## Implemented factory document

```text
FactoryDocument
  schema_version
  catalog_id
  catalog_data_version
  metadata
  base_id
  next_entity_id
  entities[]
```

Both document formats record `catalog_id` and `catalog_data_version` for provenance. An identity or version mismatch alone is non-blocking: otherwise-valid documents load with a `CatalogCompatibility` result. Loading still requires full validation, including validation against the active catalog, and can fail. Opening does not rewrite the on-disk file; reconstructed blueprints use active-catalog provenance.

The factory session retains compatibility state, but later actions such as selecting an instance can replace the visible open-result notice. A successful factory save records active-catalog provenance and resets session compatibility to `Exact`. The blueprint library instead displays mismatch indicators per cached library row.

No viewport, camera, or other editor-only metadata is persisted — only the layout, its entity allocator, and document metadata.

## Implemented blueprint document

```text
BlueprintDocument
  schema_version
  catalog_id
  catalog_data_version
  blueprint_id
  metadata
    name
    description (required, nullable)
    created_at
    updated_at
  nodes[]
```

- `nodes[]` uses origins normalized relative to the selection, with fresh canonical local entity IDs starting at 1;
- capturing a selection creates an independent copy: no external component is pulled in automatically, and the source factory's own entity IDs are not reused;
- an `interfaces[]` field for named physical ports open at the selection boundary, and inserting a blueprint back into a factory as a batch with new IDs, remain planned for Phase 5 — schema v1 as implemented today has no `interfaces[]` field and is not yet insertable.

## Implemented persistence

Factories and blueprints are separate, readable local JSON files. Each format has its own `schema_version`, currently `1` for both, with no migration between versions yet since only one version of each exists.

Save inputs come from domain APIs and `DocumentMetadata` construction, not a shared complete-domain-validation pass at save time. Factory encoding checks `next_entity_id`, formats metadata timestamps, then rejects zero entity IDs while encoding entities. Blueprint encoding formats metadata timestamps and serializes existing nodes without a complete domain or canonical-ID revalidation pass.

Both save paths follow this sequence:

1. encode the complete document to bytes in memory;
2. create a temporary file in the target's directory;
3. write all encoded bytes to the temporary file;
4. flush and synchronize the temporary file;
5. atomically replace the target with the temporary file;
6. report success only after replacement.

A failure at any step leaves the previously saved file (if any) completely untouched. Loading validates a complete document all-or-nothing; an unsupported schema version, invalid JSON, or any other decode failure leaves the currently open document unchanged. A version-migration step ("read and identify the version, migrate in memory when supported") is designed into this sequence for when a second schema version exists, but is not exercised today.

## Outside model v1

- conveyor connectivity graph;
- adjacent-port validation;
- recipes and production balance;
- throughput and bottlenecks;
- active regional rules;
- live link between a blueprint and an inserted instance;
- solver and auto-layout.
