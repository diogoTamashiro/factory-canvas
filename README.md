# Graph Planner

Aplicativo desktop Windows, offline e nativo para auxiliar jogadores de **Arknights: Endfield** a planejar layouts de fábrica em um canvas 2D leve.

> **Estado atual:** redefinição de produto concluída na Tarefa 0. A implementação existente em `iced` é legada e será substituída gradualmente por uma interface `egui`.

## Objetivo do primeiro MVP

O usuário escolhe um tipo de base e organiza blocos de dimensões conhecidas dentro de um espaço com limites fixos.

O MVP deve permitir:

- escolher **Base Principal** ou **Base Secundária**;
- visualizar claramente os limites da área disponível;
- pesquisar e selecionar um bloco;
- colocar um bloco usando seu footprint fixo (`largura × altura`);
- mover, girar e remover blocos;
- bloquear sobreposição e placement fora da área;
- usar pan e zoom;
- desfazer e refazer ações;
- salvar e reabrir o layout localmente.

## Tipos de base

| Tipo | Descrição | Dimensões |
|---|---|---|
| Base Principal | Área maior para o layout principal do jogador | Pendente de dado real |
| Base Secundária | Área menor para um layout secundário | Pendente de dado real |

As dimensões não serão inventadas. Elas entrarão no catálogo somente depois de confirmadas no jogo.

## Fora do primeiro MVP

- conexão de portas e desenho de esteiras;
- divisores e integradores;
- cálculo de throughput;
- solver CP-SAT e otimização automática;
- roteamento automático;
- captura de tela/OCR;
- importação automática do jogo;
- rede, login, cloud ou IA;
- renderização 3D ou sprites pesados.

Galeria, Planner, solver Python e captura existentes ficam congelados. O código não será apagado nesta etapa, mas não fará parte da navegação principal do novo produto.

## Decisões técnicas

- **Produto:** Graph Planner
- **Plataforma:** Windows desktop
- **Linguagem:** Rust
- **UI alvo:** `eframe/egui`
- **Renderização:** canvas 2D customizado; nunca um widget por tile
- **Persistência inicial:** JSON local versionado
- **Runtime:** totalmente offline

A escolha de Rust + egui prioriza baixo consumo, interação de canvas, binário nativo e código independente de serviços externos.

## Arquitetura alvo

```text
UI egui ───────┐
               ├──> domínio puro
persistência ──┘
```

O domínio não conhece egui, SQLite, filesystem ou Python. Veja [`docs/architecture.md`](docs/architecture.md).

## Código legado

A versão atual ainda contém:

- UI `iced`;
- galeria e captura de tela;
- Planejador com bridge Python/OR-Tools;
- modelo temporário `Cell` por tile.

Esses componentes permanecem somente para preservar o histórico enquanto o novo editor é construído em tarefas pequenas e reversíveis.

## Executar o estado atual

Pré-requisitos: Rust stable e Python 3.11 para o solver legado.

```powershell
cargo run
```

## Verificação

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

## Documentação

- [Escopo do produto](docs/product-scope.md)
- [Arquitetura](docs/architecture.md)
- [Padrões de engenharia](docs/engineering-standards.md)
- [Como contribuir](CONTRIBUTING.md)
- [ADR 0001 — Rust + egui](docs/adr/0001-editor-ui.md)
- [Dados conhecidos do CAI](reference/cai-data.md)

## Princípio de manutenção

> O projeto deve ser compreensível, compilável, testável e modificável sem depender de IA ou do histórico de conversas que o originou.
