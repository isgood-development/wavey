use super::super::helpers::theme;
use crate::core::json;
use crate::state::AppSettings;

use iced::widget::{button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Length, Task};

pub struct State {
    pub values: Option<AppSettings>,

    theme: theme::Themes,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    ThemeSelected(theme::Themes),
    ToggleRpcEnabled,
    Continue,
}

impl State {
    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::Continue => Task::none(),
            Event::ThemeSelected(theme) => Task::perform(json::save_settings(AppSettings {
                    theme: theme.to_string(),
                    ..Default::default()
                }), |_| Event::Continue),
            

            Event::ToggleRpcEnabled => {
                if self.values.as_ref().unwrap().rpc_enabled {
                    self.values.as_mut().unwrap().rpc_enabled = false;
                } else {
                    self.values.as_mut().unwrap().rpc_enabled = true;
                }

                Task::perform(json::save_settings(AppSettings {
                    rpc_enabled: self.values.as_ref().unwrap().rpc_enabled,
                    ..Default::default()
                }), |_| Event::Continue)
            }
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
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

impl Default for State {
    fn default() -> Self {
        Self {
            values: None,

            theme: theme::Themes::default(),
        }
    }
}
