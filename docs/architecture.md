# Arquitetura do Factory Canvas

## Estado

O binário padrão já usa `eframe/egui`; a implementação iced permanece congelada em um binário legado enquanto a migração avança por recortes pequenos.

## Objetivo

Manter regras de layout independentes de UI e persistência, permitindo testar toda a geometria sem abrir uma janela.

## Dependências permitidas

```text
┌──────────────┐      ┌────────────────┐
│ UI (egui)    │ ───> │                │
└──────────────┘      │ domínio puro   │
┌──────────────┐      │                │
│ persistência │ ───> │                │
└──────────────┘      └────────────────┘
```

- domínio não importa egui, filesystem, banco ou Python;
- UI chama operações públicas do domínio;
- persistência converte entre arquivo versionado e domínio;
- UI e persistência não dependem uma da outra;
- nenhuma dependência circular.

## Binários durante a migração

- `factory-canvas` é o binário padrão e inicia `src/egui_main.rs` com eframe 0.36, backend `glow` e AccessKit;
- `factory-canvas-legacy` compila `src/main.rs` sem adaptar iced ao novo shell;
- ambos podem ser verificados durante a transição, mas apenas o binário egui recebe funcionalidades do novo editor;
- o estado egui possui um `FactoryLayout`, `SelectedSet` e `CanvasState`; paleta, placement, seleção, remoção, enumeração e desenho consultam somente APIs públicas do domínio.

## Estrutura corrente e alvo incremental

```text
src/
  egui_main.rs             # entry point padrão eframe
  egui_app.rs              # shell, estado, paleta, feedback e modais
  egui_app_tests.rs        # testes das transições do editor
  egui_canvas.rs           # fit, viewport, marquee, foco e painter
  selected_set.rs          # conjunto ordenado e modos de seleção
  main.rs                  # entry point iced legado
  lib.rs
  domain/
    base.rs                # templates de base e níveis confirmados
    geometry.rs            # ponto, dimensão e rotação
    catalog.rs             # definições de blocos
    layout.rs              # layout, ocupação, validação e edição
  persistence/             # futuro JSON versionado e save atômico
  history.rs               # futuro comando de undo/redo
```

O canvas foi extraído quando ganhou uma responsabilidade real separável: transformar coordenadas e desenhar. `egui_app.rs` continua dono das transições de estado; `egui_canvas.rs` não modifica o layout nem replica validação espacial. Novos componentes só serão extraídos quando houver outra fronteira concreta.

## Direção versionada: CAD, documentos e dados

O modelo atual continua válido para o catálogo mínimo compilado. A evolução aprovada separa três camadas, sem introduzir validação de produção prematura:

```text
CatalogManifest + dados modulares ──> definições estáticas
                                       │
FactoryDocument ─────────────────────> entidades posicionadas
                                       │
BlueprintDocument ───────────────────> cópias relativas e interfaces expostas
```

- o pacote de dados do jogo tem `schema_version` e `data_version` SemVer, com entidades construíveis, PACs, produtos, tipos de porta, regiões e regras;
- máquinas, esteiras, postes e futuros componentes convergem para entidades construíveis com o mesmo mecanismo espacial;
- uma porta contém âncora física relativa, lado, direção de fluxo e tipo; rotação transforma somente a âncora e o lado físicos por regra única do domínio, enquanto fluxo e tipo permanecem atributos lógicos estáticos;
- uma entidade posicionada guarda opcionalmente o produto escolhido pelo usuário, sem calcular receitas, conectividade ou throughput;
- `FactoryDocument` e `BlueprintDocument` são JSON locais separados, legíveis e migráveis por `schema_version`;
- blueprint salva uma seleção literal em coordenadas relativas e, ao inserir, cria cópias com IDs novas;
- interfaces de blueprint representam portas físicas abertas na fronteira da seleção; elas não afirmam ligação de esteira ou fluxo confirmado.

O contrato completo está em [`docs/data-model.md`](data-model.md) e a decisão está registrada na [ADR 0003](adr/0003-cad-documents-and-blueprints.md).

## Modelo inicial

