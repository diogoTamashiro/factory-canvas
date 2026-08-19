"""Factory Canvas solver — CAI (Arknights: Endfield) production planner.

Reads a JSON objective + space budget from stdin, builds an LP model with
Google OR-Tools, and writes the solution JSON to stdout.

Dados reais transcritos de prints do jogo (ver reference/cai-data.md).
Cada máquina roda uma Fórmula (receita). Ciclo base: 2s (=30/min) ou 10s (=6/min).
Taxa por máquina = saida_por_ciclo * (60 / tempo_ciclo_s).

Input JSON:
{
  "objective": {"Cilindro de Cuprium": 10, "Pó de Originium Denso": 5},
  "space": 40,
  "recipes": [ ... ]   // opcional; default = RECIPES abaixo
}

Output JSON:
{
  "feasible": true,
  "machines": {"Unidade de Coleta de Sementes": 3, ...},
  "throughput": {"Baga Vermelha": 6.0, ...},
  "bottlenecks": ["espaco insuficiente para X"]
}
"""

import json
import sys
import unicodedata


def norm(s: str) -> str:
    """Normaliza nome p/ casar: minusculo + sem acento + trim."""
    s = unicodedata.normalize("NFKD", s)
    s = "".join(c for c in s if not unicodedata.combining(c))
    return s.lower().strip()

# Recipes reais do CAI (PT-BR). Cada receita = 1 máquina rodando 1 fórmula.
# inputs/outputs em unidades por ciclo; cycle em segundos.
RECIPES = [
    # --- Produção I ---
    {"machine": "Unidade de Plantio", "cycle": 2,
     "inputs": {}, "outputs": {"Baga Vermelha": 1}},
    {"machine": "Unidade de Coleta de Sementes", "cycle": 2,
     "inputs": {}, "outputs": {"Baga Vermelha": 2}},
    {"machine": "Unidade de Coleta de Sementes", "cycle": 2,
     "inputs": {}, "outputs": {"Fruta Amarela": 2}},
    {"machine": "Unidade de Coleta de Sementes", "cycle": 2,
     "inputs": {}, "outputs": {"Flor Branca": 2}},
    {"machine": "Unidade de Trituração", "cycle": 2,
     "inputs": {}, "outputs": {"Pó Amarelo": 1}},
    {"machine": "Unidade de Trituração", "cycle": 2,
     "inputs": {}, "outputs": {"Pó Vermelho": 2}},
    {"machine": "Unidade de Tratamento de Água", "cycle": 2,
     "inputs": {}, "outputs": {"Água Limpa": 1}},
    # --- Produção II ---
    {"machine": "Unidade de Moagem", "cycle": 2,
     "inputs": {"Cristal Azul": 2, "Minério Amarelo": 1}, "outputs": {"Bloco Azul": 1}},
    {"machine": "Unidade de Montagem", "cycle": 2,
     "inputs": {"Cristal Azul": 1}, "outputs": {"Placa de Circuito Azul": 1}},
    {"machine": "Unidade de Montagem", "cycle": 10,
     "inputs": {"Cristal Vermelho": 5}, "outputs": {"Placa de Circuito Roxa": 1}},
    {"machine": "Unidade de Moldagem", "cycle": 2,
     "inputs": {"Cristal Azul": 2}, "outputs": {"Frasco Azul": 1}},
    {"machine": "Unidade de Embalagem", "cycle": 10,
     "inputs": {"Circuito Azul": 5, "Bloco Marrom": 10}, "outputs": {"Bateria": 1}},
    {"machine": "Unidade de Embalagem", "cycle": 2,
     "inputs": {"Pote Marrom": 1, "Cristal Verde": 1}, "outputs": {"Cilindro Cinza": 2}},
    {"machine": "Crisol do Reator", "cycle": 2,
     "inputs": {"Pó Cinza": 1, "Gota Azul": 1}, "outputs": {"Gota Branca": 1}},
    {"machine": "Crisol do Reator", "cycle": 2,
     "inputs": {"Minério Marrom": 1, "Gota Azul": 1}, "outputs": {"Gota Verde": 1}},
    {"machine": "Forja dos Céus", "cycle": 2,
     "inputs": {"Bloco Azul": 2, "Líquido Azul": 1}, "outputs": {"Bloco Verde": 1}},
    {"machine": "Unidade de Purificação", "cycle": 2,
     "inputs": {"Líquido Verde": 4}, "outputs": {"Líquido Verde Escuro": 1, "Líquido Azul": 1}},
    {"machine": "Unidade de Separação", "cycle": 2,
     "inputs": {"Frasco Azul": 1, "Gota Azul": 1}, "outputs": {"Frasco Azul": 1, "Gota Azul": 1}},
    {"machine": "Unidade de Transmutação Fluido-Gás", "cycle": 2,
     "inputs": {"Líquido Verde": 1}, "outputs": {"Gás Branco": 1}},
    {"machine": "Unidade de Transmutação Fluido-Gás", "cycle": 10,
     "inputs": {"Líquido Verde": 2}, "outputs": {"Gás Verde": 5}},
    {"machine": "Unidade de Transmutação Sólido-Gás", "cycle": 2,
     "inputs": {"Cristal Verde Escuro": 1}, "outputs": {"Nuvem Branca": 1}},
    {"machine": "Unidade de Transmutação Sólido-Gás", "cycle": 10,
     "inputs": {"Cristal Verde Escuro": 2}, "outputs": {"Nuvem Verde": 5}},
    # --- Projetos CAI (receitas de alto nível, 1 máquina "virtual" por projeto) ---
    {"machine": "Projeto: Cilindro de Cuprium", "cycle": 2,
     "inputs": {"Minério de Cuprium": 1, "Inergênio": 1}, "outputs": {"Cilindro de Cuprium": 1}},
    {"machine": "Projeto: Pó de Originium Denso", "cycle": 2,
     "inputs": {"Folha Rugosa": 1, "Minério de Originium": 1}, "outputs": {"Pó de Originium Denso": 1}},
    {"machine": "Projeto: Seringa de Broto-Agulha [A]", "cycle": 2,
     "inputs": {"Garrafa de Cuprium": 1, "Solução de Broto-Agulha": 1},
     "outputs": {"Seringa de Broto-Agulha [A]": 1}},
    {"machine": "Projeto: Peça de Hetonita", "cycle": 2,
     "inputs": {"Solução de Hetonita": 1, "Minério de Ferrium": 1}, "outputs": {"Peça de Hetonita": 1}},
    {"machine": "Projeto: Peça de Cuprium", "cycle": 2,
     "inputs": {"Minério de Cuprium": 1, "Água Limpa": 1}, "outputs": {"Peça de Cuprium": 1}},
    {"machine": "Projeto: Bateria CP de Wuling", "cycle": 2,
     "inputs": {"Xircônio": 1, "Pó de Originium Denso": 1}, "outputs": {"Bateria CP de Wuling": 1}},
    {"machine": "Projeto: Solução de Hetonita", "cycle": 2,
     "inputs": {"Minério de Cuprium": 1, "Água Limpa": 1, "Ácido de Precipitação": 1},
     "outputs": {"Solução de Hetonita": 1}},
    {"machine": "Projeto: Xiranita Eficiente", "cycle": 2,
     "inputs": {"Inergênio": 1, "Água Limpa": 1, "Carbono": 1}, "outputs": {"Xiranita": 1}},
    {"machine": "Projeto: Xiranita Pesada", "cycle": 2,
     "inputs": {"Xiranita": 1, "Água Limpa": 1, "Esgoto": 1}, "outputs": {"Xiranita Pesada": 1}},
    {"machine": "Projeto: Engarrafamento de Xiragênio", "cycle": 2,
     "inputs": {"Xiranita Líquida": 1, "Cilindro de Cuprium": 1}, "outputs": {"Xiragênio Engarrafado": 1}},
    {"machine": "Projeto: Núcleo Separador", "cycle": 2,
     "inputs": {"Inergênio": 1, "Água Limpa": 1, "Minério de Cuprium": 1, "Xiranita": 1},
     "outputs": {"Núcleo Separador": 1}},
    {"machine": "Projeto: Conversão Geral de Fluido em Gás", "cycle": 2,
     "inputs": {"material fluido": 1, "Xiranita Líquida": 1}, "outputs": {"estado gasoso": 1}},
]


