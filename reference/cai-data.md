# Dados reais do CAI (Arknights: Endfield) — transcritos de prints do jogo

Fonte: prints em `C:\Users\diogo\OneDrive\Imagens\Screenshots` (2026-08-08), banco de dados
"Arquivos de Instalações" + telas de "Prévia de Projeto". Termo oficial PT-BR: **CAI**
(autom. industrial). Em inglês: AIC.

Observação-chave: toda máquina tem ciclo base de **2s** (= 30/min por receita) ou **10s**
(= 6/min). "30" que o Diogo citou = 60s / 2s.

## Instalações (máquinas) — Fórmulas Disponíveis

### Produção I
- **Unidade de Trituração** — Energia 5 — Triturar
  - Modo PDR: várias (Cristal Vermelho->Cristal Vermelho, Bloco Cinza->Bloco Cinza x2,
    Planta->Pó Vermelho x2, Planta->Pó Amarelo, etc.) — ciclo 2s
- **Unidade de Montagem** — Energia 20 — Montar
  - Cristal Azul (1) -> Placa de Circuito Azul (1) [2s]
  - Haste Roxa (1) -> Placa de Circuito Verde (1) [2s]
  - Cristal Vermelho (1) -> Placa de Circuito Vermelha (1) [2s]
  - Cristal Vermelho (5) -> Placa de Circuito Roxa (1) [10s]
- **Unidade de Moldagem** — Energia 10 — Molde
  - Modo PDR: 10x [Lingote] -> 1x [Caixa] [2s]; 2x [Cabo Roxo] -> 1x [Frasco Químico Azul] [2s];
    2x Cristal Azul -> 1x [Frasco Azul] [2s]; 2x [Capacete] -> 1x [Frasco Roxo] [2s];
    2x Minério Vermelho -> 1x [Frasco Vermelho] [2s]; 2x Minério Vermelho -> 1x [Frasco Marrom] [2s]
  - Modo de Gás: 2x Minério Vermelho + 1x Cristal Azul -> 1x [Frasco Marrom] [2s]
- **Unidade de Plantio** — Energia 20 — Cultura
  - Modo PDR: 1x Baga Vermelha + 1x Arbusto Verde -> 1x Baga Vermelha [2s];
    1x Fruta Amarela + 1x Arbusto Verde -> 1x Flor Amarela [2s];
    1x Raiz Marrom + 1x Flor Amarela -> 1x Flor Amarela [2s];
    1x Flor Branca + 1x Galho Marrom -> 1x Galho Marrom [2s]
  - Modo de Fluido: 1x Esfera Branca + 1x Gota Azul -> 2x Flor Branca [2s];
    1x Varetas Marrons + 1x Gota Azul -> 2x Planta Verde [2s]
- **Unidade de Coleta de Sementes** — Energia 10 — Semente
  - 1x Erva Verde -> 2x Baga Vermelha [2s]; 1x Erva Verde -> 2x Fruta Amarela [2s];
    1x Flor Amarela -> 2x Flor Amarela [2s]; 1x Galho -> 2x Flor Branca [2s];
    1x Flor Branca -> 2x Melão Verde [2s]; 1x Erva Verde -> 2x Varetas [2s];
    1x Trigo -> 2x Farelo Marrom [2s]; 1x Pimenta Vermelha -> 2x Pó Vermelho [2s]
- **Unidade de Tratamento de Água** — Energia 50 — Tratar
  - 1x [Efluente Azul] -> Tratamento concluído (Água Limpa) [2s]
  - 1x [Efluente Verde] -> Tratamento concluído [2s]
  - 1x [Efluente de Xircônio] -> Tratamento concluído [2s]

### Produção II
- **Unidade de Enchimento** — Energia 20 — Encher — Modo de Gás e Fluido
  - [Frasco] + [Algodão] -> [Frasco cheio] [2s] (várias variantes de cor)
  - [Garrafa Azul] + [Líquido Azul] -> [Garrafa Azul cheia] [2s]
- **Unidade de Embalagem** — Energia 20 — Pacote
  - 5x Circuito Azul + 10x Bloco Marrom -> 1x Bateria [10s]; 10x Circuito Azul + 5x Cristal Verde -> 1x Bateria [10s];
    5x Cristal Verde + 15x Bloco Marrom -> 1x Bateria [10s]; 1x Pote Marrom + 1x Cristal Verde -> 2x Cilindro Cinza [2s]
- **Unidade de Moagem** — Energia 50 — Moagem
  - 2x Cristal Azul + 1x Minério Amarelo -> 1x Bloco Azul [2s]; 2x Rocha Roxa + 1x Minério Amarelo -> 1x Bloco Branco [2s];
    2x Rocha Escura + 1x Minério Amarelo -> 1x Bloco Escuro [2s]; 2x Rocha Cinza + 1x Minério Amarelo -> 1x Bloco Cinza [2s];
    2x Rocha Marrom + 1x Minério Amarelo -> 1x Bloco Marrom [2s]; 2x Rocha Rosa + 1x Minério Amarelo -> 1x Bloco Rosa [2s];
    2x Rocha Amarela + 1x Minério Amarelo -> 1x Bloco Amarelo [2s]
