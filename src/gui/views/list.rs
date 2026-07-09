use iced::widget::{button, column, text};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    column![
        text("List").size(20),
        text("Discovers all Zenoh runtimes (routers, peers, clients).").size(13),
        button("List All").on_press(Message::ListRun),
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
