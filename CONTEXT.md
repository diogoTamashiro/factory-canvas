# Factory Canvas — contexto atual

> Arquivo de continuidade. Em uma conversa nova, peça para ler este arquivo antes de trabalhar no projeto.

## Produto

Factory Canvas é um aplicativo Windows, nativo e offline para planejar layouts 2D de fábricas de Arknights: Endfield.

O primeiro objetivo não é resolver ou otimizar a fábrica. É oferecer um canvas leve no qual o jogador organiza blocos de footprints determinados dentro de uma área predeterminada.

## Bases conhecidas

Existem dois tipos de layout em Wuling. Ambos são quadrados, sem obstáculos internos conhecidos e podem evoluir:

- **PAC Principal:** 80×80 no nível atualmente confirmado; níveis anteriores ainda não medidos;
- **sub-PAC:** 30×30 no nível Padrão, 40×40 na Expansão de Área I e 50×50 na Expansão de Área II.

O nível selecionado determina os limites do layout. Não inferir a progressão desconhecida da PAC Principal.

## Catálogo inicial confirmado

- **Poste de Xiranita:** Energia, footprint 2×2;
- **Unidade de Refinaria:** Produção I, footprint 3×3;
- **Unidade de Trituração:** Produção I, footprint 3×3.

Todos permitem rotações de 0°, 90°, 180° e 270° e podem ser usados nas duas bases. Limites regionais, alcance de energia e portas não serão validados no primeiro recorte. Fontes e lacunas: `reference/layout-data.md`.

## Primeiro MVP

1. Escolher Base Principal ou Base Secundária.
2. Mostrar os limites fixos do layout.
3. Colocar blocos de largura e altura determinadas.
4. Mover, girar e remover blocos.
5. Impedir colisão e saída dos limites.
6. Pan, zoom, undo/redo e persistência local.

Portas, esteiras, throughput, CP-SAT, captura e OCR ficam fora deste recorte inicial.

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
- `src/domain/layout.rs` contém `EntityId`, instâncias, consulta de ocupação por tile e edição atômica: colocar, enumerar, remover, mover e girar sem violar limites ou colisão.

O domínio é independente da UI e foi desenvolvido com testes RED → GREEN.

## Nova interface

- `src/egui_main.rs` inicia o binário padrão `factory-canvas` com `eframe/egui`;
- `src/egui_app.rs` mantém `FactoryLayout`, paleta, seleção, IDs monotônicos, feedback, controles/atalhos de movimento e rotação e confirmações destrutivas de troca de base e remoção individual;
- `src/egui_canvas.rs` concentra fit, hit testing, seleção por tile e desenho do grid e das instâncias;
- os três blocos confirmados podem ser selecionados na paleta e posicionados por clique com rotação inicial zero;
- clicar em instância pintada ou linha textual do sidebar seleciona-a; o canvas destaca o footprint, controles/setas movem um tile, **Girar 90°**/`R` giram no sentido horário e `Remover bloco`, `Delete` ou `Backspace` abrem confirmação antes da remoção;
- `FactoryLayout::place` continua sendo a única autoridade de bounds e colisão, e as edições usam exclusivamente `move_instance`, `rotate_instance` e `remove_instance`;
- a lista textual do sidebar acompanha semanticamente as instâncias pintadas com ID, nome, origem, footprint e rotação;
- `src/main.rs` continua congelado e é compilado separadamente como `factory-canvas-legacy` durante a migração.

## Roadmap e próxima implementação

Consulte `docs/roadmap.md` para a sequência manual, decisões de UX, invariantes e gates. O próximo recorte funcional recomendado é preview de footprint sem duplicar validação espacial; depois virão pan/zoom, undo/redo e persistência local.
