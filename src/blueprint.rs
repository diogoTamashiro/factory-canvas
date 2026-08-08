//! Modelo de blueprint em grid 2D do CAI (Arknights: Endfield).
//!
//! Cada blueprint = grade W x H de tiles. Cada tile pode conter o nome de uma
//! instalação (máquina) ou estar vazio. Os dados de tamanho NxN e lista de
//! instalações dos Projetos CAI foram transcritos de prints do jogo
//! (ver reference/cai-data.md).

/// Um blueprint: grade de tiles (row-major). `None` = tile vazio.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Blueprint {
    pub w: usize,
    pub h: usize,
    /// tiles[y * w + x] = nome da instalação ou None.
    pub tiles: Vec<Option<String>>,
}

impl Blueprint {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            tiles: vec![None; w * h],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&String> {
        if x < self.w && y < self.h {
            self.tiles[y * self.w + x].as_ref()
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, machine: Option<String>) {
        if x < self.w && y < self.h {
            self.tiles[y * self.w + x] = machine;
        }
    }

    /// Conta quantas vezes cada instalação aparece no grid.
    pub fn machine_counts(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for t in &self.tiles {
            if let Some(m) = t {
                *counts.entry(m.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Serializa para JSON (para salvar em disco).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Desserializa de JSON.
    pub fn from_json(s: &str) -> Option<Blueprint> {
        serde_json::from_str(s).ok()
    }

    /// Valida o grid contra um Projeto CAI de referência.
    /// Retorna lista de diferenças (o que falta / o que sobra).
    pub fn validate_against_project(&self, proj: &CaiProject) -> Vec<String> {
        let mut issues = Vec::new();
        // Tamanho.
        if self.w != proj.w || self.h != proj.h {
            issues.push(format!(
                "tamanho do grid {}x{} difere do projeto {}x{}",
                self.w, self.h, proj.w, proj.h
            ));
        }
        // Contagem por instalação (do projeto).
        let mut expected: std::collections::HashMap<String, usize> = proj
            .installations
            .iter()
            .map(|(m, q)| ((*m).to_string(), *q))
            .collect();
        let actual = self.machine_counts();
        for (m, q) in &actual {
            let exp = expected.get(m).copied().unwrap_or(0);
            if *q < exp {
                issues.push(format!("falta(m) {}x '{}' (tem {}, precisa {})", exp - q, m, q, exp));
            } else if *q > exp {
                issues.push(format!("sobra(m) {}x '{}' (tem {}, projeto usa {})", q - exp, m, q, exp));
            }
            expected.remove(m);
        }
        for (m, q) in &expected {
            if *q > 0 {
                issues.push(format!("ausente: {}x '{}' (projeto usa {})", q, m, q));
            }
        }
        if issues.is_empty() {
            issues.push("OK: grid bate com o Projeto CAI de referência.".into());
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_new_and_set() {
        let mut bp = Blueprint::new(3, 2);
        bp.set(0, 0, Some("A".into()));
        bp.set(1, 0, Some("A".into()));
        assert_eq!(bp.get(0, 0).unwrap(), "A");
        assert_eq!(bp.tiles.len(), 6);
        let counts = bp.machine_counts();
        assert_eq!(counts.get("A"), Some(&2));
    }

    #[test]
    fn validate_matches_project() {
        // Monta o grid exatamente como o projeto Xiranita Eficiente (11x11).
        let proj = &CAI_PROJECTS[0];
        let mut bp = Blueprint::new(proj.w, proj.h);
        let mut tile = 0;
        for (m, q) in proj.installations {
            for _ in 0..*q {
                bp.set(tile % bp.w, tile / bp.w, Some((*m).to_string()));
                tile += 1;
            }
        }
        let issues = bp.validate_against_project(proj);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].starts_with("OK"), "esperado OK, veio: {:?}", issues);
    }

    #[test]
    fn validate_detects_missing() {
        let proj = &CAI_PROJECTS[0]; // Xiranita Eficiente
        let bp = Blueprint::new(proj.w, proj.h); // vazio
        let issues = bp.validate_against_project(proj);
        assert!(issues.iter().any(|i| i.contains("ausente")));
    }

    #[test]
    fn json_roundtrip() {
        let mut bp = Blueprint::new(2, 2);
        bp.set(0, 0, Some("X".into()));
        let j = bp.to_json();
        let back = Blueprint::from_json(&j).expect("deve desserializar");
        assert_eq!(back.get(0, 0).unwrap(), "X");
        assert_eq!(back.w, 2);
    }
}

/// Catálogo de Projetos CAI (referência do jogo) — transcritos de prints.
#[derive(Debug, Clone)]
pub struct CaiProject {
    pub name: &'static str,
    pub w: usize,
    pub h: usize,
    pub tags: &'static str,
    /// (instalação, quantidade) — lista de instalações do projeto.
    pub installations: &'static [(&'static str, usize)],
    /// O que fornecer para o projeto rodar.
    pub inputs: &'static str,
    /// O que o projeto produz.
    pub output: &'static str,
}

/// Projetos CAI conhecidos (de reference/cai-data.md).
pub const CAI_PROJECTS: &[CaiProject] = &[
    CaiProject {
        name: "Xiranita Eficiente",
        w: 11, h: 11,
        tags: "Eficiência, Wuling, Avançado",
        installations: &[("Minerador", 1), ("Processador", 2), ("Logística", 1),
                          ("Armazenamento", 1), ("Bateria", 1), ("Estação Q", 1), ("Esteira", 1)],
        inputs: "Inergênio, Água Limpa, Carbono",
        output: "Xiranita",
    },
    CaiProject {
        name: "Engarrafamento de Xiragênio",
        w: 18, h: 11,
        tags: "Gás, Garrafa, Wuling, Avançado",
        installations: &[("Processador", 1), ("Braço", 1), ("Terminal CAI", 1),
                          ("Cilindro", 1), ("Logística", 1), ("Bateria", 1), ("Armazém", 1), ("Piso", 1)],
        inputs: "Xiranita Líquida, Cilindros de Cuprum",
        output: "Xiragênio (engarrafado)",
    },
    CaiProject {
        name: "Conversão Geral de Fluido em Gás",
        w: 9, h: 8,
        tags: "Transmutar, Wuling, Avançado",
        installations: &[("Transmutador", 1), ("Suporte", 1), ("Tanque", 1), ("Bateria", 1)],
        inputs: "material fluido, Xiranita Líquida",
        output: "estado gasoso",
    },
    CaiProject {
        name: "Xiranita Pesada",
        w: 21, h: 9,
        tags: "Xiranita, Wuling, Complexo",
        installations: &[("Extrator", 1), ("Processador", 2), ("Logística Avançada", 1),
                          ("Depósito", 1), ("Bateria", 1), ("Estação Q", 1), ("Esteira", 1)],
        inputs: "Xiranita, Água Limpa, Esgoto",
        output: "Xiranita Pesada",
    },
    CaiProject {
        name: "Solução de Hetonita",
        w: 17, h: 16,
        tags: "Solução, Purificação, Wuling",
        installations: &[("Esteira", 1), ("Processador", 1), ("Fornalha", 1), ("Tanque", 1),
                          ("Estação Log (Azul)", 1), ("Silo", 1), ("Canos", 1), ("Estação Log (Verde)", 1), ("Piso", 1)],
        inputs: "Minério de Cuprium, Água Limpa, Ácido de Precipitação",
        output: "Solução de Hetonita",
    },
    CaiProject {
        name: "Núcleo Separador",
        w: 21, h: 9,
        tags: "Eficiência, Gás, Wuling, Avançado",
        installations: &[("Britador", 1), ("Máquina Pequena", 1), ("Reator", 1),
                          ("Estação Log (2)", 1), ("Tanque H", 1), ("Cilindro Azul", 1), ("IO", 1), ("Piso", 1)],
        inputs: "Inergênio, Água Limpa, Minério de Cuprium, Xiranita",
        output: "Núcleos Separadores",
    },
    CaiProject {
        name: "Cilindro de Cuprium",
        w: 14, h: 9,
        tags: "Gás, Garrafa, Wuling, Básico",
        installations: &[("Mech", 1), ("Reator", 1), ("Logística", 1), ("Tanque", 1), ("Cilindro Gás", 1), ("Estação Q", 1), ("Piso", 1), ("Tanque Amarelo", 1)],
        inputs: "Minério de Cuprium, Inergênio",
        output: "Cilindros de Cuprium",
    },
    CaiProject {
        name: "Peça de Hetonita",
        w: 24, h: 9,
        tags: "Equipamento, Peças, Wuling",
        installations: &[("Estação Log", 1), ("Reator V", 1), ("Tanque", 1), ("Gerador", 1),
                          ("Montador", 2), ("Centrífuga", 1), ("Bateria", 1), ("Forno", 1), ("Esteira", 1)],
        inputs: "Solução de Hetonita, Minério de Ferrium",
        output: "Peças de Hetonita",
    },
    CaiProject {
        name: "Peça de Cuprium",
        w: 12, h: 7,
        tags: "Esgoto, Peças, Wuling",
        installations: &[("Bateria", 1), ("Forno", 1), ("Reator Químico", 1), ("Silo", 1), ("Esteira", 1), ("Estação Q", 1), ("Piso", 1)],
        inputs: "Minério de Cuprium, Água Limpa",
        output: "Peças de Cuprium",
    },
    CaiProject {
        name: "Seringa de Broto-Agulha [A]",
        w: 15, h: 12,
        tags: "Medicamentos, Wuling",
        installations: &[("Montador", 1), ("Fundidor", 1), ("Reator Químico", 1), ("Gerador", 1), ("Tanque L", 1), ("Baú", 1), ("Piso", 1)],
        inputs: "Garrafas de Cuprium, Solução de Broto-Agulha",
        output: "Seringas de Broto-Agulha [A]",
    },
    CaiProject {
        name: "Bateria CP de Wuling",
        w: 8, h: 7,
        tags: "Bateria, Wuling",
        installations: &[("Gerador", 1), ("Porto Drone", 1), ("Logística", 1), ("Piso", 1)],
        inputs: "Xircônio, Pó de Originium Denso",
        output: "Baterias CP de Wuling",
    },
    CaiProject {
        name: "Pó de Originium Denso",
        w: 13, h: 6,
        tags: "Moer, Vale",
        installations: &[("Produção", 1), ("Torre", 2), ("Hub Log", 1), ("Estação Log (Verde)", 1), ("Esteira", 1)],
        inputs: "Folha Rugosa, Minério de Originium",
        output: "Pó de Originium Denso",
    },
];

/// Lista de instalações que o usuário pode colocar no editor (catálogo de máquinas).
pub const PLACEABLE_MACHINES: &[&str] = &[
    "Unidade de Trituração",
    "Unidade de Montagem",
    "Unidade de Moldagem",
    "Unidade de Plantio",
    "Unidade de Coleta de Sementes",
    "Unidade de Tratamento de Água",
    "Unidade de Enchimento",
    "Unidade de Embalagem",
    "Unidade de Moagem",
    "Crisol do Reator",
    "Crisol Expandido",
    "Forja dos Céus",
    "Unidade de Purificação",
    "Unidade de Separação",
    "Unidade de Transmutação Fluido-Gás",
    "Unidade de Transmutação Sólido-Gás",
    "Poste de Xiranita",
    "Relé de Xiranita",
];
