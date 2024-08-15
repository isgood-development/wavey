use std::fmt;

use iced::Theme;

pub fn match_theme(new: Option<Themes>) -> Theme {
    match new {
        Some(theme) => match theme {
            Themes::Dark => Theme::Dark,
            Themes::Light => Theme::Light,
            Themes::Dracula => Theme::Dracula,
            Themes::Nord => Theme::Nord,
            Themes::SolarizedLight => Theme::SolarizedLight,
            Themes::SolarizedDark => Theme::SolarizedDark,
            Themes::GruvboxLight => Theme::GruvboxLight,
            Themes::GruvboxDark => Theme::GruvboxDark,
            Themes::CatppuccinLatte => Theme::CatppuccinLatte,
            Themes::CatppuccinFrappe => Theme::CatppuccinFrappe,
            Themes::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            Themes::CatppuccinMocha => Theme::CatppuccinMocha,
            Themes::TokyoNight => Theme::TokyoNight,
            Themes::TokyoNightStorm => Theme::TokyoNightStorm,
            Themes::TokyoNightLight => Theme::TokyoNightLight,
            Themes::KanagawaWave => Theme::KanagawaWave,
            Themes::KanagawaDragon => Theme::KanagawaDragon,
            Themes::KanagawaLotus => Theme::KanagawaLotus,
            Themes::Moonfly => Theme::Moonfly,
            Themes::Nightfly => Theme::Nightfly,
            Themes::Oxocarbon => Theme::Oxocarbon,
            Themes::Ferra => Theme::Ferra,
        },
        None => todo!(),
    }
}

pub fn get_theme_from_settings(name: &String) -> Theme {
    match name.as_str() {
        "Dark" => Theme::Dark,
        "Light" => Theme::Light,
        "Dracula" => Theme::Dracula,
        "Nord" => Theme::Nord,
        "Solarized Light" => Theme::SolarizedLight,
        "Solarized Dark" => Theme::SolarizedDark,
        "Gruvbox Light" => Theme::GruvboxLight,
        "Gruvbox Dark" => Theme::GruvboxDark,
        "Catppuccin Latte" => Theme::CatppuccinLatte,
        "Catppuccin Frappe" => Theme::CatppuccinFrappe,
        "Catppuccin Macchiato" => Theme::CatppuccinMacchiato,
        "Catppuccin Mocha" => Theme::CatppuccinMocha,
        "Tokyo Night" => Theme::TokyoNight,
        "Tokyo Night Storm" => Theme::TokyoNightStorm,
        "Tokyo Night Light" => Theme::TokyoNightLight,
        "Kanagawa Wave" => Theme::KanagawaWave,
        "Kanagawa Dragon" => Theme::KanagawaDragon,
        "Kanagawa Lotus" => Theme::KanagawaLotus,
        "Moonfly" => Theme::Moonfly,
        "Nightfly" => Theme::Nightfly,
        "Oxocarbon" => Theme::Oxocarbon,
        "Ferra" => Theme::Ferra,
        _ => Theme::Dark,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Themes {
    #[default]
    Dark,
    Light,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

impl Themes {
    pub const ALL: &'static [Self] = &[
        Self::Dark,
        Self::Light,
        Self::Dracula,
        Self::Nord,
        Self::SolarizedLight,
        Self::SolarizedDark,
        Self::GruvboxLight,
        Self::GruvboxDark,
        Self::CatppuccinLatte,
        Self::CatppuccinFrappe,
        Self::CatppuccinMacchiato,
        Self::CatppuccinMocha,
        Self::TokyoNight,
        Self::TokyoNightStorm,
        Self::TokyoNightLight,
        Self::KanagawaWave,
        Self::KanagawaDragon,
        Self::KanagawaLotus,
        Self::Moonfly,
        Self::Nightfly,
        Self::Oxocarbon,
        Self::Ferra,
    ];
}

impl fmt::Display for Themes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::SolarizedLight => "Solarized Light",
            Self::SolarizedDark => "Solarized Dark",
            Self::GruvboxLight => "Gruvbox Light",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::TokyoNight => "Tokyo Night",
            Self::TokyoNightStorm => "Tokyo Night Storm",
            Self::TokyoNightLight => "Tokyo Night Light",
            Self::KanagawaWave => "Kanagawa Wave",
            Self::KanagawaDragon => "Kanagawa Dragon",
            Self::KanagawaLotus => "Kanagawa Lotus",
            Self::Moonfly => "Moonfly",
            Self::Nightfly => "Nightfly",
            Self::Oxocarbon => "Oxocarbon",
            Self::Ferra => "Ferra",
        }
        .fmt(f)
    }
}
