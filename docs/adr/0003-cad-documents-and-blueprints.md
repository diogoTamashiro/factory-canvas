# ADR 0003 — Documentos CAD, blueprints e dados versionados

- **Status:** Accepted
- **Data:** 2026-08-21
- **Decisor:** Diogo

## Contexto

Factory Canvas começou como um editor 2D de ocupação espacial para uma base. O produto passa a evoluir para uma ferramenta CAD offline de fábrica: o jogador deve poder projetar a fábrica inteira, focar subconjuntos, configurar o produto de cada máquina e salvar módulos produtivos reutilizáveis.

Os dados de jogo — PACs, entidades construíveis, produtos, portas, regiões e mecânicas — serão mantidos e versionados pelo Diogo. Eles não podem ficar acoplados ao canvas, ao documento do usuário ou a regras de produção ainda não confirmadas.

O repositório público também não armazena dados privados de referência. Logo, o formato público precisa descrever contratos e schemas sem depender de arquivos privados.

## Decisão

- separar o documento da fábrica (`FactoryDocument`) da definição reutilizável de módulo (`BlueprintDocument`);
- persistir ambos como JSON local legível, cada qual com `schema_version` e migrações explícitas;
- usar um pacote modular de dados de jogo com manifesto e `data_version` SemVer controlado pelo Diogo;
- unificar máquinas, esteiras, postes e futuros componentes como entidades construíveis posicionáveis;
- armazenar a escolha de produto em cada entidade posicionada, enquanto a definição estática declara apenas capacidades;
- salvar blueprints como cópias independentes em coordenadas relativas, sem vínculo vivo com a fábrica ou definição de origem;
- tratar toda porta física exposta na fronteira de uma seleção como interface nomeável do blueprint, sem afirmar conexão ou fluxo confirmado;
- manter validação de receita, conectividade de esteiras, throughput, regras regionais ativas e solver fora da primeira implementação desses documentos.

O contrato detalhado está em [`docs/data-model.md`](../data-model.md).

## Consequências

### Positivas

- o CAD pode evoluir independentemente da coleta de dados de jogo;
- blueprints são portáveis e repetíveis sem reutilizar IDs da fábrica;
- o formato fica auditável, migrável e adequado para trabalho offline;
- catálogo, documento do usuário e interface mantêm fronteiras explícitas;
- esteiras podem participar do mesmo sistema espacial sem criar um modelo paralelo.

### Negativas

- a migração futura de `BlockTemplate`/`BlockInstance` estáticos para IDs de dados exige uma fase própria e testes de compatibilidade;
- JSON versionado introduz responsabilidade de migração e save atômico;
- interfaces de blueprint inicialmente representam portas expostas, não conectividade real;
- a versão dos dados precisa ser mantida com disciplina para preservar proveniência dos documentos.

## Alternativas consideradas

### Um documento único para fábrica e módulos

Rejeitada porque um blueprint deixaria de ser uma unidade reutilizável independente e manteria IDs/estado da fábrica de origem.

### SQLite como formato primário de documentos

Rejeitada para a primeira versão porque JSON separado é mais legível, portátil, fácil de versionar e suficiente para a biblioteca local offline. SQLite pode voltar como índice de recentes ou busca, sem substituir os documentos portáveis.

### Conectar portas diretamente no primeiro modelo

Rejeitada porque o jogo usa esteiras colocadas pelo jogador. A primeira versão precisa representar entidades espaciais e portas físicas, sem inventar regras de topologia, fluxo ou compatibilidade ainda não confirmadas.

## Revisão

Revisar esta ADR quando a conectividade de esteiras ou a primeira migração de `schema_version` exigir uma mudança incompatível do contrato de documentos.