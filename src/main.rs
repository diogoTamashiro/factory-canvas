mod db;
mod screenshot;

use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Element, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(
        "softFactory — Arknights: Endfield CAI Planner",
        update,
        view,
    )
    .theme(|_| Theme::Dark)
    .run()
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
}

#[derive(Debug, Clone)]
enum Message {
    Init,
    SelectTab(Tab),
    Capture,
    ReloadCaptures,
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
                    Err(e) => state.status = format!("falha: {e}"),
                }
            }
            Task::none()
        }
        Message::ReloadCaptures => {
            if let Some(conn) = state.conn.as_ref() {
                state.captures = db::list_captures(conn).unwrap_or_default();
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

fn placeholder(title: &str) -> Element<'_, Message> {
    container(text(format!("{title} — em breve")).size(16))
        .padding(10)
        .into()
}

fn view(state: &State) -> Element<'_, Message> {
    let content: Element<'_, Message> = match state.tab {
        Tab::Gallery => gallery_view(state),
        Tab::Planner => placeholder("Planejador"),
        Tab::Config => placeholder("Config"),
    };
    row![sidebar(state), content].spacing(4).into()
}
