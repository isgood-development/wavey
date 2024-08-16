use std::collections::HashMap;

use crate::core::format;
use crate::core::request;
use crate::ui::helpers::helper;
use crate::ui::helpers::icons;
use crate::ui::helpers::style;

use iced::widget::Space;
use iced::widget::{column, container, image, row, slider, text};
use iced::{time, Alignment, Element, Length, Task};

use tokio::time::Duration;

pub struct State {
    pub is_paused: bool,
    pub seconds_passed: u64,
    pub active_thumbnail_handle: Option<iced::advanced::image::Handle>,
    pub tracks: Vec<HashMap<String, String>>,
    pub active_video_id: String,
    pub total_duration: u64,
    pub display_name: String,

    formatted_current_duration: String,
    formatted_total_duration: String,
    slider_value: f32,
    slider_is_active: bool,
    volume: f32,
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
    InitiatePlay(
        String,
        String,
        u64,
        Option<iced::advanced::image::Handle>,
        Vec<HashMap<String, String>>,
    ),
    ThumbnailRetrieved(iced::advanced::image::Handle),
}

impl State {
    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::Mute => {
                self.volume = 0.0;

                Task::none()
            }

            Event::Unmute => {
                self.volume = 0.5;

                Task::none()
            }

            Event::VolumeChanged(value) => {
                self.volume = value;

                Task::none()
            }

            Event::Tick => {
                if self.is_paused {
                    return Task::none();
                }

                self.seconds_passed += 1;
                self.slider_value += 1.0;

                if self.seconds_passed >= self.total_duration {
                    self.slider_is_active = false;
                    self.slider_value = 0.0;
                    self.total_duration = 0;
                    self.seconds_passed = 0;

                    self.formatted_current_duration = "0:00".to_string();
                    self.formatted_total_duration = "0:00".to_string();
                    self.display_name = "Nothing is playing.".to_string();

                    let index = self
                        .tracks
                        .iter()
                        .position(|x| x.get("video_id").unwrap() == &self.active_video_id);

                    if index.is_some() {
                        let next_index = index.unwrap() + 1;

                        if next_index < self.tracks.len() {
                            let next_track = self.tracks.get(next_index).unwrap();
                            let video_id = next_track.get("video_id").unwrap().to_string();
                            let display_name = next_track.get("display_name").unwrap().to_string();
                            let total_duration =
                                next_track.get("duration").unwrap().parse::<u64>().unwrap();

                            self.display_name = display_name.clone();
                            self.slider_is_active = true;
                            self.total_duration = total_duration;
                            self.active_video_id = video_id.clone();

                            return Task::perform(
                                request::request_thumbnail_by_video_id(video_id),
                                Event::ThumbnailRetrieved,
                            );
                        }
                    }

                    return Task::none();
                }

                self.formatted_current_duration = format::duration(self.seconds_passed);
                self.formatted_total_duration = format::duration(self.total_duration);

                Task::none()
            }
            Event::BackwardPressed => Task::none(),
            Event::ForwardPressed => Task::none(),

            Event::ProgressChanged(value) => {
                self.slider_value = value;
                self.seconds_passed = value as u64;

                self.formatted_current_duration = format::duration(self.seconds_passed);
                self.formatted_total_duration = format::duration(self.total_duration);

                Task::none()
            }

            Event::InitiatePlay(video_id, display_name, total_duration, handle, tracks) => {
                self.is_paused = false;
                self.slider_is_active = false;
                self.slider_value = 0.0;
                self.seconds_passed = 0;

                self.display_name = display_name.clone();
                self.slider_is_active = true;
                self.total_duration = total_duration;
                self.tracks = tracks;
                self.active_video_id = video_id.clone();

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
                if self.is_paused {
                    self.is_paused = false;
                } else {
                    self.is_paused = true;
                }

                Task::none()
            }

            Event::SeekTo(value) => {
                self.seconds_passed = value as u64;
                self.slider_value = value;

                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
        let pause_or_play: Element<Event>;
        let volume_icon: Element<Event>;
        let thumbnail: Element<Event>;

        if self.is_paused {
            pause_or_play =
                helper::action(icons::play_icon(), "Play", Some(Event::PauseToggleAction));
        } else {
            pause_or_play =
                helper::action(icons::pause_icon(), "Pause", Some(Event::PauseToggleAction));
        }

        if self.volume == 0.0 {
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
                    text(&self.display_name).size(14),
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
                            0.0..=self.total_duration as f32,
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
                        slider(0.0..=1.0, self.volume, Event::VolumeChanged)
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
            formatted_current_duration: String::from("0:00"),
            formatted_total_duration: String::from("0:00"),
            slider_value: 0.0,
            total_duration: 0,
            seconds_passed: 1,
            slider_is_active: false,
            display_name: String::from("Nothing is playing."),
            is_paused: false,
            volume: 0.5,
            tracks: Vec::new(),
            active_video_id: String::new(),
        }
    }
}
