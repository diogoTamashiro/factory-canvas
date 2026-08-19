# Dados confirmados de layout e catálogo

Este documento registra os dados usados pelo novo domínio do Factory Canvas. Ele separa fatos confirmados, decisões internas do aplicativo e informações ainda pendentes para que estimativas não sejam tratadas como dados oficiais.

## Convenções

- dimensões de bases e footprints são expressas em tiles;
- os identificadores em inglês abaixo pertencem ao Factory Canvas, pois o jogo não fornece IDs aos jogadores;
- alcance em metros permanece em metros até existir uma conversão confirmada para tiles;
- modelos 3D e iluminação das prévias do jogo são apenas referência: o planejador representa as construções em 2D;
- espaço para conectar esteiras não aumenta o footprint físico da construção.
- as fontes usam os IDs estáveis e hashes registrados em `reference/layout-evidence.md`.

## Bases de Wuling

As duas bases são quadradas, não possuem obstáculos internos conhecidos e podem evoluir.

| Base | Nível confirmado | Dimensão | Fonte |
|---|---|---:|---|
| PAC Principal | nível atual observado | 80×80 | confirmação direta do Diogo; `BASE-01` e `BASE-02` são apenas apoio visual |
| sub-PAC | Padrão | 30×30 | `BASE-03` |
| sub-PAC | Expansão de Área I | 40×40 | `BASE-03` |
| sub-PAC | Expansão de Área II | 50×50 | `BASE-03` |

Os demais níveis da PAC Principal ainda não foram medidos. Portanto, `80×80` não deve ser usado para inferir uma progressão desconhecida. Para a sub-PAC, `50×50` é o nível conhecido após a Expansão de Área II, não sua única dimensão possível.

O tamanho 80×80 da PAC Principal já era conhecido pelo Diogo. As seleções 80×6 e 6×80 foram feitas somente para gerar retorno visual no jogo; o tamanho não foi deduzido a partir delas.

## Catálogo inicial

### Poste de Xiranita

- **ID interno proposto:** `xiranite_power_pole`
- **Nome fornecido:** Poste de energia
- **Nome visível no jogo:** Poste de Xiranita
- **Categoria:** Energia
- **Footprint:** 2×2
- **Rotações:** 0°, 90°, 180° e 270°
- **Bases:** PAC Principal e sub-PAC
- **Limite:** participa do limite de construções da região; valor numérico não confirmado
- **Espaço livre obrigatório:** nenhum
- **Alcance:** pendente de reconciliação; a coleta atual informou 80 m como máximo geral, enquanto `reference/cai-data.md` registra 80 m para NAP/relés e 30 m para instalações
- **Evidência adicional:** a prévia informa que conecta automaticamente ao NAP após a colocação
- **Fonte:** `BLOCK-01` e confirmação do Diogo

Nenhuma distância será tratada como regra confirmada nem validada no primeiro MVP até reconciliar as duas descrições e confirmar a conversão entre metros e tiles. O screenshot `BLOCK-01` comprova o texto de conexão automática ao NAP, mas não comprova as distâncias.

### Unidade de Refinaria

- **ID interno proposto:** `refinery_unit`
- **Categoria:** Produção I
- **Footprint:** 3×3
- **Rotações:** 0°, 90°, 180° e 270°
- **Bases:** PAC Principal e sub-PAC
- **Limite:** participa do limite de construções da região; valor numérico não confirmado
- **Espaço livre obrigatório:** nenhum além do footprint
- **Orientação operacional:** convém manter livres as entradas e saídas usadas pela receita; o jogador pode aceitar bloqueios quando houver pouco espaço
- **Regras conhecidas:** requer energia e deve estar na área do PAC para receber e enviar itens
- **Fontes:** `BLOCK-02` e `BLOCK-03`; ambos exibem explicitamente `Unidade de Refinaria`, nos modos de Fluido e PDR

As setas amarelas são evidência de conexões, mas suas coordenadas, tipos e disponibilidade por modo não entram no catálogo até serem medidos com precisão.

### Unidade de Trituração

- **ID interno proposto:** `crushing_unit`
- **Categoria:** Produção I
- **Footprint:** 3×3
- **Rotações:** 0°, 90°, 180° e 270°
- **Bases:** PAC Principal e sub-PAC
- **Limite:** participa do limite de construções da região; valor numérico não confirmado
- **Espaço livre obrigatório:** nenhum além do footprint
- **Orientação operacional:** convém manter livres as entradas e saídas usadas pela receita; o jogador pode aceitar bloqueios quando houver pouco espaço
- **Regras conhecidas:** requer energia e deve estar na área do PAC para receber e enviar itens
- **Fonte:** `BLOCK-04`, associado à Unidade de Trituração por correção explícita posterior do Diogo

`BLOCK-04` não exibe o nome da construção. A associação acima vem da correção posterior do usuário, que substituiu o mapeamento conflitante da descrição inicial; ela não vem de texto visível na imagem.

## Referência visual 2D

`BLOCK-05` foi identificado pelo Diogo da seguinte forma:

- construção à esquerda, com símbolo de fogo: Unidade de Refinaria;
- construção ao centro, com símbolo de forno/pedras: Unidade de Trituração;
- construção no canto superior direito: Poste de Xiranita.

Esses símbolos orientam uma representação 2D futura. O domínio atual armazena geometria e templates selecionáveis de base; identidade e catálogo de blocos pertencem à próxima tarefa. O planejador não incorporará modelos 3D ou iluminação do jogo.

## Dados ainda pendentes

- níveis e dimensões anteriores da PAC Principal;
- valores numéricos dos limites de construção por região;
- relação confirmada entre metros e tiles;
- posições, direções e tipos exatos das portas por construção, modo e receita;
- regras completas de energia e conexão entre NAP, postes, relés e instalações.
