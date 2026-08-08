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
struct State;

fn update(_state: &mut State, _message: ()) -> Task<()> {
    Task::none()
}

fn view(_state: &State) -> iced::Element<'_, ()> {
    container(text("softFactory — F1 scaffold").size(24))
        .padding(20)
        .into()
}
