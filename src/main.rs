pub mod blueprint;
pub mod db;
pub mod screenshot;
pub mod solver_bridge;

use iced::widget::{button, column, row, scrollable, text, text_input, Column};
use iced::{application, Element, Task, Theme};
use serde_json::json;

/// Timestamp ISO simples (sem dependência externa de cron).
fn chrono_now() -> String {
    // Usa std::time desde a época; formato legível aproximado.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

pub fn main() -> iced::Result {
    application("Graph Planner — Arknights: Endfield", update, view)
        .theme(|_| Theme::Dark)
        .run_with(|| (State::default(), Task::done(Message::Init)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Gallery,
    Planner,
    Editor,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum EditorSubmode {
    #[default]
    Editar,
    Referencia,
}

#[derive(Default)]
struct State {
    conn: Option<rusqlite::Connection>,
    tab: Tab,
    captures: Vec<db::CaptureRow>,
    status: String,
    // Planner fields
    objective_input: String,
    space: String,
    result: String,
    // Editor (grid 2D) fields
    blueprint: blueprint::Blueprint,
    selected_machine: Option<String>,
    editor_submode: EditorSubmode,
    selected_project: Option<usize>,
    blueprint_name: String,
    validation: Vec<String>,
    diff: Vec<(usize, usize, String)>,
    import_text: String,
}

#[derive(Debug, Clone)]
enum Message {
    Init,
    SelectTab(Tab),
    Capture,

    ObjectiveChanged(String),
    SpaceChanged(String),
    Solve,
    Solved(Result<String, String>),
    // Editor
    SelectMachine(Option<String>),
    PlaceMachine(usize),
    ResizeGrid(usize, usize),
    ClearBlueprint,
    SelectEditorSubmode(EditorSubmode),
    LoadCaiProject(usize),
    BlueprintNameChanged(String),
    SaveBlueprint,
    LoadBlueprint,
    ValidateBlueprint,
    DiffBlueprint,
    ImportTextChanged(String),
    ImportBlueprint,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Init => match db::init_db() {
            Ok(conn) => {
                let captures = db::list_captures(&conn).unwrap_or_default();
                state.conn = Some(conn);
                state.captures = captures;
                state.status = "pronto".into();
                Task::none()
            }
            Err(e) => {
                state.status = format!("erro db: {e}");
                Task::none()
            }
        },
        Message::SelectTab(tab) => {
            state.tab = tab;
            Task::none()
        }
        Message::Capture => {
            if let Some(conn) = state.conn.as_ref() {
                match screenshot::capture_screen(conn) {
                    Ok(path) => {
                        state.captures = db::list_captures(conn).unwrap_or_default();
                        state.status = format!("capturado: {}", path.display());
                    }
                    Err(e) => {
                        eprintln!("capture failed: {e}");
                        state.status = format!("falha: {e}")
                    }
                }
            } else {
                eprintln!("capture blocked: db not initialized");
                state.status = "erro: db nao iniciado".into();
            }
            Task::none()
        }

        Message::ObjectiveChanged(v) => {
            state.objective_input = v;
            Task::none()
        }
        Message::SpaceChanged(v) => {
            state.space = v;
            Task::none()
        }
        Message::Solve => {
            // Parse "Item:qtd, Outro:qtd" -> objective dict.
            let mut objective = serde_json::Map::new();
            for part in state.objective_input.split(',') {
                let part = part.trim();
                if let Some((name, qty)) = part.split_once(':') {
                    if let Ok(n) = qty.trim().parse::<f64>() {
                        objective.insert(name.trim().to_string(), serde_json::Value::from(n));
                    }
                }
            }
            let space: i64 = state.space.trim().parse().unwrap_or(20);
            Task::perform(
                async move {
                    let request = json!({
                        "objective": objective,
                        "space": space,
                    });
                    solver_bridge::run_solver(&request).map_err(|e| e.to_string())
                },
                |r| {
                    Message::Solved(r.map(|v| serde_json::to_string_pretty(&v).unwrap_or_default()))
                },
            )
        }
        Message::Solved(Ok(res)) => {
            state.result = res;
            Task::none()
        }
        Message::Solved(Err(e)) => {
            state.result = format!("erro: {e}");
            Task::none()
        }
        Message::SelectMachine(m) => {
            state.selected_machine = m;
            Task::none()
        }
        Message::PlaceMachine(idx) => {
            if let Some(m) = &state.selected_machine {
                let x = idx % state.blueprint.w;
                let y = idx / state.blueprint.w;
                // Sentinelas de esteira.
                match m.as_str() {
                    "__APAGAR__" => state.blueprint.set(x, y, None),
                    "__BELT_N__" => state.blueprint.set_belt(x, y, blueprint::Direction::N),
                    "__BELT_S__" => state.blueprint.set_belt(x, y, blueprint::Direction::S),
                    "__BELT_E__" => state.blueprint.set_belt(x, y, blueprint::Direction::E),
                    "__BELT_W__" => state.blueprint.set_belt(x, y, blueprint::Direction::W),
                    _ => state.blueprint.set(x, y, Some(m.clone())),
                }
            }
            Task::none()
        }
        Message::ResizeGrid(w, h) => {
            state.blueprint = blueprint::Blueprint::new(w, h);
            Task::none()
        }
        Message::ClearBlueprint => {
            state.blueprint = blueprint::Blueprint::new(state.blueprint.w, state.blueprint.h);
            Task::none()
        }
        Message::SelectEditorSubmode(sm) => {
            state.editor_submode = sm;
            Task::none()
        }
        Message::LoadCaiProject(idx) => {
            if let Some(proj) = blueprint::CAI_PROJECTS.get(idx) {
                let mut bp = blueprint::Blueprint::new(proj.w, proj.h);
                // Distribui as instalações sequencialmente nos tiles (referência).
                let mut tile = 0;
                for (machine, qty) in proj.installations {
                    for _ in 0..*qty {
                        if tile < bp.w * bp.h {
                            bp.set(tile % bp.w, tile / bp.w, Some((*machine).to_string()));
                            tile += 1;
                        }
                    }
                }
                state.blueprint = bp;
                state.selected_project = Some(idx);
            }
            Task::none()
        }
        Message::BlueprintNameChanged(v) => {
            state.blueprint_name = v;
            Task::none()
        }
        Message::SaveBlueprint => {
            let name = if state.blueprint_name.trim().is_empty() {
                "sem-nome".to_string()
            } else {
                state.blueprint_name.trim().to_string()
            };
            let json = state.blueprint.to_json();
            let ts = chrono_now();
            // Persiste no SQLite (se disponível).
            if let Some(conn) = state.conn.as_ref() {
                match db::insert_blueprint(conn, &name, &json, &ts) {
                    Ok(()) => state.status = format!("salvo no DB: {name}"),
                    Err(e) => state.status = format!("erro DB: {e}"),
                }
            } else {
                // Fallback: arquivo JSON.
                let dir = std::path::Path::new("data/blueprints");
                let _ = std::fs::create_dir_all(dir);
                let path = dir.join(format!("{name}.json"));
                match std::fs::write(&path, &json) {
                    Ok(()) => state.status = format!("salvo: {}", path.display()),
                    Err(e) => state.status = format!("erro ao salvar: {e}"),
                }
            }
            Task::none()
        }
        Message::LoadBlueprint => {
            let name = if state.blueprint_name.trim().is_empty() {
                "sem-nome".to_string()
            } else {
                state.blueprint_name.trim().to_string()
            };
            // Tenta SQLite primeiro.
            if let Some(conn) = state.conn.as_ref() {
                match db::get_blueprint(conn, &name) {
                    Ok(Some(row)) => {
                        if let Some(bp) = blueprint::Blueprint::from_json(&row.graph_json) {
                            state.blueprint = bp;
                            state.status = format!("carregado do DB: {name}");
                        }
                    }
                    Ok(None) => {}
                    Err(e) => state.status = format!("erro DB: {e}"),
                }
            }
            // Fallback: arquivo JSON.
            let path = std::path::Path::new("data/blueprints").join(format!("{name}.json"));
            match std::fs::read_to_string(&path) {
                Ok(s) => match blueprint::Blueprint::from_json(&s) {
                    Some(bp) => {
                        state.blueprint = bp;
                        state.status = format!("carregado: {}", path.display());
                    }
                    None => state.status = "erro: JSON inválido".into(),
                },
                Err(e) => state.status = format!("erro ao carregar: {e}"),
            }
            Task::none()
        }
        Message::ValidateBlueprint => {
            if let Some(i) = state.selected_project {
                if let Some(p) = blueprint::CAI_PROJECTS.get(i) {
                    state.validation = state.blueprint.validate_against_project(p);
                } else {
                    state.validation = vec!["nenhum projeto de referência selecionado".into()];
                }
            } else {
                state.validation =
                    vec!["selecione um Projeto CAI (aba Referência) para validar".into()];
            }
            Task::none()
        }
        Message::DiffBlueprint => {
            if let Some(i) = state.selected_project {
                if let Some(p) = blueprint::CAI_PROJECTS.get(i) {
                    state.diff = state.blueprint.diff_against_project(p);
                } else {
                    state.diff = vec![];
                }
            } else {
                state.diff = vec![];
            }
            Task::none()
        }
        Message::ImportTextChanged(v) => {
            state.import_text = v;
            Task::none()
        }
        Message::ImportBlueprint => {
            match blueprint::Blueprint::from_text(&state.import_text) {
                Some(bp) => {
                    state.blueprint = bp;
                    state.status =
                        format!("importado: {}x{}", state.blueprint.w, state.blueprint.h);
                }
                None => {
                    state.status =
                        "erro: formato inválido (use 'x,y=MÁQUINA' ou 'x,y>BELT_DIR')".into()
                }
            }
            Task::none()
        }
    }
}

fn sidebar(state: &State) -> Element<'_, Message> {
    let btn = |label, tab| {
        button(text(label).size(16))
            .on_press(Message::SelectTab(tab))
            .style(if state.tab == tab {
                button::primary
            } else {
                button::secondary
            })
            .width(140)
    };
    column![
        text("Graph Planner").size(18),
        btn("Galeria", Tab::Gallery),
        btn("Planejador", Tab::Planner),
        btn("Editor", Tab::Editor),
        btn("Config", Tab::Config),
    ]
    .spacing(8)
    .padding(10)
    .into()
}

fn gallery_view(state: &State) -> Element<'_, Message> {
    let list = if state.captures.is_empty() {
        column![text("nenhuma captura ainda").size(14)].spacing(4)
    } else {
        state
            .captures
            .iter()
            .fold(Column::new().spacing(4), |col, c| {
                col.push(text(format!("#{}  {}  {}", c.id, c.ts, c.path)).size(12))
            })
    };
    column![
        button("Capturar tela").on_press(Message::Capture),
        text(&state.status).size(12),
        scrollable(list),
    ]
    .spacing(8)
    .padding(10)
    .into()
}

fn planner_view(state: &State) -> Element<'_, Message> {
    column![
        text("Planejador de producao (CAI)").size(16),
        text("Objetivo (item:qtd/min, separado por virgula)").size(12),
        text_input(
            "ex: Cilindro de Cuprium:10, Po de Originium Denso:5",
            &state.objective_input
        )
        .on_input(Message::ObjectiveChanged),
        text("Orcamento de espaco (tiles)").size(12),
        text_input("ex: 40", &state.space).on_input(Message::SpaceChanged),
        button("Resolver").on_press(Message::Solve),
        scrollable(text(&state.result).size(12)),
    ]
    .spacing(8)
    .padding(10)
    .into()
}

