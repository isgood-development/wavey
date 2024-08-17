use std::collections::HashMap;

use super::super::helpers::helper;
use super::super::helpers::icons;
use super::super::helpers::style;
use crate::core::db;
use crate::core::format;
use crate::core::request;

use iced::event;
use iced::event::Event as IcedEvent;
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::{
    self, button, column, container, horizontal_space, hover, row, scrollable, text, text_input,
    Space,
};
use iced::Subscription;
use iced::{Alignment, Element, Length, Task};

use log;

pub struct State {
    track_list: Vec<HashMap<String, String>>,
    show_edit_modal: bool,
    show_add_modal: bool,
    new_display_name: String,
    active_video_id: Option<String>,
    active_display_name: Option<String>,
    thumbnails_received: bool,
    thumbnails: Vec<HashMap<String, iced::advanced::image::Handle>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    HideEditModal,
    HidePlaylistModal,
    Submit,
    DeleteTrack,
    GetThumbnailHandles,
    AddToPlaylist(String, i32),
    ThumbnailsReceived(Vec<HashMap<String, iced::advanced::image::Handle>>),
    NewDisplayName(String),
    ShowEditModal(String, String),
    ShowAddModal(String),
    PlayTrack(
        String,
        String,
        u64,
        Option<iced::advanced::image::Handle>,
        Option<Vec<HashMap<String, String>>>,
    ),
    KeyboardEvent(IcedEvent),
}

