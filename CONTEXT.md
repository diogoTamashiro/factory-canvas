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
- `src/domain/layout.rs` contém `EntityId`, instâncias, placement atômico, enumeração imutável por ID e remoção que devolve a instância removida.

O domínio é independente da UI e foi desenvolvido com testes RED → GREEN.

## Próxima implementação

Adicionar operações validadas de mover e girar instâncias, reutilizando as mesmas regras de limites e colisão.
