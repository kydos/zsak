pub mod delete;
pub mod doctor;
pub mod graph;
pub mod liveliness;
pub mod list;
pub mod publish;
pub mod query;
pub mod queryable;
pub mod scout;
pub mod sidebar;
pub mod storage;
pub mod subscribe;

use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::{AppState, Message};

pub fn config_modal(state: &AppState) -> Element<Message> {
    let config_file = state.session_config.config_file.clone().unwrap_or_default();
    let form = column![
        text("Session Configuration").size(16),
        iced::widget::horizontal_rule(1),
        row![
            text("Config file:").width(120),
            text_input("(optional path)", &config_file)
                .on_input(Message::ConfigFileChanged)
                .width(Length::Fill),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Mode:").width(120),
            text_input("peer|client|router", &state.session_config.mode)
                .on_input(Message::ConfigModeChanged)
                .width(Length::Fill),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Connect to:").width(120),
            text_input("[\"tcp/host:7447\"]", &state.session_config.endpoints)
                .on_input(Message::ConfigEndpointsChanged)
                .width(Length::Fill),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Listen:").width(120),
            text_input("[\"tcp/0.0.0.0:7447\"]", &state.session_config.listen)
                .on_input(Message::ConfigListenChanged)
                .width(Length::Fill),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Name:").width(120),
            text_input("zsak-gui", &state.session_config.name)
                .on_input(Message::ConfigNameChanged)
                .width(Length::Fill),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("REST port:").width(120),
            text_input("8000", &state.session_config.rest_port)
                .on_input(Message::ConfigRestPortChanged)
                .width(Length::Fill),
        ].spacing(8).align_y(iced::Alignment::Center),
        checkbox("Admin space", state.session_config.admin_enabled)
            .on_toggle(Message::ConfigAdminToggled),
        checkbox("Multicast scouting", state.session_config.multicast_scouting)
            .on_toggle(Message::ConfigMulticastToggled),
        {
            let err_text: Element<Message> = if let Some(ref e) = state.session_error {
                text(e.as_str())
                    .color(iced::Color::from_rgb(1.0, 0.3, 0.3))
                    .into()
            } else {
                iced::widget::Space::new(0, 0).into()
            };
            err_text
        },
        row![
            iced::widget::horizontal_space(),
            button("Cancel").on_press(Message::HideConfig),
            button("Connect").on_press(Message::ConnectSession),
        ].spacing(8),
    ]
    .spacing(10)
    .padding(20)
    .width(500);

    container(
        container(form)
            .style(container::rounded_box)
            .padding(4),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
        ..Default::default()
    })
    .into()
}
