use iced::widget::{
    button, checkbox, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{alignment::Horizontal, Border, Element, Length, Pixels, Shadow, Theme};

use super::atoms::antd;
use super::molecules;
use super::{AntCheckbox, AntSecondaryButton, AntTextButton, AntTextEditor, AntTextInput, F11App, Message};

pub fn logs_sidebar(app: &F11App) -> Element<'_, Message> {
    let dark = app.dark_mode;
    let actions_disabled = app.running || app.stopping;

    let header = row![
        column![
            text("日志").size(16),
            text("事件输出")
                .size(12)
                .style(iced::theme::Text::Color(antd::subtext_color(dark))),
        ]
        .spacing(2)
        .width(Length::Fill),
        checkbox("跟随", app.follow_logs)
            .on_toggle(Message::ToggleFollowLogs)
            .style(iced::theme::Checkbox::Custom(Box::new(AntCheckbox))),
        button("复制")
            .style(iced::theme::Button::custom(AntTextButton))
            .on_press_maybe((!app.logs.is_empty()).then_some(Message::CopyLogs)),
        button("导出")
            .style(iced::theme::Button::custom(AntTextButton))
            .on_press_maybe((!app.logs.is_empty()).then_some(Message::ExportLogs)),
        button("清空")
            .style(iced::theme::Button::custom(AntTextButton))
            .on_press(Message::ClearLogs),
        button("收起")
            .style(iced::theme::Button::custom(AntSecondaryButton))
            .on_press_maybe((!actions_disabled).then_some(Message::CollapseLogsSidebar)),
    ]
    .spacing(10)
    .align_items(iced::Alignment::Center);

    let content = app
        .logs
        .iter()
        .fold(column![], |col, line| col.push(molecules::log_line(dark, line)));

    let padded = container(content).padding([0, 14, 0, 0]);

    let scroller = scrollable(padded)
        .id(app.log_scroll_id.clone())
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Properties::new()
                .width(Pixels(6.0))
                .scroller_width(Pixels(6.0))
                .margin(Pixels(2.0)),
        ))
        .height(Length::Fill)
        .width(Length::Fill);

    let body = molecules::card(dark, "输出", scroller.into());

    let root = column![header, body]
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill);

    let bg = if app.dark_mode {
        antd::soft_bg_dark()
    } else {
        antd::soft_bg()
    };

    container(root)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::from(move |_theme: &Theme| {
            iced::widget::container::Appearance {
                text_color: None,
                background: Some(bg),
                border: Border::default(),
                shadow: Shadow::default(),
            }
        }))
        .into()
}

