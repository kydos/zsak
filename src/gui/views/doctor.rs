use iced::widget::{button, column, text};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    column![
        text("Doctor").size(20),
        text("Checks ZSAK_HOME environment variable and zenohd in PATH.").size(13),
        button("Run Doctor").on_press(Message::DoctorRun),
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
