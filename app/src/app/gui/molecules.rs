use iced::widget::{column, container, text};
use iced::{Element, Length, Shadow, Theme};

use super::atoms::antd;
use super::atom_helper_log::{classify_log_line, LogLevel};
use super::Message;

pub fn card<'a>(dark: bool, title: &str, content: Element<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(13)
                .style(iced::theme::Text::Color(antd::subtext_color(dark))),
            content
        ]
        .spacing(8),
    )
    .padding(10)
    .width(Length::Fill)
    .style(iced::theme::Container::from(move |_theme: &Theme| {
        iced::widget::container::Appearance {
            text_color: None,
            background: Some(antd::card_bg(dark)),
            border: antd::card_border_for(dark),
            shadow: if dark { Shadow::default() } else { antd::card_shadow() },
        }
    }))
    .into()
}

pub fn log_line<'a>(dark: bool, line: &str) -> Element<'a, Message> {
    let color = match classify_log_line(line) {
        LogLevel::Error => antd::ERROR,
        LogLevel::Warn => antd::WARNING,
        LogLevel::Info | LogLevel::Other => {
            if dark {
                iced::Color::from_rgb8(0xd9, 0xd9, 0xd9)
            } else {
                iced::Color::from_rgb8(0x00, 0x00, 0x00)
            }
        }
        LogLevel::Debug => antd::subtext_color(dark),
    };

    text(line.to_string())
        .size(12)
        .style(iced::theme::Text::Color(color))
        .into()
}
