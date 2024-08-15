use std::collections::HashMap;

use super::super::helpers::helper;
use super::super::helpers::icons;
use super::super::helpers::style;
use crate::core::db;
use crate::core::format;
use crate::core::request;

use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input,
};
use iced::{Alignment, Length, Task};

pub struct State {
    create_playlist_mode: bool,
    playlist_view: bool,
    playlist_name_input: String,
    playlists: Vec<HashMap<String, String>>,
    tracks: Vec<HashMap<String, String>>,
    thumbnails_recieved: bool,
    thumbnails: Vec<HashMap<String, iced::advanced::image::Handle>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    OpenInListMode,
    OpenInCreateMode,
    CreatePlaylist,
    PlayTrack(
        String,
        String,
        u64,
        Option<iced::advanced::image::Handle>,
        Option<Vec<HashMap<String, String>>>,
    ),
    ThumbnailHandlesReceived(Vec<HashMap<String, iced::advanced::image::Handle>>),
    OpenPlaylist(i32),
    PlaylistNameInput(String),
}

impl State {
    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::PlayTrack(_video_id, _display_name, _duration, _handle, _tracks) => Task::none(),
            Event::OpenPlaylist(index) => {
                self.playlist_view = true;
                self.thumbnails = Vec::new();
                self.thumbnails_recieved = false;
                self.tracks = Vec::new();

                let playlist_tracks = db::get_playlist_tracks(
                    self.playlists[index as usize]
                        .get("id")
                        .unwrap()
                        .parse::<i32>()
                        .unwrap(),
                );

                for track in playlist_tracks {
                    match db::get_music_from_id(
                        track.get("music_id").unwrap().parse::<i32>().unwrap(),
                    ) {
                        Ok(music) => self.tracks.push(music),
                        Err(e) => {
                            log::error!("Failed to get music from id: {:?}", e);
                        }
                    }
                }

                Task::perform(
                    request::request_thumbnail_from_playlist(self.tracks.clone()),
                    Event::ThumbnailHandlesReceived,
                )
            }

            Event::ThumbnailHandlesReceived(thumbnails) => {
                self.thumbnails = thumbnails;
                self.thumbnails_recieved = true;

                Task::none()
            }

            Event::OpenInListMode => {
                self.playlist_view = false;
                self.create_playlist_mode = false;

                Task::none()
            }

            Event::CreatePlaylist => {
                self.playlist_view = false;
                self.create_playlist_mode = false;

                let _ = db::add_playlist(self.playlist_name_input.clone());

                self.playlists = db::get_all_playlists();

                Task::none()
            }
            Event::OpenInCreateMode => {
                self.create_playlist_mode = true;

                Task::none()
            }

            Event::PlaylistNameInput(value) => {
                self.playlist_name_input = value;

                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
        if self.playlist_view {
            let mut col = column![];
            if self.thumbnails_recieved {
                for track in self.tracks.iter() {
                    let handle = self
                        .thumbnails
                        .iter()
                        .find(|x| x.contains_key(track.get("video_id").unwrap()))
                        .unwrap()
                        .get(track.get("video_id").unwrap())
                        .unwrap();

                    col = col.push(
                        row![
                            helper::action(
                                icons::play_icon(),
                                "Play",
                                Some(Event::PlayTrack(
                                    track.get("video_id").unwrap().to_string(),
                                    track.get("display_name").unwrap().to_string(),
                                    track
                                        .get("duration")
                                        .unwrap()
                                        .parse::<i32>()
                                        .unwrap()
                                        .try_into()
                                        .unwrap(),
                                    Some(handle.clone()),
                                    Some(self.tracks.clone()),
                                ))
                            ),
                            helper::thumbnail(handle.clone()),
                            text(track.get("display_name").unwrap()),
                            horizontal_space(),
                            text(format::duration(
                                track
                                    .get("duration")
                                    .unwrap()
                                    .parse::<i32>()
                                    .unwrap()
                                    .try_into()
                                    .unwrap()
                            )),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    );
                }
            } else {
                for track in self.tracks.iter() {
                    col = col.push(
                        row![
                            helper::action(
                                icons::play_icon(),
                                "Play",
                                Some(Event::PlayTrack(
                                    track.get("video_id").unwrap().to_string(),
                                    track.get("display_name").unwrap().to_string(),
                                    track
                                        .get("duration")
                                        .unwrap()
                                        .parse::<i32>()
                                        .unwrap()
                                        .try_into()
                                        .unwrap(),
                                    None,
                                    Some(self.tracks.clone())
                                ))
                            ),
                            text(track.get("display_name").unwrap()),
                            horizontal_space(),
                            text(format::duration(
                                track
                                    .get("duration")
                                    .unwrap()
                                    .parse::<i32>()
                                    .unwrap()
                                    .try_into()
                                    .unwrap()
                            )),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    );
                }
            }

            let content = container(scrollable(
                col.spacing(10)
                    .align_x(Alignment::Start)
                    .width(Length::Fill),
            ))
            .padding(10);

            return content.into();
        }

        if self.create_playlist_mode {
            return container(
                column![
                    text("Create new playlist").size(22),
                    row![
                        text("Name:"),
                        text_input("...", &self.playlist_name_input)
                            .on_input(Event::PlaylistNameInput)
                    ]
                    .spacing(10)
                    .width(400)
                    .align_y(Alignment::Center),
                    button("Create new playlist").on_press(Event::CreatePlaylist),
                ]
                .spacing(15)
                .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        }

        let mut col = column![];

        // amend for loop to get an index
        for (index, playlist) in self.playlists.iter().enumerate() {
            col = col.push(
                button(text(playlist.get("name").unwrap()))
                    .on_press(Event::OpenPlaylist(index as i32))
                    .style(style::sidebar_button),
            );
        }

        let content = container(
            column![
                text("Your Playlists").size(22),
                scrollable(col.spacing(25).align_x(Alignment::Center))
            ]
            .align_x(Alignment::Center)
            .spacing(10),
        )
        .padding(10)
        .center_x(Length::Fill);

        content.into()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            create_playlist_mode: false,
            playlist_view: false,
            playlist_name_input: String::new(),
            playlists: db::get_all_playlists(),
            tracks: Vec::new(),
            thumbnails_recieved: false,
            thumbnails: Vec::new(),
        }
    }
}
