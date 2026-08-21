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
- KISS/YAGNI: não introduzir ECS, event bus, plugins, DI, preview, drag-and-drop ou abstrações especulativas fora do recorte atual.
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
- não há arraste, clique em destino vazio, preview, pan/zoom, undo/redo ou persistência neste recorte.

### Contratos verificados

- movimento válido altera somente a origem da instância selecionada e preserva ID, seleção e `next_entity_id`;
- tentativa fora da base preserva layout, seleção e alocador e retorna `InstanceEditError::OutOfBounds` traduzido;
- rotação horária preserva ID e origem e atualiza a rotação;
- botões, setas, `R` e remoção são encaminhados por um único dispatcher de intenções do estado do app;
- a lista semântica continua expondo ID, origem, footprint rotacionado e rotação após cada edição.

## Próximo recorte ativo — preview de footprint

Ao retomar, comece pelo preview de placement, não por pan/zoom ou drag-and-drop:

1. Escreva um RED no canvas para a candidata derivada do template ativo e do tile sob o cursor.
2. Desenhe apenas uma prévia semitransparente; não copie validação de bounds ou colisão para a UI.
3. Mantenha o clique final encaminhado a `FactoryLayout::place` como hoje.
4. Preserve seleção/movimento/rotação: preview só existe enquanto há ferramenta de placement ativa.
5. Atualize testes, documentação, gates, smoke e revisão antes de publicar.

## Próximos recortes, em ordem

### 1. Pan e zoom do canvas

Adicionar viewport explícita, zoom centrado no cursor e desenhar apenas a região visível. Rever hit testing para aplicar a transformação inversa. Não iniciar antes que os testes de coordenada cubram pan/zoom.

### 2. Undo/redo por comandos

Modelar comandos de placement, remoção, movimento, rotação e troca de base. Só então considerar remoção imediata sem confirmação; enquanto não houver histórico, remoção individual deve continuar confirmada.

### 3. Persistência JSON local

Definir `schema_version`, serialização legível e save atômico: validar → serializar → arquivo temporário no mesmo volume → sincronizar quando aplicável → rename atômico → informar sucesso. Carregamento inválido nunca pode sobrescrever o layout atual.

### 4. Ajustes de acessibilidade e acabamento

Revisar a linha selecionável do sidebar para, se a versão egui permitir sem clipping, usar um controle com semântica de botão/foco ainda mais explícita. Manter sempre o label completo com ID, nome, origem, footprint e rotação.

### Itens deliberadamente posteriores

Portas, esteiras, divisores, integradores, receitas, throughput, solver/CP-SAT, auto-layout, OCR, importação do jogo, login, cloud, IA, sprites pesados e renderização 3D.

## Fluxo de engenharia por recorte

1. Criar ou revisar um plano em `.hermes/plans/` (não versionado) com escopo, decisões e gates.
2. Criar branch limpa a partir de `origin/master` integrado.
3. Trabalhar em tracer bullets RED → GREEN: um comportamento, teste falhando, implementação mínima, teste verde.
4. Executar gates completos antes de congelar o stage.
5. Fazer smoke de janela real para qualquer mudança de UI nativa; validar também a representação semântica de objetos pintados.
6. Stage explícito, `git diff --cached --check`, scan de segurança das linhas adicionadas e revisão independente do snapshot congelado.
7. Somente depois criar commit com `[verified]`, publicar a branch e abrir PR contra `master`.
8. Não habilitar merge automático; após merge, confirmar `origin/master` antes do próximo recorte.

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

1. `cargo build --bin factory-canvas` antes de abrir `target/debug/factory-canvas.exe`;
2. testar o fluxo de sucesso e pelo menos uma rejeição/segurança;
3. conferir labels semânticos em paralelo ao painter;
4. iniciar ambos os binários release (`factory-canvas` e `factory-canvas-legacy`);
5. encerrar todos os processos de teste e confirmar que nenhum executável bloqueia o próximo build.

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