- **Crisol do Reator** — Energia 50 — Reação química
  - 1x Pó Cinza + 1x Gota Azul -> 1x Gota Branca [2s]; 1x Minério Marrom + 1x Gota Azul -> 1x Gota Verde [2s];
    1x Cristal Verde + 1x Gota Azul -> 1x Cristal Verde [2s]; 1x Cristal Verde + 1x Gota Amarela -> 1x Cristal Verde [2s];
    1x Minério Marrom + 1x Gota Amarela -> 1x Gota Vermelha [2s];
    1x Gota Verde + 1x Gota Cinza -> 1x Cristal Verde + 1x Cristal Verde Escuro [2s];
    2x Gota Azul + 1x Gota Cinza Escura -> 1x Cristal Escuro + 1x Gota Cinza [2s];
    2x Gota Vermelha + 1x Gota Cinza Escura -> 1x Cristal Vermelho + 1x Gota Cinza [2s]
- **Crisol Expandido** — Energia 100 — Reação (maior)
  - Rocha + Água -> Bloco Mineral [2s]; Planta + Água -> Planta [2s];
    Rocha Marrom + Água -> Planta Verde [2s]; Rocha Marrom + Água -> Líquido Vermelho [2s];
    Planta Verde + Rocha Cinza -> 2x Cristal Verde [2s]; 2x Água + Rocha Escura -> Cristal Escuro + Gota d'Água [2s];
    2x Chama + Rocha Escura -> Cristal Vermelho + Gota d'Água [2s]
- **Forja dos Céus** — Energia 50 — Xiranita
  - 2x Bloco Azul + 1x Líquido Azul -> 1x Bloco Verde [2s]; 10x Bloco Preto + 5x Líquido Azul -> 1x Bloco Verde [10s]
- **Unidade de Purificação** — Energia 50 — Purificar
  - Modo de Fluido: 4x Líquido Verde -> 1x Líquido Verde Escuro + 1x Líquido Azul [2s];
    4x Líquido Vermelho -> 1x Líquido Rosa + 1x Líquido Amarelo [2s]
  - Modo de Gás: 2x Gás Verde + 2x Motor -> 1x Gás Verde [2s]; 2x Gás Rosa + 2x Motor -> 1x Gás Rosa [2s]
- **Unidade de Separação** — Energia 20 — Separar
  - várias (Recipiente + Fluff -> Recipiente + Fluff) [2s] — separação de compostos
- **Unidade de Transmutação Fluido-Gás** — Energia 50
  - Gaseificar: Líquido Azul -> Gás Branco [2s]; Líquido Amarelo -> Gás Branco [2s];
    Líquido Verde -> Gás Branco [2s]; 2x Líquido Verde -> 5x Gás Verde [10s];
    Líquido Vermelho -> Gás Rosa [2s]; Líquido Branco -> Gás Branco [2s]
  - Fluidificar: Gás Branco -> Líquido Azul [2s] (e inverso)
- **Unidade de Transmutação Sólido-Gás** — Energia 50
  - Gaseificar: 1x Cristal Verde Escuro -> 1x Nuvem Branca [2s]; 2x Cristal Verde Escuro -> 5x Nuvem Verde [10s];
    2x Cristal Vermelho -> 1x Nuvem Rosa [2s]; 1x Cristal Vermelho -> 2x Nuvem Rosa [2s]
  - Solidificar: 1x Nuvem Branca -> 1x Cristal Verde Escuro [2s]; 5x Nuvem Verde -> 2x Cristal Verde Escuro [10s];
    1x Nuvem Rosa -> 1x Cristal Vermelho [2s]

### Energia (não produzem item, transmitem)
- **Poste de Xiranita** — Energia — conecta NAP/Relés a 80m, instalações a 30m
- **Relé de Xiranita** — Energia — conecta instalações de energia a 80m

## Blueprint de Projetos (Prévia de Projeto CAI) — "forneça X para produzir Y"

