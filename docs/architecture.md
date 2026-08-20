# Arquitetura do Factory Canvas

## Estado

Documento alvo da nova arquitetura. A implementação atual em iced é legada e será substituída gradualmente.

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

## Módulos planejados

```text
src/
  main.rs                  # bootstrap eframe
  lib.rs
  domain/
    base.rs                # templates de base e níveis confirmados
    geometry.rs            # ponto, dimensão e rotação
    catalog.rs             # definições de blocos
    layout.rs              # layout, tipo de base e instâncias
    occupancy.rs           # tiles ocupados e colisões
    validation.rs          # invariantes e erros do domínio
  persistence/
    plan_file.rs           # JSON versionado e save atômico
  ui/
    app.rs                 # estado da aplicação
    canvas.rs              # desenho, viewport e hit testing
    palette.rs
    inspector.rs
    status_bar.rs
  history.rs               # comandos de undo/redo
```

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

`instances` expõe somente referências imutáveis em ordem crescente de `EntityId`. `remove_instance` devolve a instância retirada ou `None` quando o ID não existe; remover não exige revalidação espacial porque apenas reduz a área ocupada e não altera as instâncias restantes.

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

- um único painter desenha grid e blocos;
- apenas região visível é desenhada;
- world/grid e screen têm transformações explícitas e testáveis;
- zoom ocorre em torno do cursor;
- lógica de placement não fica no código de pintura;
- repaint contínuo só quando houver interação/animação necessária.

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

- `src/db.rs`;
- `src/screenshot.rs`;
- `src/solver_bridge.rs`;
- `solver/`;
- editor `Cell` por tile em `src/blueprint.rs`.

Eles não serão conectados ao novo domínio durante o primeiro MVP.
