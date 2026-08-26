# Factory Canvas — contexto atual

> Arquivo de continuidade. Em uma conversa nova, peça para ler este arquivo antes de trabalhar no projeto.

## Produto

Factory Canvas é um aplicativo Windows, nativo e offline para planejar layouts 2D de fábricas de Arknights: Endfield.

O primeiro objetivo não é resolver ou otimizar a fábrica. É oferecer uma ferramenta CAD leve na qual o jogador organiza a fábrica inteira ou módulos produtivos em um canvas 2D, com dados e validações de jogo evoluindo separadamente.

## Bases conhecidas

Existem dois tipos de layout em Wuling. Ambos são quadrados, sem obstáculos internos conhecidos e podem evoluir:

- **PAC Principal:** 80×80 no nível atualmente confirmado; níveis anteriores ainda não medidos;
- **sub-PAC:** 30×30 no nível Padrão, 40×40 na Expansão de Área I e 50×50 na Expansão de Área II.

O nível selecionado determina os limites do layout. Não inferir a progressão desconhecida da PAC Principal.

## Catálogo inicial confirmado

- **Poste de Xiranita:** Energia, footprint 2×2;
- **Unidade de Refinaria:** Produção I, footprint 3×3;
- **Unidade de Trituração:** Produção I, footprint 3×3.

Todos permitem rotações de 0°, 90°, 180° e 270° e podem ser usados nas duas bases. Limites regionais, alcance de energia e portas não são validados no editor atual. As fontes detalhadas são privadas e permanecem fora do repositório público.

## Primeiro MVP

1. Escolher Base Principal ou Base Secundária.
2. Mostrar os limites fixos do layout.
3. Colocar blocos de largura e altura determinadas.
4. Mover, girar e remover blocos.
5. Impedir colisão e saída dos limites.
6. Navegar pela fábrica toda ou por subconjuntos com pan, zoom, foco e seleção múltipla.
7. Salvar fábrica e blueprints de módulos produtivos localmente.

Conectividade de esteiras, validação de receitas, throughput, CP-SAT, captura e OCR ficam fora do primeiro incremento CAD; entidades construíveis, portas físicas e configuração de produto são contratos planejados de dados.

## Stack confirmada

- Windows desktop somente;
- Rust;
- migração de `iced` para `eframe/egui`;
- canvas 2D customizado;
- `serde` + JSON versionado inicialmente;
- offline, sem IA em runtime.

## Estado do repositório

O diretório local e o repositório GitHub se chamam `factory-canvas`, alinhados ao nome do produto.

A implementação atual é legada e contém UI iced, galeria, captura, planner e bridge Python/OR-Tools. Ela será congelada, não apagada de uma vez.

## Regras de engenharia

- KISS e YAGNI;
- DRY com moderação e SOLID pragmático;
- domínio sem dependência de UI/I/O;
- ACID nas operações de persistência;
- TDD para domínio e persistência;
- dependências mínimas;
- documentação e ADRs versionados;
- commits pequenos, verificáveis e narrados;
- código sustentável sem IA.

Detalhes: `docs/engineering-standards.md`.

## Novo domínio

- `src/domain/geometry.rs` contém `GridPoint`, `GridSize`, `Rotation` e transformação de footprints;
- `src/domain/base.rs` contém os quatro templates selecionáveis confirmados: PAC Principal 80×80 e sub-PAC 30×30, 40×40 e 50×50.
- `src/domain/catalog.rs` contém os três blocos iniciais confirmados, com IDs estáveis, nomes, categorias e footprints.
- `src/domain/layout.rs` contém `EntityId`, instâncias, consulta de ocupação por tile e edição atômica: colocar, enumerar, remover, mover e girar sem violar limites ou colisão; movimento e rotação de conjuntos validam o layout final inteiro antes do commit;

O domínio é independente da UI e foi desenvolvido com testes RED → GREEN.

## Nova interface

- `src/egui_main.rs` inicia o binário padrão `factory-canvas` com `eframe/egui`;
- `src/selected_set.rs` mantém IDs selecionadas em ordem determinística e aplica `Replace`, `Add` e `Toggle` sem duplicatas;
- `src/egui_app.rs` mantém `FactoryLayout`, paleta, seleção, IDs monotônicos, feedback, ações atômicas do grupo e confirmações destrutivas de troca de base e remoção singular/em lote;
- `src/egui_canvas.rs` concentra fit, `CanvasState`, viewport de pan/zoom, hit testing, marquee por origem, foco de seleção, preview de placement e desenho do grid/instâncias;
- os três blocos confirmados podem ser selecionados na paleta e posicionados por clique com rotação inicial zero; enquanto um bloco está ativo, seu footprint aparece semitransparente no tile sob o cursor sem antecipar bounds ou colisão;
- clique normal substitui a seleção; `Shift` adiciona; `Ctrl` alterna; arraste do botão esquerdo iniciado em espaço vazio cria marquee e considera somente a origem das instâncias;
- todas as IDs selecionadas recebem destaque; controles/setas movem o conjunto um tile, **Girar 90°**/`R` gira o conjunto e `Remover bloco(s)`, `Delete` ou `Backspace` abre uma confirmação única para as IDs congeladas;
- `FactoryLayout::place` continua sendo a autoridade de placement; `move_instances_by` e `rotate_instances_clockwise` são as autoridades atômicas para edições de grupo;
- a lista textual do sidebar acompanha semanticamente as instâncias pintadas com ID, nome, origem, footprint e rotação e suporta os mesmos modificadores de seleção;
- roda do mouse amplia/reduz no cursor, botão do meio move a viewport, `Home` enquadra a base inteira e `F`/botão enquadra os bounds físicos da seleção; nenhuma navegação altera o layout;
- `src/main.rs` continua congelado e é compilado separadamente como `factory-canvas-legacy` durante a migração.

## Roadmap e próxima implementação

Consulte `docs/roadmap.md` para a sequência manual, decisões de UX, invariantes e gates. A direção versionada é CAD com documentos de fábrica, blueprints independentes e pacote modular de dados. Viewport e seleção múltipla já estão integradas; o próximo recorte é pacote de dados e produto por entidade.
