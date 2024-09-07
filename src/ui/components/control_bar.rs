use crate::core::format;
use crate::core::request;
use crate::state;
use crate::ui::helpers::helper;
use crate::ui::helpers::icons;
use crate::ui::helpers::style;

use iced::widget::Space;
use iced::widget::{column, container, image, row, slider, text};
use iced::{time, Alignment, Element, Length, Task};

use tokio::time::Duration;

pub struct State {
    pub player_state: state::PlayerState,

    active_thumbnail_handle: Option<iced::advanced::image::Handle>,
    volume_slider: f32,
    formatted_current_duration: String,
    formatted_total_duration: String,
    slider_value: f32,
    slider_is_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    BackwardPressed,
    ForwardPressed,
    PauseToggleAction,
    Tick,
    Mute,
    Unmute,
    SeekTo(f32),
    ProgressChanged(f32),
    VolumeChanged(f32),
    InitiatePlay(String, Option<iced::advanced::image::Handle>),
    ThumbnailRetrieved(iced::advanced::image::Handle),
}

impl State {
    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::Mute => {
                self.volume_slider = 0.0;

                Task::none()
            }

            Event::Unmute => {
                self.volume_slider = 0.5;

                Task::none()
            }

            Event::VolumeChanged(value) => {
                self.volume_slider = value;

                Task::none()
            }

            Event::Tick => {
                if self.player_state.is_paused {
                    return Task::none();
                }

                self.player_state.seconds_passed += 1;
                self.slider_value += 1.0;

                if self.player_state.seconds_passed >= self.player_state.total_duration {
                    self.slider_is_active = false;
                    self.slider_value = 0.0;
                    self.player_state.total_duration = 0;
                    self.player_state.seconds_passed = 0;

                    self.formatted_current_duration = "0:00".to_string();
                    self.formatted_total_duration = "0:00".to_string();
                    self.player_state.display_name = "Nothing is playing.".to_string();

                    let index = self.player_state.queued_tracks.iter().position(|x| {
                        x.get("video_id").unwrap() == &self.player_state.active_video_id
                    });

                    if index.is_some() {
                        let next_index = index.unwrap() + 1;

                        if next_index < self.player_state.queued_tracks.len() {
                            let next_track =
                                self.player_state.queued_tracks.get(next_index).unwrap();
                            let video_id = next_track.get("video_id").unwrap().to_string();
                            let display_name = next_track.get("display_name").unwrap().to_string();
                            let total_duration =
                                next_track.get("duration").unwrap().parse::<u64>().unwrap();

                            self.player_state.display_name = display_name.clone();
                            self.slider_is_active = true;
                            self.player_state.total_duration = total_duration;
                            self.player_state.active_video_id = video_id.clone();

                            return Task::perform(
                                request::request_thumbnail_by_video_id(video_id),
                                Event::ThumbnailRetrieved,
                            );
                        }
                    }

                    return Task::none();
                }

                self.formatted_current_duration =
                    format::duration(self.player_state.seconds_passed);
                self.formatted_total_duration = format::duration(self.player_state.total_duration);

                Task::none()
            }
            Event::BackwardPressed => Task::none(),
            Event::ForwardPressed => Task::none(),

            Event::ProgressChanged(value) => {
                self.slider_value = value;
                self.player_state.seconds_passed = value as u64;

                self.formatted_current_duration =
                    format::duration(self.player_state.seconds_passed);
                self.formatted_total_duration = format::duration(self.player_state.total_duration);

                Task::none()
            }

            Event::InitiatePlay(video_id, handle) => {
                self.slider_value = 0.0;
                self.slider_is_active = true;
                self.formatted_current_duration = "0:00".to_string();
                self.formatted_total_duration = "0:00".to_string();

                if handle.is_none() {
                    return Task::perform(
                        request::request_thumbnail_by_video_id(video_id),
                        Event::ThumbnailRetrieved,
                    );
                } else {
                    self.active_thumbnail_handle = handle;
                }

                Task::none()
            }

            Event::ThumbnailRetrieved(handle) => {
                self.active_thumbnail_handle = Some(handle);

                Task::none()
            }

            Event::PauseToggleAction => {
                if self.player_state.is_paused {
                    self.player_state.is_paused = false;
                } else {
                    self.player_state.is_paused = true;
                }

                Task::none()
            }

            Event::SeekTo(value) => {
                self.player_state.seconds_passed = value as u64;
                self.slider_value = value;

                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
        let pause_or_play: Element<Event>;
        let volume_icon: Element<Event>;
        let thumbnail: Element<Event>;

        if self.player_state.is_paused {
            pause_or_play =
                helper::action(icons::play_icon(), "Play", Some(Event::PauseToggleAction));
        } else {
            pause_or_play =
                helper::action(icons::pause_icon(), "Pause", Some(Event::PauseToggleAction));
        }

        if self.volume_slider == 0.0 {
            volume_icon = helper::action(icons::volume_off(), "Unmute", Some(Event::Mute));
        } else {
            volume_icon = helper::action(icons::volume_on(), "Mute", Some(Event::Unmute));
        }

        if self.active_thumbnail_handle.is_none() {
            thumbnail = container(text("")).into();
        } else {
            thumbnail = container(
                image(self.active_thumbnail_handle.clone().unwrap())
                    .width(90)
                    .height(60),
            )
            .into();
        }

        container(
            row![
                Space::with_width(10),
                container(thumbnail).width(Length::FillPortion(3)),
                column![
                    text(&self.player_state.display_name).size(14),
                    row![
                        helper::action(
                            icons::backward_icon(),
                            "Back",
                            Some(Event::BackwardPressed)
                        ),
                        pause_or_play,
                        helper::action(
                            icons::forward_icon(),
                            "Forward",
                            Some(Event::ForwardPressed)
                        ),
                    ]
                    .spacing(10),
                    row![
                        text(&self.formatted_current_duration).size(14),
                        slider(
                            0.0..=self.player_state.total_duration as f32,
                            self.slider_value,
                            Event::ProgressChanged
                        )
                        .width(350)
                        .step(1.0),
                        text(&self.formatted_total_duration).size(14),
                    ]
                    .spacing(10),
                ]
                .spacing(5)
                .align_x(Alignment::Center)
                .max_width(400)
                .width(Length::FillPortion(7)),
                container(
                    row![
                        volume_icon,
                        slider(0.0..=1.0, self.volume_slider, Event::VolumeChanged)
                            .step(0.1)
                            .width(120)
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                )
                .width(Length::FillPortion(3))
            ]
            .align_y(Alignment::Center)
            .spacing(10),
        )
        .width(Length::Fill)
        .style(style::dynamic_colour)
        .center_x(Length::Fill)
        .height(100)
        .padding(10)
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Event> {
        if self.slider_is_active {
            return time::every(Duration::from_secs(1)).map(|_| Event::Tick);
        }

        iced::Subscription::none()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            active_thumbnail_handle: None,
            player_state: state::PlayerState::default(),
            formatted_current_duration: String::from("0:00"),
            formatted_total_duration: String::from("0:00"),
            slider_value: 0.0,
            slider_is_active: false,
            volume_slider: 0.5,
        }
    }
}
