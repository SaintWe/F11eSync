use iced::widget::{button as button_widget, checkbox as checkbox_style, text_editor as text_editor_widget, text_input as text_input_widget};
use iced::{Background, Border, Color, Theme, Vector};

use super::atoms::antd;

#[derive(Debug, Clone, Copy)]
pub struct AntPrimaryButton;

impl button_widget::StyleSheet for AntPrimaryButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(antd::PRIMARY)),
            text_color: Color::WHITE,
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            shadow_offset: Vector { x: 0.0, y: 1.0 },
            ..button_widget::Appearance::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(antd::PRIMARY_HOVER)),
            ..self.active(_style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            shadow_offset: Vector::default(),
            ..self.hovered(style)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AntDangerButton;

impl button_widget::StyleSheet for AntDangerButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(antd::ERROR)),
            text_color: Color::WHITE,
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            shadow_offset: Vector { x: 0.0, y: 1.0 },
            ..button_widget::Appearance::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            background: Some(Background::Color(Color::from_rgb8(0xff, 0x78, 0x75))),
            ..self.active(_style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            shadow_offset: Vector::default(),
            ..self.hovered(style)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AntSecondaryButton;

impl button_widget::StyleSheet for AntSecondaryButton {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button_widget::Appearance {
        let palette = style.extended_palette();
        let dark = palette.background.base.color == antd::BG_DARK;
        let surface = if dark { antd::CARD_DARK } else { antd::CARD };

        button_widget::Appearance {
            background: Some(Background::Color(surface)),
            text_color: palette.background.base.text,
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: antd::border_color(dark),
            },
            ..button_widget::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let mut a = self.active(style);
        a.border.color = antd::PRIMARY;
        a
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        button_widget::Appearance {
            shadow_offset: Vector::default(),
            ..self.hovered(style)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AntTextButton;

impl button_widget::StyleSheet for AntTextButton {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button_widget::Appearance {
        let palette = style.extended_palette();
        button_widget::Appearance {
            background: None,
            text_color: palette.primary.base.color,
            border: Border::default(),
            ..button_widget::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button_widget::Appearance {
        let palette = style.extended_palette();
        button_widget::Appearance {
            text_color: palette.primary.strong.color,
            ..self.active(style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button_widget::Appearance {
        self.hovered(style)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AntTextInput;

impl text_input_widget::StyleSheet for AntTextInput {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_input_widget::Appearance {
        let palette = style.extended_palette();
        let dark = palette.background.base.color == antd::BG_DARK;
        let surface = if dark { antd::CARD_DARK } else { antd::CARD };
        text_input_widget::Appearance {
            background: Background::Color(surface),
            border: antd::input_border(antd::border_color(dark)),
            icon_color: palette.secondary.base.text,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input_widget::Appearance {
        let mut a = self.active(style);
        a.border = antd::input_border(antd::PRIMARY);
        a
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();
        palette.secondary.base.color
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();
        palette.background.base.text
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();
        palette.secondary.weak.color
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color { a: 0.35, ..antd::PRIMARY }
    }

    fn disabled(&self, style: &Self::Style) -> text_input_widget::Appearance {
        let mut a = self.active(style);
        let palette = style.extended_palette();
        let dark = palette.background.base.color == antd::BG_DARK;
        a.background = Background::Color(if dark {
            Color::from_rgb8(0x1a, 0x1a, 0x1a)
        } else {
            Color::from_rgb8(0xf5, 0xf5, 0xf5)
        });
        a
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AntTextEditor;

impl text_editor_widget::StyleSheet for AntTextEditor {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_editor_widget::Appearance {
        let palette = style.extended_palette();
        let dark = palette.background.base.color == antd::BG_DARK;
        let surface = if dark { antd::CARD_DARK } else { antd::CARD };
        text_editor_widget::Appearance {
            background: Background::Color(surface),
            border: antd::input_border(antd::border_color(dark)),
        }
    }

    fn focused(&self, style: &Self::Style) -> text_editor_widget::Appearance {
        let mut a = self.active(style);
        a.border = antd::input_border(antd::PRIMARY);
        a
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();
        palette.secondary.base.color
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();
        palette.background.base.text
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        let palette = style.extended_palette();
        palette.secondary.weak.color
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color { a: 0.35, ..antd::PRIMARY }
    }

    fn disabled(&self, style: &Self::Style) -> text_editor_widget::Appearance {
        let mut a = self.active(style);
        let palette = style.extended_palette();
        let dark = palette.background.base.color == antd::BG_DARK;
        a.background = Background::Color(if dark {
            Color::from_rgb8(0x1a, 0x1a, 0x1a)
        } else {
            Color::from_rgb8(0xf5, 0xf5, 0xf5)
        });
        a
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AntCheckbox;

impl checkbox_style::StyleSheet for AntCheckbox {
    type Style = Theme;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox_style::Appearance {
        let palette = style.extended_palette();
        let bg = palette.background.base.color;
        let dark = bg == antd::BG_DARK;
        let surface = if dark { antd::CARD_DARK } else { antd::CARD };

        checkbox_style::Appearance {
            background: Background::Color(if is_checked { antd::PRIMARY } else { surface }),
            icon_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: if is_checked { antd::PRIMARY } else { antd::border_color(dark) },
            },
            text_color: Some(palette.background.base.text),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox_style::Appearance {
        let mut a = self.active(style, is_checked);
        if !is_checked {
            a.border.color = antd::PRIMARY;
        }
        a
    }
}

