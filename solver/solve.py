"""softFactory solver — minimal CAI production optimizer (mock recipes).

Reads a JSON objective + space budget from stdin, builds a tiny LP model
with Google OR-Tools, and writes the solution JSON to stdout.

Input JSON:
{
  "objective": {"Steel": 10},   # desired units/min
  "space": 20,                   # max tiles available
  "recipes": [ ... ]             # optional override; defaults to MOCK below
}

Output JSON:
{
  "feasible": true,
  "machines": {"Furnace": 5},
  "throughput": {"Steel": 5.0}
}
"""

import json
import sys

# Mock CAI recipes (illustrative, not real game data):
#   Furnace:   2 Iron/min  -> 1 Steel/min,  occupies 1 tile
#   Smelter:   3 Copper/min -> 2 Wire/min,   occupies 2 tiles
MOCK_RECIPES = [
    {
        "name": "Furnace",
        "inputs": {"Iron": 2.0},
        "outputs": {"Steel": 1.0},
        "tiles": 1,
    },
    {
        "name": "Smelter",
        "inputs": {"Copper": 3.0},
        "outputs": {"Wire": 2.0},
        "tiles": 2,
    },
]


def solve(data: dict) -> dict:
    from ortools.linear_solver import pywraplp

    recipes = data.get("recipes") or MOCK_RECIPES
    objective = data.get("objective", {})
    space = data.get("space", 20)

    if not objective:
        return {"feasible": False, "machines": {}, "throughput": {}, "reason": "no objective"}

    solver = pywraplp.Solver.CreateSolver("GLOP")
    if solver is None:
        return {"feasible": False, "machines": {}, "throughput": {}, "reason": "solver unavailable"}

    # Decision vars: how many machines of each recipe.
    counts = {r["name"]: solver.IntVar(0, solver.infinity(), r["name"]) for r in recipes}

    # Space constraint.
    solver.Add(sum(r["tiles"] * counts[r["name"]] for r in recipes) <= space)

    # For each requested output, produce at least the objective amount.
    for out_name, target in objective.items():
        produced = sum(
            r["outputs"].get(out_name, 0.0) * counts[r["name"]]
            for r in recipes
            if out_name in r["outputs"]
        )
        solver.Add(produced >= float(target))

    # Objective: maximize total throughput (sum of all outputs).
    total = sum(
        r["outputs"].get(o, 0.0) * counts[r["name"]]
        for r in recipes
        for o in r["outputs"]
    )
    solver.Maximize(total)

    status = solver.Solve()
    if status not in (pywraplp.Solver.OPTIMAL, pywraplp.Solver.FEASIBLE):
        return {"feasible": False, "machines": {}, "throughput": {}, "reason": "infeasible"}

    machines = {name: int(var.solution_value()) for name, var in counts.items()}
    throughput = {}
    for r in recipes:
        for o, rate in r["outputs"].items():
            throughput[o] = throughput.get(o, 0.0) + rate * machines[r["name"]]

    return {"feasible": True, "machines": machines, "throughput": throughput}


def main() -> None:
    raw = sys.stdin.read()
    data = json.loads(raw) if raw.strip() else {}
    result = solve(data)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
