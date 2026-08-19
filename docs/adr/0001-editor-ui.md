# ADR 0001 — Windows desktop, Rust e egui

- **Status:** Accepted — decisão de nome substituída pela ADR 0002
- **Data:** 2026-08-10
- **Decisor:** Diogo

## Contexto

O protótipo softFactory usava Rust + iced e construía o editor como uma matriz de botões. O produto foi reduzido para um planejador 2D focado em organizar blocos retangulares dentro de áreas fixas.

O modelo antigo não representa corretamente footprints maiores que um tile, rotação ou um canvas grande. A UI ainda está em fase em que uma migração tem custo baixo.

Requisitos confirmados:

- somente Windows desktop;
- aplicativo offline e nativo;
- mais leve que o render do jogo;
- visual amigável e próprio;
- código sustentável sem IA;
- canvas com pan, zoom e objetos arrastáveis.

## Decisão

- renomear o produto para **Graph Planner**;
- continuar usando Rust;
- substituir iced por `eframe/egui`;
- desenhar o layout em um único canvas customizado;
- separar domínio, UI e persistência;
- congelar Galeria, Planner, captura e solver durante o primeiro MVP;
- não criar camada de compatibilidade entre iced e egui.

## Motivos

- egui oferece painter e input adequados para ferramentas visuais;
- evita um widget por tile;
- preserva o investimento em Rust;
- permite binário nativo e offline;
- migração agora é mais barata que depois do editor crescer.

## Alternativas consideradas

### Manter iced Canvas

Menor troca de dependência, mas mais boilerplate de interação e continuidade com uma UI já considerada temporária.

### Python + PySide6/Qt

Excelente para cena 2D e UI desktop, porém exigiria trocar linguagem e produziria runtime/distribuição maiores.

### Flutter

Boa experiência visual e canvas, mas acrescentaria Dart sem necessidade de mobile ou multiplataforma.

### Tauri + React

Curva baixa para o mantenedor, mas usa WebView e contraria a preferência por UI não-web.

## Consequências

### Positivas

- domínio novo pode nascer correto e testável;
- canvas mais simples de implementar;
- menor custo de renderização que o grid de widgets;
- uma única linguagem para aplicação e modelo.

### Negativas

- UI iced existente não será reaproveitada;
- egui exige tema e componentes próprios para aparência amigável;
- acessibilidade do canvas precisa de uma lista semântica paralela;
- componentes antigos ficarão temporariamente congelados no repositório.

## Revisão

Reavaliar somente se um spike mensurável demonstrar que egui não atende desempenho, DPI, teclado ou acessibilidade mínima. Preferência estética isolada não justifica manter duas stacks.
