# Escopo do produto — Graph Planner

## Problema

Montar uma fábrica diretamente no render do jogo exige recursos, dificulta experimentar alternativas e não oferece uma visão esquemática simples do espaço. Graph Planner fornece uma representação 2D leve para o jogador testar a ocupação da base antes ou durante a montagem no jogo.

## Usuário-alvo

Jogador de Arknights: Endfield que quer organizar manualmente sua fábrica, entender o uso de espaço e experimentar disposições sem depender de ferramenta online ou render 3D.

## Proposta de valor

- mais leve que abrir/renderizar a base no jogo;
- visual claro e focado em espaço;
- totalmente offline;
- sem conta ou serviço externo;
- o jogador mantém controle sobre o layout.

## Escopo do primeiro MVP

### R1 — Tipo de base

Ao criar um layout, o usuário escolhe:

- `Main` — Base Principal, maior;
- `Secondary` — Base Secundária, menor.

Cada tipo determina um `GridSize` imutável para aquele layout. As dimensões reais permanecem pendentes; o código não deve assumir números temporários como dados oficiais.

### R2 — Catálogo de blocos

Cada definição de bloco contém, no mínimo:

- identificador estável;
- nome exibido;
- largura e altura sem rotação;
- categoria visual.

O catálogo inicial pode usar fixtures explicitamente genéricas para validar o editor. Dados reais só entram quando tiverem fonte e confirmação.

### R3 — Placement

O usuário pode colocar, selecionar, mover, girar e remover blocos. Toda operação respeita snap no grid.

### R4 — Restrições

O domínio rejeita:

- bloco fora dos limites;
- sobreposição de footprints;
- dimensão zero;
- referência a definição inexistente.

### R5 — Navegação

O canvas oferece pan, zoom e ajuste do layout à janela.

### R6 — Histórico

Placement, movimento, rotação e remoção suportam undo e redo.

### R7 — Persistência

O layout pode ser salvo e aberto localmente em formato versionado e legível.

## Não objetivos do primeiro MVP

- portas, esteiras, divisores ou integradores;
- receitas e produção;
- throughput e gargalos;
- solver ou auto-layout;
- roteamento automático;
- sincronização online;
- captura/OCR;
- renderização 3D.

## Requisitos não funcionais

- Windows desktop;
- offline;
- canvas sem widget por tile;
- CPU ociosa próxima de zero;
- interface escalável e com alto contraste;
- ações principais com atalhos de teclado;
- nenhuma dependência de IA;
- arquivos do usuário nunca sobrescritos após erro de validação.

## Critério de aceite do MVP

Um jogador consegue criar um layout Principal ou Secundário, colocar blocos de footprints diferentes, girá-los, reorganizá-los sem colisão, desfazer ações e reabrir o arquivo salvo.

## Dados pendentes do Diogo

1. Dimensões exatas da Base Principal.
2. Dimensões exatas da Base Secundária.
3. Primeiros blocos reais: nome e footprint sem rotação.
