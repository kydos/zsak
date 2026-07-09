use iced::widget::{button, column, row, text, text_input};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    column![
        text("Publish").size(20),
        row![
            text("Key:").width(80),
            text_input("key/expr", &state.publish_key)
                .on_input(Message::PublishKeyChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Value:").width(80),
            text_input("hello {N}", &state.publish_value)
                .on_input(Message::PublishValueChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Count:").width(80),
            text_input("1", &state.publish_count)
                .on_input(Message::PublishCountChanged)
                .width(80),
            text("Period (ms):").width(100),
            text_input("0", &state.publish_period)
                .on_input(Message::PublishPeriodChanged)
                .width(80),
        ].spacing(8).align_y(iced::Alignment::Center),
        button("Publish").on_press(Message::PublishRun),
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
