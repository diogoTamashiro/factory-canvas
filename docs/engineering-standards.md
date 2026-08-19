# Padrões de engenharia

## Regra principal

Factory Canvas deve ser compreensível e sustentável sem IA. Decisões vivem no repositório, e não no histórico de conversas.

## KISS

- escolher a solução com menos conceitos;
- preferir funções e tipos explícitos;
- não introduzir ECS, event bus, plugins ou DI framework;
- `main.rs` contém somente bootstrap;
- medir antes de otimizar.

## YAGNI

- implementar somente comportamento aprovado;
- sem campos, traits ou configurações para hipóteses futuras;
- não manter iced e egui em paralelo;
- spikes são descartáveis;
- backlog não justifica complexidade atual.

## DRY com moderação

- regra de rotação, footprint e colisão tem uma fonte de verdade;
- não abstrair após a primeira repetição;
- duplicação pequena e clara é preferível a abstração obscura.

## SOLID pragmático

- módulos têm responsabilidade coesa;
- domínio não depende de UI ou I/O;
- traits existem quando há contrato e mais de uma necessidade real;
- nenhuma camada existe somente para “seguir SOLID”.

## Código

- identificadores internos em inglês;
- UI e documentação de produto em PT-BR;
- comentários explicam motivo e invariantes;
- enums no lugar de booleanos ambíguos;
- sem código comentado, debug prints ou TODO sem tarefa concreta;
- `unsafe` exige ADR, teste, benchmark e justificativa;
- erros de input/I/O não usam `unwrap()`;
- warnings novos bloqueiam commit.

## ACID

ACID vale para persistência:

- **Atomicidade:** arquivo temporário + rename; SQLite usa transação;
- **Consistência:** validar antes de salvar e depois de carregar;
- **Isolamento:** nenhum consumidor observa save parcial;
- **Durabilidade:** sucesso somente depois de concluir a gravação.

Arquivos têm `schema_version`; migrações preservam o original até validar o resultado. Queries SQLite são parametrizadas.

## Dependências

- preferir `std`;
- cada crate precisa de benefício atual documentado;
- revisar licença, manutenção, features e impacto no build;
- `Cargo.lock` é versionado;
- remover crates sem uso;
- nenhuma rede em runtime no MVP.

## TDD

Domínio, persistência e bug fixes seguem RED → GREEN → REFACTOR:

1. teste de comportamento;
2. executar e observar a falha esperada;
3. implementação mínima;
4. teste específico e suíte completa;
5. refatorar mantendo verde.

Testar geometria, rotação, limites, colisão, IDs e roundtrip. UI mantém lógica testável fora do painter e usa checklist manual para interação visual.

## Git

- uma tarefa lógica por commit;
- Conventional Commits;
- todo commit de código compila e passa os testes relevantes;
- não misturar reformatação ampla com mudança funcional;
- arquitetura recebe ADR;
- commits são narrados ao Diogo e devem ser reversíveis isoladamente.

## Revisão antes do commit

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

Além dos comandos:

- diff contém somente a tarefa;
- sem segredo, SQL interpolado ou path traversal;
- erros e edge cases relevantes tratados;
- documentação atualizada;
- código explicável sem consultar chat de IA.

## Definition of Done

Uma tarefa só está pronta quando tem critério de aceite, testes aplicáveis, código legível, verificação limpa, documentação atualizada, diff revisado e commit atômico.
