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

## Escopo do primeiro ciclo CAD

### R1 — Tipo de base

Ao criar um layout, o usuário escolhe:

- `Main` — PAC Principal, com 80×80 no nível atualmente confirmado;
- `Secondary` — sub-PAC, com 30×30, 40×40 ou 50×50 conforme o nível de expansão.

`BaseTemplate` representa a opção selecionada, e seu tipo e nível confirmado determinam o `GridSize` do layout. As duas bases ficam em Wuling, são quadradas, não possuem obstáculos internos conhecidos e podem evoluir. Níveis desconhecidos da PAC Principal não serão inferidos.

O shell `egui` atual enumera `BaseTemplate::ALL` e redesenha a grade conforme os bounds derivados da opção escolhida. A troca é imediata quando o layout está vazio; quando há instâncias, um modal cancela com segurança ou exige confirmação explícita para trocar e limpar.

### R2 — Catálogo de entidades construíveis

O editor atual ainda possui um catálogo compilado de blocos. A direção versionada é um catálogo unificado de entidades construíveis: máquinas, esteiras, postes e futuros componentes compartilham placement, footprint, rotação, bounds e colisão.

Cada definição de entidade conterá, no mínimo:

- identificador estável;
- nome exibido;
- largura e altura sem rotação;
- categoria visual;
- portas físicas relativas, direção de fluxo e tipo;
- capacidades estáticas, como produtos que uma máquina pode produzir.

O domínio expõe em `BlockTemplate::ALL` o catálogo inicial com Poste de Xiranita (2×2), Unidade de Refinaria (3×3) e Unidade de Trituração (3×3). Todos aceitam quatro rotações e ambas as bases. A futura migração para IDs de catálogo carregados de dados preservará o domínio espacial e não antecipará validação de portas, receitas ou fluxo.

A paleta egui atual deriva nomes e footprints dessas definições, preserva o template selecionado para placements repetidos e não mantém um catálogo paralelo na UI. O pacote público descreve schemas; dados detalhados do jogo podem permanecer locais e ignorados.

### R3 — Placement

O usuário pode colocar, selecionar, mover, girar e remover blocos. Toda operação respeita snap no grid.

`FactoryLayout::place` anexa uma instância somente após validar ID, footprint rotacionado, limites e colisão. Footprints usam retângulos semiabertos: sobreposição é rejeitada, mas contato de borda é permitido.

O domínio também enumera instâncias de forma imutável e determinística por ID. A remoção devolve a instância retirada ou `None` para um ID ausente. Movimento e rotação singulares recebem valores absolutos e revalidam limites/colisão; a rotação singular preserva sua origem. Para conjuntos, `move_instances_by` e `rotate_instances_clockwise_about` removem as posições antigas numa cópia do layout, validam todos os destinos finais e fazem commit apenas se o lote inteiro for aceito. O pivô orbital vem do centro dos footprints físicos, encaixado no grid em direção ao canto superior esquerdo.

A interface egui converte clique em coordenada do grid, usa o tile vazio como origem superior esquerda, cria IDs monotônicos e chama `FactoryLayout::place`. Com um bloco ativo, o canvas desenha seu footprint semitransparente; a prévia é somente visual. Clique normal substitui a seleção, `Shift` adiciona e `Ctrl` alterna, tanto no canvas quanto na lista textual. Sem ferramenta ativa, arrastar o botão esquerdo desde espaço vazio cria marquee e inclui somente instâncias cuja origem está no retângulo. Todas as selecionadas recebem destaque. Controles/setas movem o conjunto; **Girar 90°**/`R` gira uma instância na própria origem ou move e orienta duas ou mais ao redor do pivô comum. O pivô persiste até a composição da seleção mudar e acompanha movimentos válidos; falhas preservam lote e pivô. `Remover bloco(s)`, `Delete` e `Backspace` congelam as IDs em um pedido confirmado; cancelar, Escape ou backdrop preservam layout, seleção e alocação.

As próximas fases CAD adicionam pacote modular de dados, produto configurado por entidade, documentos JSON e blueprints locais. Produto selecionado não implica validação de receita, entrada, saída ou throughput nesta fase.

### R4 — Restrições

O domínio rejeita:

- ID de instância duplicado;
- edição de ID inexistente;
- bloco fora dos limites;
- sobreposição de footprints;
- dimensão zero;
- referência a definição inexistente.

### R5 — Navegação

O ciclo CAD oferece pan, zoom, enquadramento da base e foco do conjunto selecionado.

Uma viewport persistente aplica pan e zoom à pintura e ao hit testing por uma transformação única; bordas direita e inferior continuam exclusivas. Roda do mouse amplia no cursor, botão do meio move a visão e `Home` enquadra a base inteira. `F` e **Enquadrar seleção** calculam a união dos footprints físicos selecionados, aplicam padding visual e não alteram `FactoryLayout`.

### R6 — Histórico

Placement, movimento, rotação, remoção, edição de grupo e inserção de blueprint suportarão undo e redo em fase posterior.

### R7 — Persistência

Fábrica e blueprints poderão ser salvos e abertos localmente em formatos versionados e legíveis.

### R8 — Documentos e blueprints CAD

A fábrica inteira será um `FactoryDocument`. Um conjunto selecionado poderá ser salvo como `BlueprintDocument` em biblioteca local persistente. Inserir um blueprint cria uma cópia independente com novas IDs; ele não atualiza automaticamente a definição original.

Blueprints preservam entidades em coordenadas relativas e expõem interfaces nomeáveis para portas físicas abertas na fronteira da seleção. Elas não representam conectividade confirmada nem fluxo validado.

## Não objetivos do primeiro MVP

- validação de conectividade entre portas e esteiras;
- receitas, produção automática e balanço de entradas/saídas;
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

Ao término do ciclo CAD, um jogador deverá conseguir criar um layout Principal ou Secundário, navegar entre visão geral e subconjuntos, colocar entidades construíveis com footprints diferentes, girá-las, reorganizá-las sem colisão, configurar o produto de entidades capazes, salvar o documento e reutilizar blueprints locais sem depender de rede.

## Dados ainda pendentes

1. Dados detalhados de PAC, entidades, portas, produtos, regiões e regras, mantidos no pacote local versionado.
2. Validação confirmada de conectividade de esteiras e portas.
3. Receitas, taxas, throughput e demais mecânicas de produção.
