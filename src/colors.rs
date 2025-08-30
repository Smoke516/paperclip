use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct TokyoNightColors {
    // Background colors
    pub bg_dark: Color,
    pub bg: Color,
    pub bg_highlight: Color,
    pub terminal_black: Color,
    pub fg: Color,
    pub fg_dark: Color,
    pub fg_gutter: Color,
    pub dark3: Color,
    pub comment: Color,
    pub dark5: Color,
    pub blue0: Color,
    pub blue: Color,
    pub cyan: Color,
    pub blue1: Color,
    pub blue2: Color,
    pub blue5: Color,
    pub blue6: Color,
    pub blue7: Color,
    pub magenta: Color,
    pub magenta2: Color,
    pub purple: Color,
    pub orange: Color,
    pub yellow: Color,
    pub green: Color,
    pub green1: Color,
    pub green2: Color,
    pub teal: Color,
    pub red: Color,
    pub red1: Color,
}

impl TokyoNightColors {
    pub fn new() -> Self {
        Self {
            bg_dark: Color::Rgb(0x1a, 0x1b, 0x26),
            bg: Color::Rgb(0x24, 0x28, 0x3a),
            bg_highlight: Color::Rgb(0x29, 0x2e, 0x42),
            terminal_black: Color::Rgb(0x41, 0x48, 0x68),
            fg: Color::Rgb(0xc0, 0xca, 0xf5),
            fg_dark: Color::Rgb(0xa9, 0xb1, 0xd6),
            fg_gutter: Color::Rgb(0x3b, 0x42, 0x61),
            dark3: Color::Rgb(0x54, 0x5c, 0x7e),
            comment: Color::Rgb(0x56, 0x5f, 0x89),
            dark5: Color::Rgb(0x73, 0x7a, 0xa2),
            blue0: Color::Rgb(0x3d, 0x59, 0xa1),
            blue: Color::Rgb(0x7a, 0xa2, 0xf7),
            cyan: Color::Rgb(0x7d, 0xcf, 0xff),
            blue1: Color::Rgb(0x2a, 0xc3, 0xde),
            blue2: Color::Rgb(0x0d, 0xb9, 0xd7),
            blue5: Color::Rgb(0x89, 0xdd, 0xff),
            blue6: Color::Rgb(0xb4, 0xf9, 0xf8),
            blue7: Color::Rgb(0x39, 0x4b, 0x70),
            magenta: Color::Rgb(0xbb, 0x9a, 0xf7),
            magenta2: Color::Rgb(0xff, 0x00, 0x7c),
            purple: Color::Rgb(0x9d, 0x7c, 0xd8),
            orange: Color::Rgb(0xff, 0x9e, 0x64),
            yellow: Color::Rgb(0xe0, 0xaf, 0x68),
            green: Color::Rgb(0x9e, 0xce, 0x6a),
            green1: Color::Rgb(0x73, 0xda, 0xca),
            green2: Color::Rgb(0x41, 0xa6, 0xb5),
            teal: Color::Rgb(0x1a, 0xbc, 0x9c),
            red: Color::Rgb(0xf7, 0x76, 0x8e),
            red1: Color::Rgb(0xdb, 0x4b, 0x4b),
        }
    }
}

impl Default for TokyoNightColors {
    fn default() -> Self {
        Self::new()
    }
}
