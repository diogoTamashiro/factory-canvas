//! Modelo de blueprint em grid 2D do CAI (Arknights: Endfield).
//!
//! Cada blueprint = grade W x H de tiles. Cada tile pode conter o nome de uma
//! instalação (máquina) ou estar vazio. Os dados de tamanho NxN e lista de
//! instalações dos Projetos CAI foram transcritos de prints do jogo
//! (ver reference/cai-data.md).

/// Um blueprint: grade de tiles (row-major). `None` = tile vazio.
#[derive(Debug, Clone, Default)]
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
