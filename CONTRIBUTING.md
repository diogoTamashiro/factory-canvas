# Contributing to Factory Canvas

## Principle

Contributions must make the project easier to maintain by someone without access to AI or to its development history.

## Environment

- Windows 10/11;
- stable Rust;
- Git;
- Python 3.11 only for legacy solver components.

## Workflow

1. Read `docs/product-scope.md` and `docs/architecture.md`.
2. Confirm that the change belongs to the current scope.
3. Create a small branch: `feat/...`, `fix/...`, `refactor/...`, or `docs/...`.
4. Write the behavior test first when applicable.
5. Implement only what is needed to make it pass.
6. Refactor while keeping the suite green.
7. Review the diff.
8. Run the verification commands.
9. Create an atomic commit using Conventional Commits.

## Verification

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

Documentation-only changes may skip builds and tests if they do not alter commands, configuration, or behavior.

## Commits

Format:

```text
type(scope): short description
```

Examples:

```text
docs(scope): redefine product as Factory Canvas
feat(model): add grid geometry and rotation
fix(storage): preserve original file on failed save
```

Keep one task per commit. Do not include opportunistic changes or unrelated generated files.

## Code rules

- follow KISS and YAGNI;
- keep the domain independent of egui and I/O;
- add no crate without a concrete need;
- use no `unwrap()` for input or I/O;
- introduce no new warnings;
- write comments that explain reasons, not syntax;
- record a source and confidence level for game data;
- depend on neither AI nor the network at runtime.

Read the complete rules in `docs/engineering-standards.md`.

## Architectural changes

Create or update an ADR in `docs/adr/` with the context, decision, alternatives, and consequences.

## Definition of Done

- requirement and acceptance criteria are clear;
- TDD was used when applicable;
- tests and verification commands pass cleanly;
- errors are handled;
- documentation is current;
- the diff has been reviewed;
- the commit is atomic and reversible;
- a human-readable summary was delivered to the maintainer.
