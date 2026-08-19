# ADR 0002 — Renomear o produto para Factory Canvas

- **Status:** Accepted
- **Data:** 2026-08-19
- **Decisor:** Diogo

## Contexto

O projeto passou pelos nomes provisórios `softFactory` e Graph Planner. O primeiro pode sugerir uma fábrica de software, enquanto o segundo pode ser confundido com uma ferramenta de grafos ou análise matemática.

O produto atual é um editor visual 2D, nativo e offline para organizar blocos de fábrica em um canvas. O nome precisa comunicar o domínio industrial e o foco na área visual de planejamento sem depender diretamente de uma marca do jogo.

## Decisão

- o nome do produto passa a ser **Factory Canvas**;
- o diretório local e o repositório GitHub passam a usar o slug `factory-canvas`;
- o pacote Cargo e o executável padrão passam a usar `factory-canvas`;
- a aplicação exibe `Factory Canvas — Arknights: Endfield` como título da janela;
- referências ao nome anterior em documentos de produto são atualizadas;
- a ADR 0001 preserva o registro histórico da decisão anterior de nome.

## Consequências

### Positivas

- o nome comunica diretamente uma ferramenta visual para fábricas;
- não sugere que o projeto constrói software;
- não fica preso ao termo técnico de grafos;
- o slug é simples e consistente entre pasta, repositório, pacote e executável.

### Negativas

- links, clones e scripts que apontavam para o repositório ou pasta antigos precisam usar `factory-canvas`;
- o crate Rust passa de `graph_planner` para `factory_canvas`;
- o arquivo SQLite legado `softfactory.db` é mantido por compatibilidade, mas seu nome não aparece mais na interface.
