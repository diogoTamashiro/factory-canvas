# ADR 0001 — Windows desktop, Rust, and egui

- **Status:** Accepted — naming decision superseded by ADR 0002
- **Date:** 2026-08-10
- **Decider:** Diogo

## Context

The softFactory prototype used Rust + iced and built the editor as a matrix of buttons. The product scope was narrowed to a 2D planner focused on arranging rectangular blocks within fixed areas.

The old model does not correctly represent footprints larger than one tile, rotation, or a large canvas. The UI is still at a stage where migration costs little.

Confirmed requirements:

- Windows desktop only;
- native, offline application;
- lighter than the game renderer;
- a friendly, distinct visual style;
- code that remains sustainable without AI;
- a canvas with pan, zoom, and draggable objects.

## Decision

- rename the product to **Graph Planner**;
- continue using Rust;
- replace iced with `eframe/egui`;
- draw the layout on one custom canvas;
- separate the domain, UI, and persistence;
- freeze Gallery, Planner, capture, and solver work during the first MVP;
- create no compatibility layer between iced and egui.

## Rationale

- egui provides suitable painting and input APIs for visual tools;
- it avoids one widget per tile;
- it preserves the investment in Rust;
- it supports a native, offline binary;
- migrating now costs less than migrating after the editor grows.

## Alternatives considered

### Keep iced Canvas

This would require fewer dependency changes, but it would keep more interaction boilerplate and continue with a UI already considered temporary.

### Python + PySide6/Qt

An excellent option for a 2D scene and desktop UI, but it would require changing languages and produce a larger runtime/distribution.

### Flutter

Good visual design and canvas support, but it would add Dart without a need for mobile or cross-platform support.

### Tauri + React

A low learning curve for the maintainer, but it uses a WebView and conflicts with the preference for a non-web UI.

## Consequences

### Positive

- the new domain can be correct and testable from the start;
- the canvas is simpler to implement;
- rendering is cheaper than a widget grid;
- the application and model use the same language.

### Negative

- the existing iced UI will not be reused;
- egui requires a custom theme and components for a friendly appearance;
- canvas accessibility requires a parallel semantic list;
- old components will remain temporarily frozen in the repository.

## Review

Reconsider only if a measurable spike shows that egui cannot meet performance, DPI, keyboard, or minimum accessibility requirements. Aesthetic preference alone does not justify maintaining two stacks.
