use iced::widget::{button, column, container, text};
use iced::{Color, Element, Length};

use crate::app::{Message, View};

pub fn sidebar_view<'a>(current: &View, connected: bool) -> Element<'a, Message> {
    let nav_btn = |label: &'static str, view: View| -> Element<'a, Message> {
        let is_active = *current == view;
        let btn = button(text(label).size(14))
            .width(Length::Fill)
            .on_press(Message::Navigate(view));
        if is_active {
            container(btn).style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.35))),
                ..Default::default()
            }).width(Length::Fill).into()
        } else {
            btn.into()
        }
    };

    let status_color = if connected {
        Color::from_rgb(0.2, 0.8, 0.2)
    } else {
        Color::from_rgb(0.8, 0.2, 0.2)
    };
    let status_label = if connected { "● Connected" } else { "● Disconnected" };

    column![
        nav_btn("Doctor", View::Doctor),
        nav_btn("Scout", View::Scout),
        nav_btn("List", View::List),
        nav_btn("Publish", View::Publish),
        nav_btn("Delete", View::Delete),
        nav_btn("Subscribe", View::Subscribe),
        nav_btn("Query", View::Query),
        nav_btn("Queryable", View::Queryable),
        nav_btn("Storage", View::Storage),
        nav_btn("Liveliness", View::Liveliness),
        nav_btn("Graph", View::Graph),
        iced::widget::vertical_space(),
        text(status_label).size(12).color(status_color),
    ]
    .spacing(2)
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
