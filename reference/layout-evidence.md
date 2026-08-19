# Manifesto de evidências de layout

Este manifesto identifica as fontes recebidas em 2026-08-19. As imagens originais não são versionadas porque algumas exibem o UID do jogador. O hash SHA-256 permite conferir que um arquivo local corresponde exatamente à fonte analisada.

## Arquivos

| ID | Arquivo original | SHA-256 | Evidência usada | Confiança |
|---|---|---|---|---|
| `BASE-01` | `composer_2026-08-19_00-36-12-118_c31d3c.png` | `5f41b8c85e861828e235e9b80345c46971d39f9b86ba1e456b85473260d03ee3` | seleção in-game de 80×6 usada como retorno visual da largura | seleção visível; PAC Principal e finalidade da seleção confirmadas pelo usuário |
| `BASE-02` | `composer_2026-08-19_00-36-46-632_d92a7b.png` | `d4ecfc98b23b660534ecad128096dd2ddd28edd169c5a28ddc7c7756caae73dc` | seleção in-game de 6×80 usada como retorno visual da altura | seleção visível; PAC Principal e finalidade da seleção confirmadas pelo usuário |
| `BASE-03` | `composer_2026-08-19_00-48-12-467_f1fda9.png` | `042a1673f3abe03356482803ddd762119b6e13d0812b76c542926c6479e91b4f` | tabela `Core AIC Area Size`: 30×30, 40×40 e 50×50; grafia `Wuling` | texto visível |
| `BLOCK-01` | `composer_2026-08-19_00-51-31-636_c91879.png` | `8104f9ae730cbb7f974d67f8c4119228702beb6348f18e9a36741516c0617853` | nome `Poste de Xiranita` e conexão automática ao NAP | texto visível; footprint confirmado pelo usuário |
| `BLOCK-02` | `composer_2026-08-19_00-54-12-381_cf772f.png` | `ef021eeddf955f39ac2f247353e82dc49e255c7d55e055e6981b2a167f1c8296` | `Unidade de Refinaria`, Modo de Fluido | texto visível; footprint confirmado pelo usuário |
| `BLOCK-03` | `composer_2026-08-19_00-58-40-861_7c9dcd.png` | `2a3735ad717e9342d7deb9ae035e88fe77637934550184273e9b13109735e651` | `Unidade de Refinaria`, Modo PDR e indicadores de conexão | texto visível; footprint confirmado pelo usuário |
| `BLOCK-04` | `composer_2026-08-19_00-59-44-325_91c0fb.png` | `c0a1d8131f930e0d87656c72673247f670b8eb3e3f8fb50a33b49cd9267f0326` | footprint e indicadores da Unidade de Trituração | identidade confirmada por correção explícita posterior; footprint confirmado pelo usuário; nome não aparece no recorte |
| `BLOCK-05` | `composer_2026-08-19_01-02-44-070_20c54a.png` | `8f3f672a213d14c2bed4e4fd341e6fc409d6a6c5de462bfa5f32a6e7f9f55454` | referência dos ícones 2D das três construções | mapeamento confirmado pelo usuário |

## Confirmações humanas associadas

Diogo confirmou na mesma coleta:

- PAC Principal e sub-PAC ficam em Wuling, são quadradas, não possuem obstáculos internos conhecidos e podem evoluir;
- a PAC Principal possui 80×80; esse tamanho já era conhecido pelo Diogo, e `BASE-01`/`BASE-02` foram produzidos apenas para obter retorno visual do tamanho;
- a sub-PAC evolui de 30×30 para 40×40 e 50×50;
- Poste de Xiranita ocupa 2×2;
- Unidade de Refinaria e Unidade de Trituração ocupam 3×3;
- as três construções permitem 0°, 90°, 180° e 270° e podem ser usadas nas duas bases;
- após uma pergunta direta para resolver o conflito da descrição inicial, `BLOCK-02` e `BLOCK-03` foram confirmados como Refinaria e `BLOCK-04` como Trituração;
- em `BLOCK-05`, esquerda/fogo é Refinaria, centro/pedras é Trituração e canto superior direito é Poste de Xiranita;
- as prévias 3D iluminadas não precisam ser reproduzidas: o Factory Canvas pode usar representação 2D.

A mensagem inicial escreveu `Wulling`; o projeto normaliza para **Wuling**, grafia visível em `BASE-03` e já usada nos dados CAI existentes.

A descrição inicial também associava os screenshots 2 e 3 à Trituração e o screenshot 4 à Refinaria. Essa associação foi substituída pela correção explícita posterior do Diogo registrada acima. O texto visível em `BLOCK-02` e `BLOCK-03` confirma Refinaria; `BLOCK-04` depende dessa correção humana porque seu recorte não mostra o nome.

## Verificação local

Com os arquivos originais disponíveis, execute:

```bash
sha256sum <arquivo>
```

O resultado deve coincidir com a coluna SHA-256. O manifesto comprova identidade do arquivo, não transforma uma interpretação visual em fato; por isso cada linha também informa seu nível de confiança.
