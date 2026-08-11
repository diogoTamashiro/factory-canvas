# Graph Planner — contexto atual

> Arquivo de continuidade. Em uma conversa nova, peça para ler este arquivo antes de trabalhar no projeto.

## Produto

Graph Planner é um aplicativo Windows, nativo e offline para planejar layouts 2D de fábricas de Arknights: Endfield.

O primeiro objetivo não é resolver ou otimizar a fábrica. É oferecer um canvas leve no qual o jogador organiza blocos de footprints determinados dentro de uma área predeterminada.

## Bases conhecidas

Existem dois tipos de layout:

- **Base Principal:** área maior;
- **Base Secundária:** área menor.

As dimensões reais ainda precisam ser fornecidas pelo Diogo. Não inventar valores.

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

O diretório e repositório ainda se chamam `softFactory`; a renomeação administrativa do repositório GitHub é separada do nome do produto.

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

## Próxima implementação após a Tarefa 0

Extrair a geometria do domínio: `GridPoint`, `GridSize`, `Rotation` e transformação de footprints, seguindo RED → GREEN → REFACTOR.
