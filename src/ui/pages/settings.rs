use super::super::helpers::theme;
use crate::core::json;
use crate::state::AppSettings;

use iced::widget::{button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Length, Task};

pub struct State {
    pub is_loaded: bool,
    pub values: Option<AppSettings>,

    theme: theme::Themes,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    ThemeSelected(theme::Themes),
    ToggleRpcEnabled,
    Continue,
    LoadSettings,
    SettingsLoaded(Option<AppSettings>),
}

impl State {
    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::SettingsLoaded(settings) => {
                self.values = settings;

                self.is_loaded = true;

                Task::none()
            }
            Event::LoadSettings => Task::perform(json::load_settings(), Event::SettingsLoaded),
            Event::Continue => Task::none(),
            Event::ThemeSelected(theme) => {
                print!("Theme selected: {:?}", theme);

                self.theme = theme;

                Task::perform(
                    json::save_settings(move |settings| {
                        settings.theme = theme.to_string();
                    }),
                    |_| Event::Continue,
                )
            }

            Event::ToggleRpcEnabled => {
                let rpc_enabled = !self.values.as_ref().unwrap().rpc_enabled;

                self.values.as_mut().unwrap().rpc_enabled = rpc_enabled;

                Task::perform(
                    json::save_settings(move |settings| {
                        settings.rpc_enabled = rpc_enabled;
                    }),
                    |_| Event::Continue,
                )
            }
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
        if !self.is_loaded {
            container(text("Loading settings..."))
                .center(Length::Fill)
                .into()
        } else {
            if self.values.is_none() {
                return container(text("Failed to load settings.")).into();
            }
            let content = container(
                scrollable(
                    column![
                        text("Settings").size(18),
                        row![
                            text("Theme:"),
                            pick_list(theme::Themes::ALL, Some(self.theme), Event::ThemeSelected),
                            text("Themes beside Light and Dark are experimental.").size(14),
                        ]
                        .align_y(Alignment::Center)
                        .spacing(10),
                        row![
                            text("Discord Rich Presence:"),
                            button(if self.values.as_ref().unwrap().rpc_enabled {
                                "Enabled"
                            } else {
                                "Disabled"
                            })
                            .on_press(Event::ToggleRpcEnabled)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(10),
                    ]
                    .spacing(40)
                    .align_x(Alignment::Start)
                    .width(Length::Fill),
                )
                .height(Length::Fill),
            )
            .padding(10);
            content.into()
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_loaded: false,
            values: None,
            theme: theme::Themes::default(),
        }
    }
}
