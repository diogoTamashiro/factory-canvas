mod db;
mod screenshot;

use iced::widget::{container, text};
use iced::Task;

pub fn main() -> iced::Result {
    iced::application(
        "softFactory — Arknights: Endfield CAI Planner",
        update,
        view,
    )
    .run()
}

#[derive(Default)]
struct State {
    db_ok: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Init,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Init => match db::init_db() {
            Ok(_conn) => {
                state.db_ok = true;
                Task::none()
            }
            Err(e) => {
                eprintln!("db init failed: {e}");
                Task::none()
            }
        },
    }
}

fn view(state: &State) -> iced::Element<'_, Message> {
    let status = if state.db_ok { "DB pronto" } else { "inicializando..." };
    container(text(format!("softFactory — F1 scaffold\n{status}")).size(20))
        .padding(20)
        .into()
}