```rust
pub enum BaseKind {
    Main,
    Secondary,
}

pub enum SecondaryLevel {
    Standard,
    AreaExpansionI,
    AreaExpansionII,
}

pub enum BaseTemplate {
    MainCurrent,
    Secondary(SecondaryLevel),
}

pub enum GridSizeError {
    ZeroWidth,
    ZeroHeight,
}

pub struct GridSize {
    width: u16,
    height: u16,
}

impl GridSize {
    pub const fn new(width: u16, height: u16) -> Result<Self, GridSizeError> {
        if width == 0 {
            return Err(GridSizeError::ZeroWidth);
        }
        if height == 0 {
            return Err(GridSizeError::ZeroHeight);
        }
        Ok(Self { width, height })
    }

    pub const fn width(self) -> u16 {
        self.width
    }

    pub const fn height(self) -> u16 {
        self.height
    }
}

pub enum BlockCategory {
    Energy,
    ProductionI,
}

pub enum BlockTemplate {
    XiranitePowerPole,
    RefineryUnit,
    CrushingUnit,
}

pub struct BlockDefinition {
    id: &'static str,
    display_name: &'static str,
    category: BlockCategory,
    footprint: GridSize,
}

pub struct EntityId(u64);

pub struct BlockInstance {
    id: EntityId,
    template: BlockTemplate,
    origin: GridPoint,
    rotation: Rotation,
}

pub enum PlacementError {
    DuplicateEntityId { id: EntityId },
    OutOfBounds { id: EntityId },
    Collision { id: EntityId, conflicting_id: EntityId },
}

pub enum InstanceEditError {
    EntityNotFound { id: EntityId },
    OutOfBounds { id: EntityId },
    Collision { id: EntityId, conflicting_id: EntityId },
}

pub struct FactoryLayout {
    base_template: BaseTemplate,
    instances: BTreeMap<EntityId, BlockInstance>,
}
```

Os tamanhos das bases vêm de uma fonte de dados confirmada. `BaseTemplate` identifica a opção exata selecionada: PAC Principal no estado atual conhecido ou sub-PAC Padrão, Expansão I ou Expansão II. Assim, o nível escolhido determina os limites sem inventar a progressão ainda desconhecida da PAC Principal.

`BlockTemplate::ALL` lista exatamente os três blocos iniciais confirmados e resolve cada opção para uma `BlockDefinition` imutável. Energia, portas, limites regionais e receitas não fazem parte dessa definição inicial.

`FactoryLayout` deriva seus limites de `BaseTemplate`, em vez de manter uma cópia que poderia divergir. `place` valida ID duplicado, footprint rotacionado, limites e colisão antes de inserir; qualquer erro preserva o estado anterior.

`instances` expõe somente referências imutáveis em ordem crescente de `EntityId`. `instance_at` resolve a instância que ocupa um `GridPoint` usando o mesmo footprint rotacionado e os mesmos limites semiabertos da validação; a UI o usa para hit testing, sem repetir aritmética espacial. `remove_instance` devolve a instância retirada ou `None` quando o ID não existe; remover não exige revalidação espacial porque apenas reduz a área ocupada e não altera as instâncias restantes.

`move_instance` e `rotate_instance` recebem valores absolutos. Ambas constroem uma candidata, validam existência, limites e colisão ignorando somente a versão atual com o mesmo ID e substituem o mapa apenas após sucesso. A rotação singular preserva a origem. Falhas retornam `InstanceEditError` sem alterar o layout.

A ocupação usa retângulos semiabertos `[left, right) × [top, bottom)`. Por isso, sobreposição é colisão, enquanto contato de borda é permitido sem inventar folga adicional. Os templates confirmados atuais são quadrados; o caminho integrado de rotação reutiliza a validação espacial, enquanto a troca de eixos 90°/270° permanece coberta pelos testes geométricos genéricos sem inventar um bloco de catálogo.

Para duas ou mais instâncias, `selection_rotation_pivot` calcula a união dos footprints efetivos, toma seu centro e encaixa coordenadas meio-tile no grid em direção ao canto superior esquerdo. `rotate_instances_clockwise_about` gira cada retângulo 90° no sistema em que Y cresce para baixo, avança sua orientação e valida todos os destinos numa cópia sem as posições antigas do próprio lote. O `SelectedSet` guarda o pivô aceito enquanto as IDs não mudarem; um movimento válido o translada pelo mesmo delta, enquanto seleção alterada o invalida e edição rejeitada o preserva.

## Invariantes