| Projeto | Tamanho | Tags | Fornecer | Produzir |
|---------|---------|------|----------|----------|
| Xiranita Eficiente | 11x11 | Eficiência, Wuling, Avançado | Inergênio, Água Limpa, Carbono | Xiranita |
| Engarrafamento de Xiragênio | 18x11 | Gás, Garrafa, Wuling, Avançado | Xiranita Líquida, Cilindros de Cuprum | Xiragênio (engarrafado) |
| Conversão Geral de Fluido em Gás | 9x8 | Transmutar, Wuling, Avançado | material fluido + Xiranita Líquida | estado gasoso |
| Xiranita Pesada | 21x9 | Xiranita, Wuling, Complexo | Xiranita, Água Limpa, Esgoto | Xiranita Pesada |
| Solução de Hetonita | 17x16 | Solução, Purificação, Wuling | Minério de Cuprium, Água Limpa, Ácido de Precipitação | Solução de Hetonita |
| Núcleo Separador | 21x9 | Eficiência, Gás, Wuling, Avançado | Inergênio, Água Limpa, Minério de Cuprium, Xiranita | Núcleos Separadores |
| Cilindro de Cuprium | 14x9 | Gás, Garrafa, Wuling, Básico | Minério de Cuprium, Inergênio | Cilindros de Cuprium |
| Peça de Hetonita | 24x9 | Equipamento, Peças, Wuling | Solução de Hetonita, Minério de Ferrium | Peças de Hetonita |
| Peça de Cuprium | 12x7 | Esgoto, Peças, Wuling | Minério de Cuprium, Água Limpa | Peças de Cuprium |
| Seringa de Broto-Agulha [A] | 15x12 | Medicamentos, Wuling | Garrafas de Cuprium, Solução de Broto-Agulha | Seringas de Broto-Agulha [A] |
| Bateria CP de Wuling | 8x7 | Bateria, Wuling | Xircônio, Pó de Originium Denso | Baterias CP de Wuling |
| Pó de Originium Denso | 13x6 | Moer, Vale | Folha Rugosa, Minério de Originium | Pó de Originium Denso |

## Mecânica de Esteiras (F3.2) — transcrita de print do AIC

Fonte: print "tempo de viagem / intervalo entre itens" do AIC atual (2026-08-08).
Cada bloco de logística = 1 tile. Latência inicial (tempo de viagem do 1º item) = **2s** para
todos os tipos. A vazão (intervalo entre itens na linha) depende do bloco:

| Tipo de Bloco | Espaço | Latência | Intervalo (vazão) |
|---------------|--------|----------|-------------------|
| Esteira Básica | 1 tile | 2s | 1 item / 2s = **30/min** |
| Divisor (Splitter) 2 saídas | 1 tile | 2s | 1 item / 4s = **15/min por linha** |
| Divisor (Splitter) 3 saídas | 1 tile | 2s | 1 item / 6s = **10/min por linha** |
| Integrador (Merger) | 1 tile | 2s | **vide correção abaixo** |

### CORREÇÃO (Diogo, 2026-08-08)
O print mostra o Integrador com "1 item / 2s (30/min)" — **isso está errado**.
O Integrador junta N linhas de entrada; o tempo de transporte aumenta conforme a
quantidade de entradas (máx 3). Regra (espelho do Divisor):

- Integrador 1 entrada: 1 item / 2s = **30/min**
- Integrador 2 entradas: 1 item / 4s = **15/min**
- Integrador 3 entradas: 1 item / 6s = **10/min**

Ou seja, intervalo de saída = (n_entradas) × 2s. Limite segue a Esteira Básica (30/min
com 1 entrada cheia).

## Modelo de conexão de máquinas (F3.2 / reforma do editor)

Decisões estruturais (Diogo, 2026-08-08) — valem para o app e para o solver de fluxo:

- **1 esteira = 1 tile.** 1 divisor/integrador = 1 tile.
- **Máquinas têm entradas e saídas FIXAS** (posições definidas pela instalação),
  mas podem ser **giradas** (rotação 0/90/180/270) para o player apontar na direção desejada.
- **Uma máquina pode ter N entradas e N saídas** — depende do propósito dela.
  - Ex.: **Triturador** e **Refinadora** têm **3 entradas e 3 saídas**.
  - Ex.: receitas de Triturador que consomem 3 entradas (3 linhas de matéria-prima).
- **No geral**, de uma máquina sai **1 esteira** (1 linha de saída), mesmo que ela tenha
  múltiplas saídas possíveis — o player liga só a que precisa.
- A futura parte gráfica que monta blueprints **vai mudar** (renderer novo), mas o
  modelo de dados (tiles + conexões + rotação) é este.
- **CORREÇÃO de tamanho:** máquinas NÃO são 1×1. A maioria ocupa retângulos maiores:
  exemplos citados: **3×3**, **7×4**, **2×2** — depende do propósito da máquina.
  (O assumption "1 máquina = 1 tile" da F2.A era só para o planner simplificado.)

## Notas para o solver (F2.A)
- Cada máquina = 1 tile (assumir 1x1 para F2.A; grids reais vêm na F2.B).
- Taxa por máquina = (saída por ciclo) × (60 / tempo_ciclo_segundos).
- Espaço disponível = orçamento de tiles (input do usuário).
- Objetivo do usuário = itens/min desejados; solver devolve qtd de cada máquina + viabilidade.
