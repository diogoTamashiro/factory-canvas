# softFactory

Auxiliar desktop (Windows) offline-first para **manutenção e inspeção de layouts/blueprints de fábricas do CAI** de *Arknights: Endfield*.

> Fase 1 (FEITO): captura de tela local + agregação em SQLite + solver de produção (OR-Tools) via bridge Rust↔Python.
> Fase 2 (FEITO): recipes reais do CAI transcritas de prints do jogo; Planner recebe alvos e devolve máquinas/viabilidade/throughput; Editor de grid 2D com Projetos CAI de referência. Sem IA, sem rede.

## Stack
- **GUI:** Rust + [iced](https://iced.rs/) (nativo Windows, tema Dark)
- **Solver:** Python 3.11 + Google OR-Tools (**CP-SAT**, modelo de inteiros exato)
- **Bridge:** `std::process::Command` trocando JSON (stdin/stdout)
- **Storage:** SQLite local (embedded via `rusqlite`)

## Estrutura
```
src/
  main.rs          # UI iced: sidebar (Galeria / Planejador / Editor / Config)
  db.rs            # SQLite: tabelas captures + blueprints
  screenshot.rs    # captura de tela -> data/shots/*.png + registro no db
  solver_bridge.rs # chama solver/solve.py com JSON
  blueprint.rs     # modelo Blueprint (grid 2D: máquinas + esteiras), catálogo de Projetos CAI
solver/
  solve.py         # OR-Tools CP-SAT (recipes reais do CAI, ver reference/cai-data.md)
  requirements.txt
reference/
  cai-data.md      # dados reais transcritos de prints do jogo (instalações + projetos)
data/              # gitignored: softfactory.db + shots/
```

## Setup (Windows)
1. Instale [Rust](https://rustup.rs) (stable) — inclui MSVC Build Tools.
2. Crie o venv do projeto e instale o solver:
   ```powershell
   uv venv --python 3.11
   uv pip install -r solver/requirements.txt
   ```
   > Não use `python3` solto: no Windows ele abre a Microsoft Store. Sempre `.venv\Scripts\python.exe`.

## Rodar (dev)
```powershell
cargo run
```

## Build (release -> .exe nativo)
```powershell
cargo build --release
# executavel: target/release/softfactory.exe
```

## Uso
- **Galeria:** botão "Capturar tela" salva PNG em `data/shots/` e lista no SQLite.
- **Planejador:** informe o objetivo no formato `Item:qtd/min, Outro:qtd` (ex: `Cilindro de Cuprium:10, Po de Originium Denso:5`) e o orçamento de espaço (tiles). O solver CP-SAT devolve as máquinas necessárias, o throughput e se é viável.
  - Cada máquina = 1 tile; ciclo base 2s (=30/min) ou 10s (=6/min).
  - Receitas reais transcritas de prints do jogo (ver `reference/cai-data.md`).
- **Config:** caminhos do solver/db/capturas.
- **Editor (grid 2D):** dois submodos:
  - *Editar:* selecione uma máquina na paleta e clique num tile para colocá-la (✖ Apagar remove). Há também 4 botões de **esteira** (↑↓→←) que colocam uma correia com direção de fluxo. Redimensione (11x11 / 14x9 / 24x9), limpe.
    - **Salvar/Carregar** persiste o blueprint no SQLite (`data/softfactory.db`, tabela `blueprints`); fallback em `data/blueprints/<nome>.json` se o DB não estiver disponível.
    - **Importar Texto:** cole linhas `x,y=MÁQUINA` ou `x,y>BELT_DIR` (DIR em N/S/E/W) para reconstruir um layout rapidamente.
    - **Validar vs Projeto** compara seu grid com o Projeto CAI selecionado (falta/sobra de máquinas, tamanho).
    - **Diff vs Projeto** lista tile-a-tile o que está diferente do projeto de referência (modo manutenção).
  - *Referência (Projeto CAI):* escolha um dos 12 Projetos CAI transcritos (ex: Xiranita Eficiente 11x11). O app desenha o grid NxN com as instalações e mostra tags/insumos/produção.
- **Config:** caminhos do solver/db/capturas.

### Testar o solver direto (sem a GUI)
```powershell
echo '{"objective":{"Cilindro de Cuprium":10},"space":40}' | .venv\Scripts\python.exe solver\solve.py
```

## Dados do CAI
Os dados de produção das instalações foram transcritos de prints do banco de dados do jogo
("Arquivos de Instalações") e das telas de "Prévia de Projeto". Ver `reference/cai-data.md`.
Valores aproximados (ciclo 2s/10s) — refináveis conforme o jogo atualiza.

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
- [x] F1.10 captura funcional (db init + screenshots 0.8 + retry)
- [x] F2.1 transcrição de prints -> reference/cai-data.md
- [x] F2.2 recipes reais (JSON) em solve.py
- [x] F2.3 solver CP-SAT (viabilidade + máquinas + throughput)
- [x] F2.4 GUI Planner com nomes reais do CAI
- [x] F2.5 testes do solver (via CLI)
- [x] F2.6 README F2 documentado (seção "Uso" + solver CLI + dados do CAI)
- [x] F2.B.1 modelo Blueprint + catálogo de Projetos CAI
- [x] F2.C build release (softfactory.exe) + README F2.B (aba Editor)
- [x] F2.B.2 editor de grid 2D (colocar/remover máquinas)
- [x] F2.B.3 visualizar Projeto CAI como referência
- [x] F2.B.4 validação + salvar/carregar blueprint
- [x] F2.C build release + README F2.B
- [x] F3.1 esteiras como entidade no grid (direção N/S/E/W)
- [x] F3.3 diff de manutenção contra Projeto CAI
- [x] F3.4 import de blueprint por texto plano
- [x] F4 persistência de blueprints no SQLite
- [ ] F3.2 validação de fluxo CP-SAT (PENDENTE de dados de mecânica de esteira)