pub fn main_window(app: &F11App) -> Element<'_, Message> {
    let dark = app.dark_mode;
    let dir_ready = !app.dir_input.trim().is_empty();

    let status_text = if app.stopping {
        "停止中…"
    } else if app.running {
        if app.connected {
            "运行中 · 已连接"
        } else {
            "运行中 · 未连接"
        }
    } else {
        "已停止"
    };

    let header = row![
        column![
            text("F11eSync").size(20),
            text(status_text)
                .size(12)
                .style(iced::theme::Text::Color(antd::subtext_color(dark))),
        ]
        .spacing(2)
        .width(Length::Fill),
        checkbox("跟随系统", app.follow_system_theme)
            .on_toggle(Message::ToggleFollowSystemTheme)
            .style(iced::theme::Checkbox::Custom(Box::new(AntCheckbox))),
        if app.follow_system_theme {
            row![]
        } else {
            row![checkbox("深色", app.manual_dark_mode)
                .on_toggle(Message::ToggleDarkMode)
                .style(iced::theme::Checkbox::Custom(Box::new(AntCheckbox)))]
        },
        checkbox("高级", app.show_advanced)
            .on_toggle(Message::ToggleAdvanced)
            .style(iced::theme::Checkbox::Custom(Box::new(AntCheckbox))),
    ]
    .spacing(10)
    .align_items(iced::Alignment::Center);

    const LABEL_W: f32 = 56.0;

    let dir_row = row![
        text("目录").width(Length::Fixed(LABEL_W)),
        text_input("/path/to/folder", &app.dir_input)
            .on_input(Message::DirInputChanged)
            .on_submit(Message::ApplyDirInput)
            .style(iced::theme::TextInput::Custom(Box::new(AntTextInput)))
            .width(Length::Fill),
        button("选择…")
            .style(iced::theme::Button::custom(AntSecondaryButton))
            .on_press_maybe((!app.running && !app.stopping).then_some(Message::BrowseDir)),
    ]
    .spacing(8)
    .align_items(iced::Alignment::Center);

    let host_row = row![
        text("Host").width(Length::Fixed(LABEL_W)),
        text_input("0.0.0.0", &app.host)
            .on_input(Message::HostChanged)
            .style(iced::theme::TextInput::Custom(Box::new(AntTextInput)))
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_items(iced::Alignment::Center);

    let port_row = row![
        text("Port").width(Length::Fixed(LABEL_W)),
        text_input("10080", &app.port)
            .on_input(Message::PortChanged)
            .style(iced::theme::TextInput::Custom(Box::new(AntTextInput)))
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_items(iced::Alignment::Center);

    let start_btn = if app.stopping {
        button(text("停止中…").width(Length::Fill).horizontal_alignment(Horizontal::Center))
            .style(iced::theme::Button::custom(AntSecondaryButton))
    } else if app.running {
        button(text("停止").width(Length::Fill).horizontal_alignment(Horizontal::Center))
            .style(iced::theme::Button::custom(super::AntDangerButton))
            .on_press(Message::StartStop)
    } else {
        button(text("启动").width(Length::Fill).horizontal_alignment(Horizontal::Center))
            .style(iced::theme::Button::custom(super::AntPrimaryButton))
            .on_press_maybe(dir_ready.then_some(Message::StartStop))
    };

    let actions_disabled = app.running || app.stopping;
    let controls = row![
        start_btn.padding(10).width(Length::Fill),
        button(text("检查更新").width(Length::Fill).horizontal_alignment(Horizontal::Center))
            .padding(10)
            .width(Length::Fill)
            .style(iced::theme::Button::custom(AntSecondaryButton))
            .on_press_maybe((!actions_disabled).then_some(Message::CheckUpdate)),
        button(text("下载更新").width(Length::Fill).horizontal_alignment(Horizontal::Center))
            .padding(10)
            .width(Length::Fill)
            .style(iced::theme::Button::custom(AntSecondaryButton))
            .on_press_maybe((!actions_disabled).then_some(Message::DownloadUpdate)),
    ]
    .spacing(8)
    .width(Length::Fill)
    .align_items(iced::Alignment::Center);

    let err = if app.last_error.is_empty() {
        Element::from(text(""))
    } else {
        Element::from(text(app.last_error.clone()).style(iced::theme::Text::Color(antd::ERROR)))
    };

    let basic = molecules::card(
        dark,
        "连接与目录",
        column![dir_row, host_row, port_row]
            .spacing(10)
            .width(Length::Fill)
            .into(),
    );

    let actions = molecules::card(dark, "操作", column![controls, err].spacing(8).width(Length::Fill).into());

    let advanced = if app.show_advanced {
        let size_limit_toggle = row![checkbox("启用服务端文件大小限制", app.enable_size_limit)
            .on_toggle(Message::EnableSizeLimit)
            .style(iced::theme::Checkbox::Custom(Box::new(AntCheckbox)))]
        .spacing(8)
        .align_items(iced::Alignment::Center);

        let size_limit_value = if app.enable_size_limit {
            Some(
                row![
                    text("最大(字节)").width(Length::Fixed(80.0)),
                    text_input("256000", &app.max_size)
                        .on_input(Message::MaxSizeChanged)
                        .style(iced::theme::TextInput::Custom(Box::new(AntTextInput)))
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_items(iced::Alignment::Center),
            )
        } else {
            None
        };

        let regex_editor = column![
            text("过滤规则(每行一个正则)：")
                .size(12)
                .style(iced::theme::Text::Color(antd::subtext_color(dark))),
            text("此处规则会与客户端规则合并生效")
                .size(11)
                .style(iced::theme::Text::Color(antd::subtext_color(dark))),
            text_editor(&app.server_side_regex)
                .on_action(Message::ServerRegexEdited)
                .style(iced::theme::TextEditor::Custom(Box::new(AntTextEditor)))
                .height(Length::Fixed(120.0)),
        ]
        .spacing(8);

        let mut adv_content = column![size_limit_toggle].spacing(10).width(Length::Fill);
        if let Some(v) = size_limit_value {
            adv_content = adv_content.push(v);
        }
        adv_content = adv_content.push(regex_editor);

        Some(molecules::card(dark, "高级设置", adv_content.into()))
    } else {
        None
    };

    let mut col = column![header, basic, actions].spacing(10).padding(10);
    if let Some(advanced) = advanced {
        col = col.push(advanced);
    }

    let left = scrollable(col)
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Properties::new()
                .width(Pixels(6.0))
                .scroller_width(Pixels(6.0))
                .margin(Pixels(2.0)),
        ))
        .height(Length::Fill)
        .width(Length::Fill);

    let bg = if app.dark_mode {
        antd::soft_bg_dark()
    } else {
        antd::soft_bg()
    };

    let content = if app.show_logs_sidebar {
        row![
            container(left).width(Length::Fill).height(Length::Fill),
            container(logs_sidebar(app))
                .width(Length::Fixed(380.0))
                .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
    } else {
        row![container(left).width(Length::Fill).height(Length::Fill)]
            .width(Length::Fill)
            .height(Length::Fill)
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::from(move |_theme: &Theme| {
            iced::widget::container::Appearance {
                text_color: None,
                background: Some(bg),
                border: Border::default(),
                shadow: Shadow::default(),
            }
        }))
        .into()
}
