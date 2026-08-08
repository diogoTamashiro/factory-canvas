# softFactory — Context (carregável em chat novo)

> Arquivo de continuidade do projeto. Num chat novo, peça: "leia C:\Users\diogo\Documents\Projetos\softFactory\CONTEXT.md e continue".

## O que é o projeto
App desktop **Windows-only**, **offline (sem rede/login)** que auxilia o jogador a desenhar **layouts e blueprints de fábricas do CAI (Complexo Automatizado Industrial) de Arknights: Endfield**.

## Por que existe (problema real)
- Equipamento de personagem no Endfield tem **status fixo** (não tem RNG de substats como Genshin/HSR/E7) → otimizador de gear estilo Fribbels **não faz sentido** aqui.
- O sistema de **fábrica** é a parte complexa: depende de I/O, tempo (throughput), espaço (tiles) e objetivo. É um problema de otimização de cadeia de produção.
- LLMs genéricos (testado: Gemini) **"sofreram"** nisso — porque resolver fluxo de I/O + restrição de espaço é álgebra/LP, não geração de texto.
- O endfieldtools.dev já existe (web, v3.1.37) com Factory Planner, mas é **online + depende de login skport** (nuvem). O BURACO que este app cobre: **versão desktop nativa, 100% local/offline**, com solver real em vez de só copiar fábrica de outros players.

## Stack decidida (travada com o usuário)
- **GUI (casca):** Rust + `iced` (Elm-style, declarativo, nativo Windows, visual 100% custom). Objetivo do usuário: **aprender Rust**.
- **Solver (cérebro):** Python 3.11 + `ortools` (Google OR-Tools — resolve fluxo de produção com restrição de I/O, tempo e espaço). Alternativa se OR-Tools falhar: `scipy.optimize.linprog`.
- **Bridge Rust↔Python:** `std::process::Command` trocando **JSON** por stdin/stdout. Sem embed de VM (PyO3 fica para fase futura).
- **Storage:** SQLite local (crate `rusqlite`) — blueprints + metadados de captura de tela.
- **Captura de tela:** Windows, salva PNG + registra no SQLite (Fase 1: fullscreen/active-window simples).
- **IA (Ollama local ou API):** camada OPCIONAL futura, só pra explicar resultado do solver e sugerir otimização. **Fora da Fase 1.**

## Arquitetura
```
[Rust + iced GUI]  -> desenha grafo, botões, galeria, captura, chama solver
       |  std::process::Command (JSON in/out)
[Python + OR-Tools] -> recebe {máquinas, I/O, taxas, objetivo, espaço}, resolve, devolve {viável?, máquinas_needed, throughput}
       |  lê/escreve
[SQLite local] -> blueprints (graph_json) + capturas (path, ts, tag)
```
Tudo offline, dois processos locais.

## Fase 1 (MVP — esqueleto funcional, sem executar ainda)
1. Verificar ambiente: `rustc`/`cargo`/`python3`/`ortools` (read-only).
2. Instalar Rust (rustup) + venv Python com OR-Tools.
3. Scaffold iced: janela nativa com sidebar (Galeria / Planejador / Config) + painel placeholder. `cargo run` abre janela.
4. Capturador de tela → PNG em `data/shots/` + registro SQLite.
5. SQLite: tabelas `captures(id,path,ts,tag)` e `blueprints(id,name,graph_json,ts)`.
6. Solver Python mínimo com **recipes MOCKADAS** (ex: Furnace consome 2 Iron/min → 1 Steel/min, ocupa 1 tile) + bridge Rust↔Python. UI define objetivo (ex: "10 Steel/min") + orçamento de espaço → mostra resultado.
7. Loop Hermes ↔ VS Code ↔ Rust; `cargo build --release` gera .exe nativo.

## Estado atual do repositório
- `C:\Users\diogo\Documents\Projetos\softFactory\` existe.
- `IDEA.md`: "um auxiliar de layout e blueprints de fábricas do endfield".
- AINDA NÃO foi executado nenhum comando (sem cargo new, sem instalação).
- Plano completo salvo em: `C:\Users\diogo\AppData\Local\hermes\plans\2026-08-07_endfield-factory-planner-fase1.md`

## Perfil do usuário (pra adaptar explicações)
- Domina: TypeScript, React, AdonisJS (web).
- Já teve aula de: Python, C, Java.
- **NUNCA mexeu com Rust** (meta de aprender) nem com C#.
- Quer visual **nativo, diferente de web** (rejeitou Electron/webview).
- Rejeitou Flutter/Dart porque o solver (OR-Tools) é fraco em Dart; por isso a split Python(solver) + Rust(GUI).
- Usa: Windows 10, navega no Opera GX, Hermes Agent como copiloto, VS Code como editor.

## Open questions / pendências
- **Data layer (Fase 2, trabalho mais pesado):** reunir recipes REAIS do CAI (I/O exato, taxas, custo de espaço por máquina) de wiki/datamine do jogo. Fase 1 usa mock.
- Nome final do app (prov. "Endfield Factory Planner" / "softFactory").
- Formato exato do grafo de produção (nós = máquinas/recursos, arestas = fluxo).
- Como o usuário desenha o grafo na UI (manual por enquanto; OCR de tela fica pra depois).

## Próximo passo sugerido
Rodar **Fase 0** (verificação do ambiente, read-only) para saber o que já está instalado antes de scaffoldar.
