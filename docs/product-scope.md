# Escopo do produto — Factory Canvas

## Problema

Montar uma fábrica diretamente no render do jogo exige recursos, dificulta experimentar alternativas e não oferece uma visão esquemática simples do espaço. Factory Canvas fornece uma representação 2D leve para o jogador testar a ocupação da base antes ou durante a montagem no jogo.

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

- `Main` — PAC Principal, com 80×80 no nível atualmente confirmado;
- `Secondary` — sub-PAC, com 30×30, 40×40 ou 50×50 conforme o nível de expansão.

`BaseTemplate` representa a opção selecionada, e seu tipo e nível confirmado determinam o `GridSize` do layout. As duas bases ficam em Wuling, são quadradas, não possuem obstáculos internos conhecidos e podem evoluir. Níveis desconhecidos da PAC Principal não serão inferidos.

### R2 — Catálogo de blocos

Cada definição de bloco contém, no mínimo:

- identificador estável;
- nome exibido;
- largura e altura sem rotação;
- categoria visual.

O primeiro catálogo real contém Poste de Xiranita (2×2), Unidade de Refinaria (3×3) e Unidade de Trituração (3×3). Todos aceitam quatro rotações e ambas as bases. Limites regionais, energia e portas permanecem metadados não validados neste recorte. Consulte `reference/layout-data.md`.

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

## Dados ainda pendentes

1. Níveis e dimensões anteriores da PAC Principal.
2. Valores numéricos dos limites de construção por região.
3. Conversão confirmada entre metros e tiles.
4. Coordenadas e tipos exatos das portas, quando essa mecânica entrar no escopo.
