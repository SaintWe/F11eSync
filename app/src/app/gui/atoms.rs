pub mod antd {
    use iced::{Background, Border, Color, Shadow, Vector};

    pub const PRIMARY: Color = Color {
        r: 0x16 as f32 / 255.0,
        g: 0x77 as f32 / 255.0,
        b: 0xff as f32 / 255.0,
        a: 1.0,
    };
    pub const PRIMARY_HOVER: Color = Color {
        r: 0x40 as f32 / 255.0,
        g: 0x93 as f32 / 255.0,
        b: 0xff as f32 / 255.0,
        a: 1.0,
    };
    pub const BG: Color = Color {
        r: 0xf5 as f32 / 255.0,
        g: 0xf5 as f32 / 255.0,
        b: 0xf5 as f32 / 255.0,
        a: 1.0,
    };
    pub const BG_DARK: Color = Color {
        r: 0x14 as f32 / 255.0,
        g: 0x14 as f32 / 255.0,
        b: 0x14 as f32 / 255.0,
        a: 1.0,
    };
    pub const CARD: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const CARD_DARK: Color = Color {
        r: 0x1f as f32 / 255.0,
        g: 0x1f as f32 / 255.0,
        b: 0x1f as f32 / 255.0,
        a: 1.0,
    };
    pub const BORDER: Color = Color {
        r: 0xd9 as f32 / 255.0,
        g: 0xd9 as f32 / 255.0,
        b: 0xd9 as f32 / 255.0,
        a: 1.0,
    };
    pub const BORDER_DARK: Color = Color {
        r: 0x30 as f32 / 255.0,
        g: 0x30 as f32 / 255.0,
        b: 0x30 as f32 / 255.0,
        a: 1.0,
    };
    pub const SUBTEXT: Color = Color {
        r: 0x59 as f32 / 255.0,
        g: 0x59 as f32 / 255.0,
        b: 0x59 as f32 / 255.0,
        a: 1.0,
    };
    pub const SUBTEXT_DARK: Color = Color {
        r: 0xa6 as f32 / 255.0,
        g: 0xa6 as f32 / 255.0,
        b: 0xa6 as f32 / 255.0,
        a: 1.0,
    };
    pub const ERROR: Color = Color {
        r: 0xff as f32 / 255.0,
        g: 0x4d as f32 / 255.0,
        b: 0x4f as f32 / 255.0,
        a: 1.0,
    };
    pub const WARNING: Color = Color {
        r: 0xfa as f32 / 255.0,
        g: 0xad as f32 / 255.0,
        b: 0x14 as f32 / 255.0,
        a: 1.0,
    };

    pub fn border_color(dark: bool) -> Color {
        if dark { BORDER_DARK } else { BORDER }
    }

    pub fn subtext_color(dark: bool) -> Color {
        if dark { SUBTEXT_DARK } else { SUBTEXT }
    }

    pub fn card_border_for(dark: bool) -> Border {
        Border {
            radius: 10.0.into(),
            width: 1.0,
            color: if dark {
                Color {
                    r: 0x2a as f32 / 255.0,
                    g: 0x2a as f32 / 255.0,
                    b: 0x2a as f32 / 255.0,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0xf0 as f32 / 255.0,
                    g: 0xf0 as f32 / 255.0,
                    b: 0xf0 as f32 / 255.0,
                    a: 1.0,
                }
            },
        }
    }

    pub fn input_border(color: Color) -> Border {
        Border {
            radius: 8.0.into(),
            width: 1.0,
            color,
        }
    }

    pub fn card_shadow() -> Shadow {
        Shadow {
            color: Color { a: 0.10, ..Color::BLACK },
            offset: Vector { x: 0.0, y: 6.0 },
            blur_radius: 18.0,
        }
    }

    pub fn soft_bg() -> Background {
        Background::Color(BG)
    }

    pub fn soft_bg_dark() -> Background {
        Background::Color(BG_DARK)
    }

    pub fn card_bg(dark: bool) -> Background {
        Background::Color(if dark { CARD_DARK } else { CARD })
    }
}

pub fn ant_theme(dark: bool) -> iced::Theme {
    use iced::{theme, Color, Theme};

    let palette = if dark {
        theme::Palette {
            background: antd::BG_DARK,
            text: Color::from_rgb8(0xe8, 0xe8, 0xe8),
            primary: antd::PRIMARY,
            success: Color::from_rgb8(0x52, 0xc4, 0x1a),
            danger: antd::ERROR,
        }
    } else {
        theme::Palette {
            background: antd::BG,
            text: Color::from_rgb8(0x00, 0x00, 0x00),
            primary: antd::PRIMARY,
            success: Color::from_rgb8(0x52, 0xc4, 0x1a),
            danger: antd::ERROR,
        }
    };

    Theme::custom("AntD".to_string(), palette)
}
