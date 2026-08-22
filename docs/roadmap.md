# Roadmap de implementação — Factory Canvas

> Documento operacional para continuar o projeto manualmente ou em outra conversa. Atualize-o ao encerrar cada recorte funcional; não use este arquivo como log de commits ou PRs.

## Objetivo do produto

Factory Canvas é um editor Windows nativo e offline para organizar manualmente uma fábrica 2D de *Arknights: Endfield*. O primeiro MVP resolve ocupação espacial em uma base escolhida; ele não otimiza produção, rota esteiras nem depende de rede.

## Princípios não negociáveis

- O domínio em `src/domain/` não depende de egui, filesystem, SQLite, rede ou Python.
- A UI chama APIs públicas de `FactoryLayout`; nunca duplica cálculo de footprint, bounds, colisão ou rotação.
- Grid: origem `(0, 0)` no canto superior esquerdo; X cresce à direita; Y cresce para baixo.
- Footprints e retângulos espaciais são semiabertos: contato de borda é permitido, sobreposição é proibida.
- `EntityId` é identidade estável, não índice de coleção. IDs novos são monotônicos e não são reutilizados após remoção.
- Erros de placement, movimento e rotação preservam o layout.
- Interface e documentação ficam em PT-BR; identificadores internos permanecem em inglês.
- KISS/YAGNI: não introduzir ECS, event bus, plugins, DI, drag-and-drop ou abstrações especulativas fora do recorte atual.
- Não alterar `src/main.rs` (binário `factory-canvas-legacy`) ao evoluir o editor egui sem uma necessidade explicitamente aprovada.

## Dados confirmados

### Bases

| Template | Bounds |
|---|---:|
| PAC Principal | 80×80 |
| Sub-PAC Padrão | 30×30 |
| Sub-PAC Expansão I | 40×40 |
| Sub-PAC Expansão II | 50×50 |

### Catálogo inicial

| Bloco | Categoria | Footprint |
|---|---|---:|
| Poste de Xiranita | Energia | 2×2 |
| Unidade de Refinaria | Produção I | 3×3 |
| Unidade de Trituração | Produção I | 3×3 |

Não inferir níveis anteriores da PAC Principal, portas, alcance de energia, receitas, throughput ou dados não confirmados.

## Estado já integrado ao editor egui

- Escolha das quatro bases confirmadas; troca destrutiva exige confirmação se houver instâncias.
- Paleta com os três blocos confirmados.
- Placement por clique usando origem superior esquerda, IDs monotônicos e validação exclusiva do domínio.
- Grid com aspect ratio preservado e hit testing com bordas direita/inferior exclusivas, incluindo proteção contra arredondamento `f32` no último tile.
- Lista semântica no sidebar com ID, nome, origem, footprint rotacionado e rotação.
- Seleção de instância pelo canvas ou pela lista textual; destaque visual da seleção.
- Remoção individual confirmada por botão, `Delete` ou `Backspace`; cancelar, Escape ou backdrop preservam estado.
- Domínio já possui `place`, `instance_at`, `remove_instance`, `move_instance` e `rotate_instance`, todos cobertos por testes.
- `factory-canvas-legacy` continua compilando como binário independente.

## Movimento e rotação visual — entregue nesta branch

### Interação de UX

Com uma instância selecionada:

- a sidebar oferece botões textuais para mover um tile: cima, esquerda, direita e baixo;
- `ArrowUp`, `ArrowLeft`, `ArrowRight` e `ArrowDown` executam os mesmos movimentos;
- **Girar 90°** e `R` giram no sentido horário;
- nenhuma edição ocorre sem seleção ou enquanto existe modal de remoção/troca de base pendente;
- os controles não antecipam bounds ou colisão: a tentativa chega ao domínio e recebe feedback PT-BR;
- não há arraste, clique para mover, pan/zoom, undo/redo ou persistência neste recorte.

### Contratos verificados

- movimento válido altera somente a origem da instância selecionada e preserva ID, seleção e `next_entity_id`;
- tentativa fora da base preserva layout, seleção e alocador e retorna `InstanceEditError::OutOfBounds` traduzido;
- rotação horária preserva ID e origem e atualiza a rotação;
- botões, setas, `R` e remoção são encaminhados por um único dispatcher de intenções do estado do app;
- a lista semântica continua expondo ID, origem, footprint rotacionado e rotação após cada edição.

## Preview de footprint no placement — implementado nesta branch

Com uma ferramenta de placement ativa:

- o canvas deriva uma candidata do template ativo e do tile sob o cursor;
- a candidata é desenhada sobre o canvas com preenchimento semitransparente e borda da cor do bloco;
- a prévia é somente visual: não consulta, replica ou antecipa bounds, colisão ou aceitação;
- o clique final permanece no fluxo existente e `FactoryLayout::place` continua a única autoridade espacial;
- selecionar uma instância limpa a ferramenta de placement e oculta a prévia; abrir modal de remoção/troca de base a suprime sem descartar a seleção de bloco guardada.

