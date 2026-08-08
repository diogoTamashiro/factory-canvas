pub mod db;
pub mod screenshot;
pub mod solver_bridge;
pub mod blueprint;

use iced::widget::{button, column, row, scrollable, text, text_input, Column};
use iced::{application, Element, Settings, Task, Theme};
use serde_json::json;

pub fn main() -> iced::Result {
    application(
        "softFactory — Arknights: Endfield CAI Planner",
        update,
        view,
    )
    .theme(|_| Theme::Dark)
    .run_with(|| (State::default(), Task::done(Message::Init)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Gallery,
    Planner,
    Config,
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
}

#[derive(Debug, Clone)]
enum Message {
    Init,
    SelectTab(Tab),
    Capture,
    ReloadCaptures,
    ObjectiveChanged(String),
    SpaceChanged(String),
    Solve,
    Solved(Result<String, String>),
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
        Message::ReloadCaptures => {
            if let Some(conn) = state.conn.as_ref() {
                state.captures = db::list_captures(conn).unwrap_or_default();
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
                        objective.insert(
                            name.trim().to_string(),
                            serde_json::Value::from(n),
                        );
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
                |r| Message::Solved(r.map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())),
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
        text("softFactory").size(18),
        btn("Galeria", Tab::Gallery),
        btn("Planejador", Tab::Planner),
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
        state.captures.iter().fold(Column::new().spacing(4), |col, c| {
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
        text_input("ex: Cilindro de Cuprium:10, Po de Originium Denso:5", &state.objective_input)
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
        Tab::Config => config_view(),
    };
    row![sidebar(state), content].spacing(4).into()
}
