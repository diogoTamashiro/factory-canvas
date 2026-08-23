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

O domínio também enumera instâncias de forma imutável e determinística por ID. A remoção devolve a instância retirada ou `None` para um ID ausente. Movimento e rotação recebem valores absolutos, revalidam limites e colisão e preservam o estado anterior em qualquer erro.

A interface egui já converte clique em coordenada do grid, usa o tile vazio como origem superior esquerda, cria IDs monotônicos e chama `FactoryLayout::place`. Com um bloco ativo, o canvas desenha seu footprint semitransparente no tile sob o cursor; a prévia é somente visual e não replica ou antecipa bounds, colisão ou validação. Sucesso desenha a instância e a adiciona a uma lista textual acessível; rejeições mostram feedback PT-BR sem consumir ID. Clicar em uma instância pintada ou em sua linha no sidebar seleciona-a e destaca seu footprint. Com seleção ativa, controles textuais ou setas movem uma tentativa de um tile; **Girar 90°** ou `R` chama `Rotation::clockwise()`. Ambos delegam à API de edição do domínio e preservam layout, seleção e alocador em erro. `Remover bloco`, `Delete` e `Backspace` solicitam confirmação antes de chamar `FactoryLayout::remove_instance`; cancelar, Escape ou o backdrop preservam layout, seleção e alocação de IDs.

As próximas fases CAD adicionam viewport, seleção múltipla, produto configurado por entidade, documentos JSON e blueprints locais. Produto selecionado não implica validação de receita, entrada, saída ou throughput nesta fase.

### R4 — Restrições

O domínio rejeita:

- ID de instância duplicado;
- edição de ID inexistente;
- bloco fora dos limites;
- sobreposição de footprints;
- dimensão zero;
- referência a definição inexistente.

### R5 — Navegação

O ciclo CAD deverá oferecer pan, zoom, foco em seleção e enquadramento do layout à janela.

O shell atual ajusta e centraliza toda a base por uma transformação testada. Hit testing já exclui as bordas direita e inferior e devolve coordenadas inteiras do grid. Pan, zoom e foco em seleção permanecem incrementos posteriores.

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
