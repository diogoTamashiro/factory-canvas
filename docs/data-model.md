# Modelo de dados v1 — Factory Canvas

> Contrato de evolução aprovado para documentos CAD e pacote de dados do jogo. Este documento não afirma que o modelo já está implementado no domínio atual.

## Separação de camadas

```text
Pacote de dados do jogo
        │
        ▼
Definições estáticas ───► Documento da fábrica ───► Blueprints locais
        │                         │                         │
        └── capacidades            └── entidades             └── cópias relativas
```

- **Dados do jogo:** fornecidos e versionados pelo Diogo.
- **Documento da fábrica:** estado espacial de uma base inteira.
- **Blueprint:** módulo produtivo salvo localmente e inserido como cópia independente.

## Identificadores

Todos os IDs persistidos de dados usam strings ASCII em `snake_case`, estáveis e independentes de nomes exibidos:

- `BuildableId`, por exemplo `refinery_unit`;
- `ProductId`, por exemplo `processed_xiranite`;
- `PortTypeId`, por exemplo `item`;
- `BaseId` e `RegionId`;
- `BlueprintId`.

`EntityId` permanece um identificador monotônico local à fábrica. Blueprints usam `BlueprintEntityId` local e nunca reutilizam `EntityId` da fábrica de origem.

## Pacote modular de dados

```text
catalog/
  manifest.json
  bases.json
  buildables.json
  products.json
  port_types.json
  regions.json
  rules.json
```

`manifest.json` contém:

```json
{
  "schema_version": 1,
  "data_version": "0.1.0",
  "files": [
    "bases.json",
    "buildables.json",
    "products.json",
    "port_types.json",
    "regions.json",
    "rules.json"
  ]
}
```

`data_version` segue SemVer:

- `MAJOR`: IDs, semântica ou contrato incompatível;
- `MINOR`: dados ou capacidades novos compatíveis;
- `PATCH`: correção compatível de dados.

## Entidade construível

Máquinas, esteiras, postes e futuros componentes usam a mesma definição espacial:

```text
BuildableDefinition
  id: BuildableId
  display_name
  category: Machine | Conveyor | Power | ...
  footprint
  ports[]
  capabilities
```

A categoria diferencia comportamento futuro, mas não cria um segundo sistema de placement, rotação, bounds ou colisão.

## Portas físicas

```text
PortDefinition
  id: PortId local ao buildable
  anchor: { tile, side }
  flow: Input | Output | Bidirectional
  port_type: PortTypeId
```

- `tile` é uma coordenada relativa dentro do footprint sem rotação;
- `side` é a face física `North`, `East`, `South` ou `West`;
- `flow` descreve o sentido lógico;
- `port_type` descreve o tipo transportado.

A rotação da entidade transforma `anchor` e `side` por uma única regra de geometria do domínio. Nesta fase, portas não validam conexão, receita, taxa ou fluxo.

## Entidade posicionada

```text
PlacedEntity
  id: EntityId
  buildable_id: BuildableId
  origin
  rotation
  production_target: Option<ProductId>
  configuration futura
```

`production_target` é escolha do usuário na instância. A definição estática informa se a entidade pode produzir e quais produtos ela pode oferecer; o primeiro incremento não calcula entradas, saídas, receita ou throughput.

## Documento da fábrica

```text
FactoryDocument
  schema_version
  catalog_data_version
  metadata
  base
  entities[]
  viewport/editor metadata opcional
```

O documento registra `catalog_data_version` para proveniência. Diferença de versão inicialmente gera aviso, nunca sobrescreve ou bloqueia o documento automaticamente.

## Documento de blueprint

```text
BlueprintDocument
  schema_version
  catalog_data_version
  blueprint_id
  name
  description opcional
  nodes[]
  interfaces[]
  metadata
```

- `nodes[]` usa origens relativas normalizadas à seleção;
- inserir um blueprint cria entidades novas com IDs monotônicas novas;
- a seleção é literal: nenhum componente externo é puxado automaticamente;
- uma interface representa uma porta física da seleção aberta para fora;
- interfaces podem receber nome do usuário, mas não afirmam esteira conectada, fluxo ou compatibilidade confirmada.

## Persistência

Fábricas e blueprints são arquivos JSON separados, legíveis e locais. Cada formato tem `schema_version` próprio.

Migração e save seguem:

1. ler e identificar versão;
2. migrar em memória quando suportado;
3. validar o documento completo;
4. serializar para arquivo temporário no mesmo volume;
5. sincronizar quando aplicável;
6. renomear atomicamente;
7. informar sucesso somente após o rename.

Arquivos desconhecidos, incompatíveis ou inválidos não substituem o documento atualmente aberto.

## Fora do modelo v1

- grafo de conectividade de esteiras;
- validação de portas adjacentes;
- receitas e balanço de produção;
- throughput e gargalos;
- regras regionais ativas;
- vínculo vivo entre blueprint e instância inserida;
- solver e auto-layout.