def rate_per_min(r: dict) -> dict:
    """Throughput (unidades/min) de UMA máquina rodando esta receita, por output."""
    per_cycle = 60.0 / r["cycle"]
    return {out: qty * per_cycle for out, qty in r["outputs"].items()}


def solve(data: dict) -> dict:
    from ortools.linear_solver import pywraplp

    recipes = data.get("recipes") or RECIPES
    objective = data.get("objective", {})
    space = data.get("space", 40)

    if not objective:
        return {"feasible": False, "machines": {}, "throughput": {},
                "bottlenecks": ["sem objetivo"]}

    solver = pywraplp.Solver.CreateSolver("CP_SAT")
    if solver is None:
        return {"feasible": False, "machines": {}, "throughput": {},
                "bottlenecks": ["solver indisponível"]}

    # Decision var: quantas máquinas rodam cada receita (índice i).
    counts = [solver.IntVar(0, solver.infinity(), f"m{i}") for i in range(len(recipes))]

    # Space constraint (1 máquina = 1 tile, assumindo F2.A).
    solver.Add(sum(counts) <= space)

    # Throughput por output (expressão linear) de TODAS as máquinas.
    all_outputs = sorted(set().union(*[set(r["outputs"]) for r in recipes]))
    produced = {}
    for out in all_outputs:
        terms = [rate_per_min(recipes[i]).get(out, 0.0) * counts[i]
                 for i in range(len(recipes))]
        produced[norm(out)] = solver.Sum(terms)  # lista, não generator

    # Cada objetivo deve ser produzido >= alvo (nome normalizado p/ casar acentos).
    bottlenecks = []
    for out_name, target in objective.items():
        key = norm(out_name)
        if key not in produced:
            bottlenecks.append(f"receita para '{out_name}' não encontrada")
            continue
        solver.Add(produced[key] >= float(target))

    # Minimizar número total de máquinas (layout enxuto de manutenção).
    solver.Minimize(solver.Sum([counts[i] for i in range(len(recipes))]))

    status = solver.Solve()
    if status not in (pywraplp.Solver.OPTIMAL, pywraplp.Solver.FEASIBLE):
        return {"feasible": False, "machines": {}, "throughput": {},
                "bottlenecks": ["inviável com o espaço informado"] + bottlenecks}

    machines = {}
    for i, r in enumerate(recipes):
        v = int(counts[i].solution_value())
        if v > 0:
            machines[r["machine"]] = machines.get(r["machine"], 0) + v

    throughput = {}
    for i, r in enumerate(recipes):
        for out, qty in rate_per_min(r).items():
            throughput[out] = throughput.get(out, 0.0) + qty * int(counts[i].solution_value())

    return {"feasible": True, "machines": machines,
            "throughput": {k: round(v, 2) for k, v in throughput.items()},
            "bottlenecks": bottlenecks}


def main() -> None:
    raw = sys.stdin.read()
    data = json.loads(raw) if raw.strip() else {}
    result = solve(data)
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
