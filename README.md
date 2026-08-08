# softFactory

Auxiliar desktop (Windows) offline-first para planejar **layouts e blueprints de fábricas do CAI** de *Arknights: Endfield*.

> Fase 1 (MVP): captura de tela local + agregação em SQLite + solver de produção (OR-Tools) via bridge Rust↔Python. Sem IA, sem rede.

## Stack
- **GUI:** Rust + [iced](https://iced.rs/) (nativo Windows, tema Dark)
- **Solver:** Python 3.11 + Google OR-Tools (LP/GLOP)
- **Bridge:** `std::process::Command` trocando JSON (stdin/stdout)
- **Storage:** SQLite local (embedded via `rusqlite`)

## Estrutura
```
src/
  main.rs          # UI iced: sidebar (Galeria / Planejador / Config)
  db.rs            # SQLite: tabelas captures + blueprints
  screenshot.rs    # captura de tela -> data/shots/*.png + registro no db
  solver_bridge.rs # chama solver/solve.py com JSON
solver/
  solve.py         # OR-Tools (recipes mockadas do CAI)
  requirements.txt
data/              # gitignored: softfactory.db + shots/
```

## Setup (Windows)
1. Instale [Rust](https://rustup.rs) (stable) — inclui MSVC Build Tools.
2. Crie o venv do projeto e instale o solver:
   ```
   uv venv --python 3.11
   uv pip install -r solver/requirements.txt
   ```
   > Não use `python3` solto: no Windows ele abre a Microsoft Store. Sempre `.venv\Scripts\python.exe`.

## Rodar (dev)
```
cargo run
```

## Build (release -> .exe nativo)
```
cargo build --release
# executavel: target/release/softfactory.exe
```

## Uso
- **Galeria:** botão "Capturar tela" salva PNG em `data/shots/` e lista no SQLite.
- **Planejador:** informe objetivo (Steel/min) e orçamento de espaço (tiles); o solver OR-Tools devolve máquinas e throughput.
- **Config:** caminhos do solver/db/capturas.

## Status
- [x] F1.1 scaffold iced
- [x] F1.2 SQLite (captures/blueprints)
- [x] F1.3 captura de tela
- [x] F1.4 UI shell (sidebar + Galeria)
- [x] F1.5 solver OR-Tools (mock)
- [x] F1.6 bridge Rust↔Python
- [x] F1.7 Planejador funcional
- [x] F1.8 Config + README
- [x] F1.9 build release