fn editor_view(state: &State) -> Element<'_, Message> {
    // Cabeçalho: alternar submodo + (se Referência) seletor de Projeto CAI.
    let submode_row = row![
        button(text("Editar").size(12))
            .on_press(Message::SelectEditorSubmode(EditorSubmode::Editar))
            .style(if state.editor_submode == EditorSubmode::Editar {
                button::primary
            } else {
                button::secondary
            }),
        button(text("Referência (Projeto CAI)").size(12))
            .on_press(Message::SelectEditorSubmode(EditorSubmode::Referencia))
            .style(if state.editor_submode == EditorSubmode::Referencia {
                button::primary
            } else {
                button::secondary
            }),
    ]
    .spacing(4);

    let mut project_picker = column![text("Projeto CAI:").size(12)];
    for (i, p) in blueprint::CAI_PROJECTS.iter().enumerate() {
        let sel = state.selected_project == Some(i);
        project_picker = project_picker.push(
            button(text(format!("{} ({}x{})", p.name, p.w, p.h)).size(11))
                .on_press(Message::LoadCaiProject(i))
                .style(if sel {
                    button::primary
                } else {
                    button::secondary
                })
                .width(200),
        );
    }

    // Paleta de máquinas selecionáveis (só no modo Editar).
    let mut palette = column![text("Máquina:").size(12)];
    let apagar_sel = state.selected_machine.as_deref() == Some("__APAGAR__");
    palette = palette.push(
        button(text("✖ Apagar").size(11))
            .on_press(Message::SelectMachine(Some("__APAGAR__".to_string())))
            .style(if apagar_sel {
                button::primary
            } else {
                button::secondary
            })
            .width(180),
    );
    for m in blueprint::PLACEABLE_MACHINES {
        let selected = state.selected_machine.as_deref() == Some(m);
        palette = palette.push(
            button(text(*m).size(11))
                .on_press(Message::SelectMachine(Some((*m).to_string())))
                .style(if selected {
                    button::primary
                } else {
                    button::secondary
                })
                .width(180),
        );
    }
    // Esteiras (4 direções).
    palette = palette.push(text("Esteira:").size(12));
    let belt_defs = [
        ("__BELT_N__", blueprint::Direction::N),
        ("__BELT_S__", blueprint::Direction::S),
        ("__BELT_E__", blueprint::Direction::E),
        ("__BELT_W__", blueprint::Direction::W),
    ];
    let belt_row = belt_defs.iter().fold(row![].spacing(2), |r, (s, d)| {
        let sel = state.selected_machine.as_deref() == Some(*s);
        r.push(
            button(text(d.glyph()).size(12))
                .on_press(Message::SelectMachine(Some((*s).to_string())))
                .style(if sel {
                    button::primary
                } else {
                    button::secondary
                }),
        )
    });
    palette = palette.push(belt_row);

    // Grid de tiles (botões). Clique coloca a máquina selecionada.
    let bp = &state.blueprint;
    let mut grid = column![].spacing(0);
    let tile_w = 22usize;
    for y in 0..bp.h {
        let mut row_tiles = row![].spacing(0);
        for x in 0..bp.w {
            let idx = y * bp.w + x;
            let label = match bp.get_cell(x, y) {
                blueprint::Cell::Machine(m) => {
                    // Mostra iniciais da máquina (ex: "UM" de Unidade de Montagem).
                    let ini: String = m
                        .split_whitespace()
                        .filter(|w| w.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false))
                        .take(2)
                        .map(|w| w.chars().next().unwrap().to_uppercase().to_string())
                        .collect();
                    if ini.is_empty() {
                        "■".into()
                    } else {
                        ini
                    }
                }
                blueprint::Cell::Belt(d) => d.glyph().to_string(),
                blueprint::Cell::Empty => "·".into(),
            };
            let is_belt = matches!(bp.get_cell(x, y), blueprint::Cell::Belt(_));
            row_tiles = row_tiles.push(
                button(text(label).size(9))
                    .on_press(Message::PlaceMachine(idx))
                    .style(if is_belt {
                        button::success
                    } else {
                        button::secondary
                    })
                    .width(tile_w as u16)
                    .height(tile_w as u16),
            );
        }
        grid = grid.push(row_tiles);
    }

    let counts = bp.machine_counts();
    let summary = if counts.is_empty() {
        "grid vazio".to_string()
    } else {
        let parts: Vec<String> = counts.iter().map(|(m, c)| format!("{m}: {c}")).collect();
        parts.join("  ")
    };

    let left_panel = match state.editor_submode {
        EditorSubmode::Editar => scrollable(palette).width(200),
        EditorSubmode::Referencia => scrollable(project_picker).width(200),
    };

    let header_note = match state.editor_submode {
        EditorSubmode::Editar => {
            "Selecione uma máquina à esquerda e clique num tile para colocar.".to_string()
        }
        EditorSubmode::Referencia => {
            if let Some(i) = state.selected_project {
                if let Some(p) = blueprint::CAI_PROJECTS.get(i) {
                    format!(
                        "{}  |  tags: {}  |  fornecer: {}  |  produz: {}",
                        p.name, p.tags, p.inputs, p.output
                    )
                } else {
                    "projeto não encontrado".into()
                }
            } else {
                "Escolha um Projeto CAI à esquerda para visualizar.".into()
            }
        }
    };

    let controls = match state.editor_submode {
        EditorSubmode::Editar => row![
            button("Redim 11x11").on_press(Message::ResizeGrid(11, 11)),
            button("Redim 14x9").on_press(Message::ResizeGrid(14, 9)),
            button("Redim 24x9").on_press(Message::ResizeGrid(24, 9)),
            button("Limpar").on_press(Message::ClearBlueprint),
            text_input("nome do blueprint", &state.blueprint_name)
                .on_input(Message::BlueprintNameChanged),
            button("Salvar").on_press(Message::SaveBlueprint),
            button("Carregar").on_press(Message::LoadBlueprint),
            button("Validar vs Projeto").on_press(Message::ValidateBlueprint),
            button("Diff vs Projeto").on_press(Message::DiffBlueprint),
        ]
        .spacing(4),
        EditorSubmode::Referencia => row![].spacing(0),
    };

    // Controle de import (formato texto plano).
    let import_row = row![
        text_input("importar: x,y=MÁQUINA (uma por linha)", &state.import_text)
            .on_input(Message::ImportTextChanged),
        button("Importar Texto").on_press(Message::ImportBlueprint),
    ]
    .spacing(4);

    let validation_text = if state.validation.is_empty() {
        String::new()
    } else {
        state.validation.join("\n")
    };

    let diff_text = if state.diff.is_empty() {
        String::new()
    } else {
        let lines: Vec<String> = state
            .diff
            .iter()
            .map(|(x, y, d)| format!("tile ({x},{y}): {d}"))
            .collect();
        format!(
            "Diff vs Projeto ({} tile(s) diferente(s)):\n{}",
            state.diff.len(),
            lines.join("\n")
        )
    };

    row![
        left_panel,
        column![
            submode_row,
            text(format!("Editor de Blueprint (CAI) — {}x{}", bp.w, bp.h)).size(14),
            text(header_note).size(11),
            controls,
            import_row,
            scrollable(grid),
            text(summary).size(11),
            text(validation_text).size(11),
            text(diff_text).size(11),
        ]
        .spacing(6)
        .padding(8),
    ]
    .spacing(8)
    .into()
}

fn config_view() -> Element<'static, Message> {
    column![
        text("Config").size(16),
        text("Solver: solver/solve.py (Python + OR-Tools)").size(12),
        text("Python: .venv/Scripts/python.exe").size(12),
        text("DB: data/softfactory.db (SQLite local)").size(12),
        text("Capturas: data/shots/").size(12),
    ]
    .spacing(6)
    .padding(10)
    .into()
}

fn view(state: &State) -> Element<'_, Message> {
    let content: Element<'_, Message> = match state.tab {
        Tab::Gallery => gallery_view(state),
        Tab::Planner => planner_view(state),
        Tab::Editor => editor_view(state),
        Tab::Config => config_view(),
    };
    row![sidebar(state), content].spacing(4).into()
}