### Contratos verificados

- candidata visual existe somente quando há template ativo e cursor dentro do grid;
- origem e footprint da prévia derivam exclusivamente do hit testing e do catálogo;
- o canvas usa `Option<BlockTemplate>` como fonte única para cursor de placement, prévia e intenção de clique;
- seleção de tile ocupado continua prioritária sobre placement;
- modais destrutivos suprimem a prévia e preservam a ferramenta após cancelamento.

## Próximo recorte ativo — pan e zoom do canvas

Ao retomar, comece por viewport e transformação de coordenadas, não por drag-and-drop:

1. Escreva REDs para transformação tela↔grid com pan e zoom, incluindo bordas exclusivas.
2. Introduza viewport explícita e zoom centrado no cursor.
3. Desenhe somente a região visível depois que a transformação estiver coberta.
4. Preserve hit testing, placement, seleção e preview aplicando a transformação inversa em um único ponto.
5. Atualize testes, documentação, gates e revisão antes de publicar; se a aceitação ainda depender de retorno visual, peça feedback ao usuário.

## Próximos recortes, em ordem

### 1. Undo/redo por comandos

Modelar comandos de placement, remoção, movimento, rotação e troca de base. Só então considerar remoção imediata sem confirmação; enquanto não houver histórico, remoção individual deve continuar confirmada.

### 2. Persistência JSON local

Definir `schema_version`, serialização legível e save atômico: validar → serializar → arquivo temporário no mesmo volume → sincronizar quando aplicável → rename atômico → informar sucesso. Carregamento inválido nunca pode sobrescrever o layout atual.

### 3. Ajustes de acessibilidade e acabamento

Revisar a linha selecionável do sidebar para, se a versão egui permitir sem clipping, usar um controle com semântica de botão/foco ainda mais explícita. Manter sempre o label completo com ID, nome, origem, footprint e rotação.

### Itens deliberadamente posteriores

Portas, esteiras, divisores, integradores, receitas, throughput, solver/CP-SAT, auto-layout, OCR, importação do jogo, login, cloud, IA, sprites pesados e renderização 3D.

## Fluxo de engenharia por recorte

1. Criar ou revisar um plano em `.hermes/plans/` (não versionado) com escopo, decisões e gates.
2. Criar branch limpa a partir de `origin/master` integrado.
3. Trabalhar em tracer bullets RED → GREEN: um comportamento lógico, teste falhando, implementação mínima, teste verde.
4. Para UI, cobrir transições, rejeições e representação semântica com testes lógicos e determinísticos; não usar automação visual como substituto desses contratos.
5. Se a aceitação depender de aparência, legibilidade, composição ou outra percepção visual, pausar e pedir feedback explícito ao usuário antes de publicar.
6. Executar gates completos antes de congelar o stage.
7. Stage explícito, `git diff --cached --check`, scan de segurança das linhas adicionadas e revisão independente do snapshot congelado.
8. Somente depois criar commit com `[verified]`, publicar a branch e abrir PR contra `master`.
9. Não habilitar merge automático; após merge, confirmar `origin/master` antes do próximo recorte.

## Gates obrigatórios

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --bins
git diff --check
hermes verify --skip-start --json --timeout 300
```

Para mudança de UI:

1. cobrir por testes lógicos/determinísticos o fluxo de sucesso, ao menos uma rejeição/segurança e as transições de estado modificadas;
2. conferir que labels semânticos expõem o mesmo estado relevante que o painter;
3. compilar o binário principal e os bins release como parte dos gates, sem tratar uma captura automatizada como prova de interação ou aparência;
4. se uma decisão depender de retorno visual, pedir feedback explícito ao usuário e registrar o resultado antes de publicar;
5. encerrar qualquer processo de teste que tenha sido iniciado e confirmar que não bloqueia o próximo build.

## Retomada manual rápida

Se este trabalho for retomado sem contexto de conversa:

```bash
git fetch origin
git status --short --branch
git log --oneline -5 origin/master
```

Depois leia, nesta ordem:

1. `docs/roadmap.md` (este arquivo);
2. `CONTEXT.md`;
3. `docs/architecture.md`;
4. `src/egui_app.rs` e `src/egui_app_tests.rs`;
5. `src/domain/layout.rs` e `tests/domain_layout_editing.rs`.

Não retome de uma branch antiga presumindo que uma PR foi mesclada. Confirme a base real, comece pelo primeiro comportamento ainda não coberto por teste e mantenha o recorte pequeno.
