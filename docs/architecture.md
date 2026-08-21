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
- o estado egui possui um `FactoryLayout`; paleta, placement, seleção, remoção, enumeração e desenho consultam somente APIs públicas do domínio.

## Estrutura corrente e alvo incremental

```text
src/
  egui_main.rs             # entry point padrão eframe
  egui_app.rs              # shell, estado, paleta, feedback e modal
  egui_app_tests.rs        # testes das transições do editor
  egui_canvas.rs           # fit, hit testing e painter do grid/instâncias
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

`move_instance` e `rotate_instance` recebem valores absolutos. Ambas constroem uma candidata, validam existência, limites e colisão ignorando somente a versão atual com o mesmo ID e substituem o mapa apenas após sucesso. Falhas retornam `InstanceEditError` sem alterar o layout.

A ocupação usa retângulos semiabertos `[left, right) × [top, bottom)`. Por isso, sobreposição é colisão, enquanto contato de borda é permitido sem inventar folga adicional. Os templates confirmados atuais são quadrados; o caminho integrado de rotação reutiliza a validação espacial, enquanto a troca de eixos 90°/270° permanece coberta pelos testes geométricos genéricos sem inventar um bloco de catálogo.

## Invariantes

- footprint sempre maior que zero;
- IDs do catálogo são estáveis e únicos entre as opções confirmadas;
- `bounds` é derivado do `BaseTemplate` selecionado;
- instância referencia um `BlockTemplate` existente;
- apenas instâncias aprovadas por `place` entram na coleção;
- enumeração pública é somente leitura e determinística por ID;
- remoção não altera nenhuma instância remanescente;
- movimento e rotação preservam ID, template e o campo não editado;
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
- linhas principais a cada dez tiles mantêm leitura nas bases maiores;
- hit testing converte screen para `GridPoint`, com bordas direita e inferior exclusivas;
- um clique em tile ocupado consulta `FactoryLayout::instance_at` e seleciona a instância antes de considerar placement; clique vazio posiciona quando há ferramenta ativa ou desseleciona quando não há;
- o tile clicado vazio é a origem superior esquerda de uma candidata com rotação zero;
- `FactoryLayout::place` decide ID duplicado, bounds e colisão antes de qualquer incremento do alocador da UI;
- instâncias aceitas aparecem no painter e em uma lista textual paralela para AccessKit, com ID, nome, origem, footprint e rotação; a instância selecionada recebe borda visual e pode ser selecionada também pela lista;
- remover passa exclusivamente por `FactoryLayout::remove_instance` após modal de confirmação; `Delete` e `Backspace` abrem o mesmo modal, e cancelar/backdrop/Escape preservam estado e IDs;
- controles textuais e setas movem a seleção um tile por vez, e **Girar 90°**/`R` chamam rotação horária; ambos delegam a `move_instance` e `rotate_instance`, mantendo seleção e alocador em rejeições;
- trocar a base vazia é imediato; com instâncias, um modal exige confirmação antes de criar outro layout vazio.

Próximos incrementos:

- adicionar preview de footprint sem duplicar validação espacial;
- pan e zoom entram em recorte próprio, com zoom em torno do cursor;
- somente a região visível será desenhada quando houver viewport móvel;
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
