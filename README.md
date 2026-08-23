# Factory Canvas

Aplicativo desktop Windows, offline e nativo para auxiliar jogadores de **Arknights: Endfield** a planejar layouts de fábrica em um canvas 2D leve.

> **Estado atual:** o domínio já contém geometria, catálogo e edição validada para colocar, enumerar, remover, mover e girar blocos. O binário padrão usa `eframe/egui`, permite escolher as quatro bases e os três blocos confirmados, mostra uma prévia semitransparente durante placement, posiciona e seleciona instâncias por clique, move uma seleção por controles ou setas e gira 90° com controle ou `R`. A próxima evolução transforma o editor em uma ferramenta CAD com navegação, seleção de módulos, blueprints locais e dados versionados. A interface `iced` permanece somente como binário legado durante a migração.

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
| PAC Principal | Área maior para o layout principal do jogador | 80×80 no nível atualmente confirmado |
| sub-PAC | Área secundária evolutiva | 30×30, 40×40 ou 50×50 conforme a expansão |

As duas bases ficam em Wuling, são quadradas, não possuem obstáculos internos conhecidos e podem evoluir. Os níveis anteriores da PAC Principal ainda não foram medidos e não serão inferidos. Dados de referência detalhados permanecem locais e não são versionados no repositório público.

## Fora do primeiro MVP

- validação de conectividade física entre portas e esteiras;
- cálculo de receitas, throughput e gargalos;
- solver CP-SAT e otimização automática;
- roteamento automático;
- captura de tela/OCR;
- importação automática do jogo;
- rede, login, cloud ou IA;
- renderização 3D ou sprites pesados.

Galeria, Planner, solver Python e captura existentes ficam congelados. O código não será apagado nesta etapa, mas não fará parte da navegação principal do novo produto.

## Decisões técnicas

- **Produto:** Factory Canvas
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

O binário `factory-canvas-legacy` ainda contém:

- UI `iced`;
- galeria e captura de tela;
- Planejador com bridge Python/OR-Tools;
- modelo temporário `Cell` por tile.

Esses componentes permanecem somente para preservar o histórico enquanto o novo editor é construído em tarefas pequenas e reversíveis.

## Executar o estado atual

Pré-requisito do novo shell: Rust stable. Python 3.11 é necessário somente para o solver legado.

```powershell
cargo run
```

No editor atual:

1. escolha uma base;
2. selecione um bloco na paleta, confira a prévia semitransparente no tile sob o cursor e clique no tile que será a origem superior esquerda do footprint;
3. clique em uma instância pintada ou em sua linha textual no sidebar para selecioná-la;
4. use os controles de direção ou as setas para mover um tile; use **Girar 90°** ou `R` para girar no sentido horário;
5. para removê-la, use **Remover bloco**, `Delete` ou `Backspace` e confirme a ação;
6. consulte no sidebar a contagem, o resultado da validação e a lista textual das instâncias.

Limites e colisões são validados exclusivamente pelo domínio. A prévia semitransparente é visual: ela não indica aceitação e não antecipa bounds ou colisão; somente o clique encaminhado a `FactoryLayout::place` decide a colocação. Trocar de base com blocos exige confirmação explícita e limpa o layout somente após `Trocar e limpar`. Uma instância selecionada recebe destaque no canvas; controles de direção e setas movem uma tentativa de um tile, enquanto **Girar 90°** e `R` aplicam rotação horária. Falhas de bounds ou colisão preservam a instância selecionada e mostram feedback PT-BR. `Cancelar`, `Escape` ou o backdrop do modal de remoção preservam layout, seleção e alocação de IDs. Pan, zoom, histórico e persistência ainda não fazem parte da interface egui.

Para abrir temporariamente a interface iced congelada:

```powershell
cargo run --bin factory-canvas-legacy
```

## Verificação

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --bins
```

## Documentação

- [Escopo do produto](docs/product-scope.md)
- [Roadmap de implementação](docs/roadmap.md)
- [Arquitetura](docs/architecture.md)
- [Padrões de engenharia](docs/engineering-standards.md)
- [Como contribuir](CONTRIBUTING.md)
- [ADR 0001 — Rust + egui](docs/adr/0001-editor-ui.md)
- [ADR 0002 — Nome Factory Canvas](docs/adr/0002-product-name-factory-canvas.md)
- [ADR 0003 — Documentos CAD, blueprints e dados versionados](docs/adr/0003-cad-documents-and-blueprints.md)
- [Modelo de dados v1](docs/data-model.md)

## Princípio de manutenção

> O projeto deve ser compreensível, compilável, testável e modificável sem depender de IA ou do histórico de conversas que o originou.