- footprint sempre maior que zero;
- IDs do catálogo são estáveis e únicos entre as opções confirmadas;
- `bounds` é derivado do `BaseTemplate` selecionado;
- instância referencia um `BlockTemplate` existente;
- apenas instâncias aprovadas por `place` entram na coleção;
- enumeração pública é somente leitura e determinística por ID;
- remoção não altera nenhuma instância remanescente;
- movimento e rotação preservam ID e template; rotação singular também preserva origem, enquanto rotação orbital altera origem e orientação;
- edição com erro preserva integralmente o estado anterior;
- footprint rotacionado permanece dentro de `bounds`;
- duas instâncias não ocupam o mesmo tile;
- IDs são únicos e estáveis durante a vida do layout;
- ocupação é derivada das instâncias e não persistida como cópia.

## Canvas

Estado corrente:

- um único painter desenha fundo, grid, contorno da base e instâncias;
- `BaseTemplate::bounds()` determina linhas e dimensões exibidas;
- uma transformação testada ajusta o grid inteiro à área disponível, centralizado e com aspecto preservado;
- `CanvasState` reúne a viewport persistente, o estado transitório do marquee e o pedido de foco; toda pintura, preview e conversão tela→grid usam o mesmo retângulo transformado;
- linhas principais a cada dez tiles mantêm leitura nas bases maiores;
- hit testing converte screen para `GridPoint`, com bordas direita e inferior exclusivas;
- clique normal em tile ocupado produz `Replace`; `Shift` produz `Add`; `Ctrl` produz `Toggle`; clique vazio só desseleciona sem modificadores;
- marquee começa apenas com arraste primário em tile vazio e sem ferramenta de placement, normaliza qualquer direção e inclui somente entidades cuja origem pertence ao retângulo contínuo do grid;
- o tile clicado vazio é a origem superior esquerda de uma candidata com rotação zero;
- `FactoryLayout::place` decide ID duplicado, bounds e colisão antes de qualquer incremento do alocador da UI;
- instâncias aceitas aparecem no painter e em lista textual paralela para AccessKit, com ID, nome, origem, footprint e rotação; todas as IDs do `SelectedSet` recebem borda visual, e a lista aceita os mesmos modificadores de seleção;
- remoção singular ou em grupo passa exclusivamente por `FactoryLayout::remove_instance` após um modal que congela as IDs; `Delete` e `Backspace` abrem o mesmo pedido, e cancelar/backdrop/Escape preservam estado e IDs;
- controles textuais e setas movem o conjunto um tile, e **Girar 90°**/`R` gira uma instância na própria origem ou duas ou mais ao redor do pivô comum persistente; `move_instances_by` e `rotate_instances_clockwise_about` removem posições antigas em uma cópia, validam o layout final e só então fazem commit atômico;
- com ferramenta de placement ativa, o canvas deriva do tile sob o cursor uma candidata visual do template ativo e a desenha semitransparente; ela não consulta ou replica bounds, colisão ou validação do domínio, e o clique final continua em `FactoryLayout::place`;
- roda do mouse aplica zoom ancorado no cursor, botão do meio aplica pan, `Home` restaura a base inteira e `F`/botão enquadra a união dos footprints físicos selecionados com padding; navegação não altera `FactoryLayout`;
- trocar a base vazia é imediato; com instâncias, um modal exige confirmação antes de criar outro layout vazio.

Próximos incrementos:

- pacote modular de dados e produto configurado por entidade entram no próximo recorte;
- somente a região visível será desenhada quando houver otimização de viewport posterior;
- repaint contínuo ocorre apenas durante interação ou animação.

## Persistência

Primeiro formato: JSON legível com `schema_version`.

Save seguro:

1. validar estado;
2. serializar;
3. escrever arquivo temporário no mesmo volume;
4. sincronizar quando aplicável;
5. renomear atomicamente;
6. somente então informar sucesso.

SQLite pode voltar no futuro para biblioteca de layouts/recentes, sem substituir o arquivo portátil do usuário.

## Componentes congelados

- `src/main.rs`, compilado como `factory-canvas-legacy`;
- `src/db.rs`;
- `src/screenshot.rs`;
- `src/solver_bridge.rs`;
- `solver/`;
- editor `Cell` por tile em `src/blueprint.rs`.

Eles não serão conectados ao novo domínio durante o primeiro MVP.