impl State {
    fn new() -> Self {
        Self {
            track_list: db::get_all_music(),
            show_edit_modal: false,
            show_add_modal: false,
            new_display_name: String::new(),
            active_video_id: None,
            active_display_name: None,
            thumbnails_received: false,
            thumbnails: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::ShowAddModal(video_id) => {
                self.show_add_modal = true;
                self.active_video_id = Some(video_id);

                Task::none()
            }
            Event::HidePlaylistModal => {
                self.show_add_modal = false;
                self.active_video_id = None;

                Task::none()
            }
            Event::AddToPlaylist(video_id, playlist_id) => {
                let _ = db::add_music_playlist(video_id, playlist_id);

                self.show_add_modal = false;

                Task::none()
            }

            Event::GetThumbnailHandles => {
                self.thumbnails_received = false;

                self.track_list = db::get_all_music();
                self.track_list.reverse();

                let video_ids: Vec<String> = self
                    .track_list
                    .iter()
                    .map(|track| track.get("video_id").unwrap().clone())
                    .collect();

                Task::perform(
                    request::request_thumbnails(video_ids),
                    Event::ThumbnailsReceived,
                )
            }
            Event::ThumbnailsReceived(thumbnails) => {
                self.thumbnails = thumbnails;

                self.thumbnails_received = true;

                Task::none()
            }

            Event::PlayTrack(_video_id, _display_name, _duration, _handle, _tracks) => Task::none(),

            Event::ShowEditModal(video_id, display_name) => {
                log::info!("Showing modal for track with video_id: {}", video_id);

                self.show_edit_modal = true;
                self.active_video_id = Some(video_id);
                self.active_display_name = Some(display_name.clone());
                self.new_display_name = display_name;
                widget::focus_next()
            }
            Event::HideEditModal => {
                log::info!("Hiding modal.");
                self.hide_modals();

                Task::none()
            }
            Event::NewDisplayName(value) => {
                self.new_display_name = value;

                Task::none()
            }
            Event::Submit => {
                let active = self.active_video_id.clone().unwrap();
                let new_display_name = self.new_display_name.clone();

                self.hide_modals();

                let _ = db::edit_display_name(active, new_display_name);

                Task::none()
            }
            Event::DeleteTrack => {
                let active = self.active_video_id.clone().unwrap();
                db::delete_music(active).unwrap();

                self.hide_modals();

                self.active_video_id = None;
                Task::none()
            }
            Event::KeyboardEvent(event) => match event {
                IcedEvent::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Tab),
                    modifiers,
                    ..
                }) => {
                    if modifiers.shift() {
                        widget::focus_previous()
                    } else {
                        widget::focus_next()
                    }
                }
                IcedEvent::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Escape),
                    ..
                }) => {
                    log::info!("Hiding modal via escape key.");

                    self.hide_modals();

                    Task::none()
                }
                _ => Task::none(),
            },
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
        let mut column = column![row![
            text("My Music").size(26),
            horizontal_space(),
            button("Refresh").on_press(Event::GetThumbnailHandles),
            Space::with_width(30)
        ]
        .align_y(Alignment::Center)
        .spacing(10)];

        if self.track_list.is_empty() {
            return container(column.push(
                text("Downloaded songs will appear here, but you don't have any!").size(20),
            ))
            .padding(10)
            .into();
        }

        for audio_file in &self.track_list {
            let video_id = audio_file.get("video_id").unwrap();
            let display_name = audio_file.get("display_name").unwrap();
            let duration = audio_file.get("duration").unwrap();
            let formatted_duration = format::duration(duration.parse::<u64>().unwrap());

            let row: Element<Event>;

            if self.thumbnails_received {
                let thumbnail_handle = self
                    .thumbnails
                    .iter()
                    .find(|thumbnail| thumbnail.contains_key(video_id))
                    .unwrap()
                    .get(video_id)
                    .unwrap();

                row = hover(
                    row![
                        helper::track_list_item(
                            thumbnail_handle.clone(),
                            &display_name,
                            &duration,
                            Event::PlayTrack(
                                video_id.clone(),
                                display_name.clone(),
                                duration.parse::<u64>().unwrap(),
                                Some(thumbnail_handle.clone()),
                                Some(self.track_list.clone())
                            ),
                            Event::ShowEditModal(video_id.clone(), display_name.clone()),
                            Event::ShowAddModal(video_id.clone()),
                            false,
                        ),
                        Space::with_width(30),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                    row![
                        helper::track_list_item(
                            thumbnail_handle.clone(),
                            &display_name,
                            &duration,
                            Event::PlayTrack(
                                video_id.clone(),
                                display_name.clone(),
                                duration.parse::<u64>().unwrap(),
                                Some(thumbnail_handle.clone()),
                                Some(self.track_list.clone())
                            ),
                            Event::ShowEditModal(video_id.clone(), display_name.clone()),
                            Event::ShowAddModal(video_id.clone()),
                            true,
                        ),
                        Space::with_width(30),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                );
            } else {
                row = row![
                    helper::action(
                        icons::play_icon(),
                        display_name,
                        Some(Event::PlayTrack(
                            video_id.clone(),
                            display_name.clone(),
                            duration.parse::<u64>().unwrap(),
                            None,
                            Some(self.track_list.clone()),
                        )),
                    ),
                    text("..."),
                    Space::with_width(10),
                    text(display_name.clone()),
                    horizontal_space(),
                    text(formatted_duration.clone()),
                    Space::with_width(10),
                    helper::action(
                        icons::edit_icon(),
                        "Edit",
                        Some(Event::ShowEditModal(video_id.clone(), display_name.clone()))
                    ),
                    Space::with_width(30),
                ]
                .align_y(Alignment::Center)
                .spacing(10)
                .into();
            }

            column = column.push(row);
        }

        let content = container(
            scrollable(column.spacing(5))
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .padding(10);

        if self.show_edit_modal {
            let edit = container(
                column![
                    text("Edit Track").size(24),
                    column![
                        text("New Track Name:"),
                        text_input("Enter here...", &self.new_display_name)
                            .on_input(Event::NewDisplayName),
                    ]
                    .align_x(Alignment::Center)
                    .spacing(10),
                    row![
                        button("Delete Track")
                            .style(button::danger)
                            .on_press(Event::DeleteTrack),
                        button("Submit")
                            .style(button::success)
                            .on_press(Event::Submit),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .align_x(Alignment::Center)
                .spacing(20),
            )
            .style(container::rounded_box)
            .width(300);

            helper::modal(content, edit, Event::HideEditModal)
        } else if self.show_add_modal {
            let playlists = db::get_all_playlists();

            let mut col = column![].spacing(10).align_x(Alignment::Center);

            for playlist in playlists {
                let id = playlist.get("id").unwrap().parse::<i32>().unwrap().clone();
                let name = playlist.get("name").unwrap().clone();

                col = col.push(button(text(name)).style(style::sidebar_button).on_press(
                    Event::AddToPlaylist(self.active_video_id.clone().unwrap(), id),
                ));
            }

            let add = container(
                column![
                    text("Add to Playlist").size(24),
                    column![text("Select a playlist:"), scrollable(col)]
                        .align_x(Alignment::Center)
                        .spacing(10),
                ]
                .align_x(Alignment::Center)
                .spacing(20),
            )
            .style(container::rounded_box)
            .center_x(Length::Fill)
            .width(300);

            helper::modal(content, add, Event::HidePlaylistModal)
        } else {
            content.into()
        }
    }

    pub fn subscription(&self) -> Subscription<Event> {
        event::listen().map(Event::KeyboardEvent)
    }

    fn hide_modals(&mut self) {
        self.show_edit_modal = false;
        self.show_add_modal = false;

        self.new_display_name.clear();
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
