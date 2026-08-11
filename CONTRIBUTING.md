# Contribuindo com o Graph Planner

## Princípio

Contribuições devem deixar o projeto mais fácil de manter por uma pessoa sem acesso a IA ou ao histórico de desenvolvimento.

## Ambiente

- Windows 10/11;
- Rust stable;
- Git;
- Python 3.11 somente para componentes legados do solver.

## Workflow

1. Leia `docs/product-scope.md` e `docs/architecture.md`.
2. Confirme que a mudança pertence ao escopo atual.
3. Crie uma branch pequena: `feat/...`, `fix/...`, `refactor/...` ou `docs/...`.
4. Escreva primeiro o teste de comportamento quando aplicável.
5. Implemente somente o necessário para fazê-lo passar.
6. Refatore mantendo a suíte verde.
7. Revise o diff.
8. Execute as verificações.
9. Faça um commit atômico usando Conventional Commits.

## Verificação

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

Mudanças somente em documentação podem dispensar build/testes se não alterarem comandos, configuração ou comportamento.

## Commits

Formato:

```text
type(scope): descrição curta
```

Exemplos:

```text
docs(scope): redefine produto como Graph Planner
feat(model): add grid geometry and rotation
fix(storage): preserve original file on failed save
```

Uma tarefa por commit. Não incluir alterações oportunistas ou arquivos gerados sem relação.

## Regras de código

- KISS e YAGNI;
- domínio independente de egui e I/O;
- nenhuma crate sem necessidade concreta;
- nenhum `unwrap()` em input ou I/O;
- nenhum warning novo;
- comentários explicam motivo, não sintaxe;
- dados do jogo têm fonte e nível de confiança;
- sem dependência de IA ou rede em runtime.

Leia as regras completas em `docs/engineering-standards.md`.

## Mudanças arquiteturais

Crie ou atualize um ADR em `docs/adr/` contendo contexto, decisão, alternativas e consequências.

## Definition of Done

- requisito e aceite claros;
- TDD quando aplicável;
- testes e verificações limpos;
- erros tratados;
- documentação atualizada;
- diff revisado;
- commit atômico e reversível;
- resumo humano entregue ao mantenedor.
