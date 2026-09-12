# Engineering standards

## Primary rule

Factory Canvas must remain understandable and maintainable without AI. Decisions live in the repository, not in conversation history.

## KISS

- choose the solution with the fewest concepts;
- prefer explicit functions and types;
- do not introduce an ECS, event bus, plugins, or a dependency-injection framework;
- keep only bootstrap code in `main.rs`;
- measure before optimizing.

## YAGNI

- implement only approved behavior;
- add no fields, traits, or configuration for future hypotheses;
- do not maintain iced and egui in parallel;
- treat spikes as disposable;
- do not let the backlog justify current complexity.

## DRY in moderation

- keep one source of truth for rotation, footprint, and collision rules;
- do not abstract after the first repetition;
- prefer small, clear duplication over an obscure abstraction.

## Pragmatic SOLID

- modules have cohesive responsibilities;
- the domain does not depend on UI or I/O;
- traits exist when there is a contract and more than one real need;
- no layer exists only to "follow SOLID."

## Code

- internal identifiers are in English;
- the default product UI and tracked project documentation are in English; the frozen legacy interface retains its existing language;
- comments explain rationale and invariants;
- use enums instead of ambiguous booleans;
- leave no commented-out code, debug prints, or TODO without a concrete task;
- `unsafe` requires an ADR, test, benchmark, and justification;
- input and I/O errors do not use `unwrap()`;
- new warnings block the commit.

## ACID

ACID applies to persistence:

- **Atomicity:** temporary file plus rename; SQLite uses a transaction;
- **Consistency:** validate before saving and after loading;
- **Isolation:** no consumer observes a partial save;
- **Durability:** report success only after the write completes.

Files have a `schema_version`; migrations preserve the original until the result has been validated. SQLite queries are parameterized.

## Dependencies

- prefer `std`;
- every crate needs a documented current benefit;
- review its license, maintenance, features, and build impact;
- track `Cargo.lock` in version control;
- remove unused crates;
- use no network access at runtime in the MVP.

## TDD

Domain, persistence, and bug fixes follow RED → GREEN → REFACTOR:

1. write a behavior test;
2. run it and observe the expected failure;
3. write the minimal implementation;
4. run the focused test, then every test file that exercises the changed module(s);
5. refactor while keeping those tests green.

Test geometry, rotation, bounds, collision, IDs, and round trips. Keep UI logic testable outside the painter and use a manual checklist for visual interaction.

## Testing scope

A gate or workflow step that says "run the tests" means the automated tests covering what a task/commit actually changes — never a blanket `cargo test` across the whole workspace, and never as a default "just to be safe" step. In practice:

- during RED → GREEN iteration, run the one test you are driving by name;
- before closing a task or committing, run every test file (`cargo test --test <file>`) or `cargo test --lib <module::path>` that imports or exercises the type(s)/function(s) the task changed;
- a full-suite `cargo test` run is a deliberate, occasional sanity check invoked when the user explicitly asks for it, not a required gate step, and not something to run "by default" at the end of a feature or phase.

This applies at every scale: a single bug fix, a full spec-kit user story, and a whole phase all scope their test gate to what they changed, not to the repository's entire test count.

## Git

- one logical task per commit;
- Conventional Commits;
- every code commit builds and passes the relevant tests;
- do not mix broad reformatting with functional changes;
- record architectural changes in an ADR;
- narrate commits to Diogo and keep each one independently reversible.

## Pre-commit review

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test <file>   # or `cargo test <name>` — only tests covering this commit; see §Testing scope
cargo build --release
```

In addition to the commands:

- the diff contains only the task;
- there are no secrets, interpolated SQL statements, or path traversal;
- relevant errors and edge cases are handled;
- documentation is current;
- the code can be explained without consulting an AI chat.

## Definition of Done

A task is done only when it has acceptance criteria, applicable tests, readable code, clean verification, current documentation, a reviewed diff, and an atomic commit.
