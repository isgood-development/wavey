mod components;
mod helpers;
mod pages;

use std::sync::mpsc;

use crate::core::playback;
use crate::core::rpc;
use crate::state;
use components::control_bar;
use components::sidebar;
use components::toast;
use pages::add_music;
use pages::ffmpeg;
use pages::playlist;
use pages::settings;
use pages::track_list;

use iced::advanced::graphics::futures::event;
use iced::event::Event as IcedEvent;
use iced::keyboard;
use iced::keyboard::key;
use iced::widget;
use iced::widget::{column, row};
use iced::{Subscription, Task, Theme};

pub struct Pages {
    pub current_page: Page,
    pub app_settings: Option<state::AppSettings>,

    nav: components::nav::State,
    sidebar: components::sidebar::State,
    controls: components::control_bar::State,

    track_list: track_list::State,
    settings: settings::State,
    add_music: add_music::State,
    ffmpeg: ffmpeg::State,
    playlist: playlist::State,

    playback_sender: mpsc::Sender<playback::AudioEvent>,
    rpc_sender: Option<mpsc::Sender<rpc::RpcEvent>>,

    toasts: Vec<toast::Toast>,
    theme: Theme,
    track_list_loaded: bool,
    rpc_enabled: bool,
}

#[derive(Default)]
pub enum Page {
    #[default]
    TrackList,
    Settings,
    AddMusic,
    FFmpeg,
    Playlist,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    NavAction(components::nav::Event),
    SidebarAction(components::sidebar::Event),
    ControlsAction(components::control_bar::Event),

    TrackListAction(track_list::Event),
    SettingsAction(settings::Event),
    AddMusicAction(add_music::Event),
    FFmpegAction(ffmpeg::Event),
    PlaylistAction(playlist::Event),

    CloseToast(usize),
    KeyboardEvent(IcedEvent),

    SettingsLoaded(state::AppSettings),
}

impl Pages {
    pub fn new() -> Self {
        let (playback_sender, playback_reciever) = mpsc::channel();

        playback::start_receiver(playback_reciever);

        Self {
            current_page: Page::TrackList,
            app_settings: None,

            nav: Default::default(),
            sidebar: Default::default(),
            controls: Default::default(),

            track_list: Default::default(),
            add_music: Default::default(),
            settings: Default::default(),
            ffmpeg: Default::default(),
            playlist: Default::default(),

            playback_sender,
            rpc_sender: None,

            toasts: vec![],
            theme: Theme::Dark,
            track_list_loaded: false,
            rpc_enabled: false,
        }
    }

    pub fn update(&mut self, message: UiEvent) -> Task<UiEvent> {
        match message {
            UiEvent::SettingsLoaded(settings) => {
                self.app_settings = Some(settings.clone());

                self.theme = helpers::theme::get_theme_from_settings(&settings.theme);
                self.rpc_enabled = settings.rpc_enabled;

                if settings.ffmpeg_path.is_empty() {
                    self.current_page = Page::FFmpeg;
                } else {
                    self.current_page = Page::TrackList;
                }

                if self.rpc_enabled {
                    let (rpc_sender, rpc_receiver) = mpsc::channel();

                    rpc::start_receiver(rpc_receiver);

                    self.rpc_sender = Some(rpc_sender);
                }

                Task::none()
            }

            UiEvent::KeyboardEvent(event) => match event {
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
                    key: keyboard::Key::Named(key::Named::Space),
                    ..
                }) => {
                    self.playback_sender
                        .send(playback::AudioEvent::PauseToggle)
                        .expect("Failed to send pause command");

                    self.controls
                        .update(components::control_bar::Event::PauseToggleAction)
                        .map(UiEvent::ControlsAction)
                }
                _ => Task::none(),
            },
            UiEvent::NavAction(event) => {
                match event {
                    components::nav::Event::CollapseSidebar => {
                        return Task::batch(vec![
                            self.sidebar
                                .update(sidebar::Event::CollapseToggle)
                                .map(UiEvent::SidebarAction),
                            self.nav.update(event.clone()).map(UiEvent::NavAction),
                        ]);
                    }
                    _ => (),
                }

                self.nav.update(event.clone()).map(UiEvent::NavAction)
            }

            UiEvent::PlaylistAction(event) => {
                let playlist_command = self
                    .playlist
                    .update(event.clone())
                    .map(UiEvent::PlaylistAction);

                match event {
                    playlist::Event::CreatePlaylist => Task::batch(vec![
                        playlist_command,
                        self.sidebar
                            .update(sidebar::Event::UpdatePlaylists)
                            .map(UiEvent::SidebarAction),
                    ]),
                    playlist::Event::PlayTrack(
                        video_id,
                        display_name,
                        duration,
                        handle,
                        tracks,
                    ) => {
                        self.playback_sender
                            .send(playback::AudioEvent::Queue(
                                video_id.clone().to_string(),
                                tracks.clone(),
                            ))
                            .expect("Failed to send play command");

                        if self.rpc_enabled {
                            self.rpc_sender
                                .as_ref()
                                .unwrap()
                                .send(rpc::RpcEvent::Set(
                                    display_name.clone(),
                                    duration.to_string(),
                                ))
                                .expect("Failed to send rpc command");
                        }

                        Task::batch(vec![
                            self.controls
                                .update(components::control_bar::Event::InitiatePlay(
                                    self.controls.active_video_id.clone(),
                                    display_name.to_string(),
                                    duration,
                                    handle.clone(),
                                    self.controls.tracks.clone(),
                                ))
                                .map(UiEvent::ControlsAction),
                            playlist_command,
                        ])
                    }
                    _ => playlist_command,
                }
            }
            UiEvent::CloseToast(index) => {
                self.toasts.remove(index);

                Task::none()
            }

            UiEvent::FFmpegAction(event) => {
                match event {
                    ffmpeg::Event::Continue => self.current_page = Page::TrackList,
                    _ => (),
                };

                self.ffmpeg.update(event).map(UiEvent::FFmpegAction)
            }

            UiEvent::AddMusicAction(event) => {
                let download_command = self
                    .add_music
                    .update(event.clone())
                    .map(UiEvent::AddMusicAction);
                match event {
                    add_music::Event::SearchQueryReceived(data) => {
                        match data {
                            Ok(data) => data,
                            Err(error) => {
                                log::error!("Failed to get search results: {:?}", error);

                                self.toasts.push(toast::Toast {
                                    title: "Search failed".into(),
                                    body: format!("Failed to get search results: {:?}", error),
                                    status: toast::Status::Danger,
                                });

                                return download_command;
                            }
                        };

                        download_command
                    }
                    add_music::Event::DownloadPressed(video_id) => {
                        self.toasts.push(toast::Toast {
                            title: "Download Started".into(),
                            body: format!("Downloading video: {}", video_id),
                            status: toast::Status::Primary,
                        });

                        download_command
                    }
                    add_music::Event::DownloadComplete(status) => {
                        match status {
                            Ok(_) => {
                                self.toasts.push(toast::Toast {
                                    title: "Download Complete".into(),
                                    body: "Downloaded video successfully".into(),
                                    status: toast::Status::Success,
                                });

                                return Task::batch(vec![
                                    self.track_list
                                        .update(track_list::Event::GetThumbnailHandles)
                                        .map(UiEvent::TrackListAction),
                                    download_command,
                                ]);
                            }
                            Err(error) => {
                                log::error!("Failed to download video: {:?}", error);

                                self.toasts.push(toast::Toast {
                                    title: "Download Failed".into(),
                                    body: format!("Failed to download video: {:?}", error),
                                    status: toast::Status::Danger,
                                });
                            }
                        };

                        download_command
                    }
                    add_music::Event::UrlResult(status) => {
                        match status {
                            Ok(_) => {
                                self.toasts.push(toast::Toast {
                                    title: "Download Complete".into(),
                                    body: "Downloaded video successfully".into(),
                                    status: toast::Status::Success,
                                });
                            }
                            Err(error) => {
                                log::error!("Failed to download video: {:?}", error);

                                self.toasts.push(toast::Toast {
                                    title: "Download Failed".into(),
                                    body: format!("Failed to download video: {:?}", error),
                                    status: toast::Status::Danger,
                                });
                            }
                        };

                        download_command
                    }
                    _ => download_command,
                }
            }
            UiEvent::SettingsAction(event) => {
                if !self.settings.is_loaded {
                    return Task::batch(vec![
                        self.settings
                            .update(settings::Event::LoadSettings)
                            .map(UiEvent::SettingsAction),
                        self.settings
                            .update(event.clone())
                            .map(UiEvent::SettingsAction),
                    ]);
                }
                match event {
                    settings::Event::ThemeSelected(theme) => {
                        self.theme = helpers::theme::match_theme(Some(theme));
                    }
                    settings::Event::ToggleRpcEnabled => {
                        if self.rpc_enabled {
                            self.rpc_sender
                                .as_ref()
                                .unwrap()
                                .send(rpc::RpcEvent::Hide)
                                .expect("Failed to send rpc command");
                        }

                        self.rpc_enabled = !self.rpc_enabled
                    }
                    _ => (),
                }
                self.settings.update(event).map(UiEvent::SettingsAction)
            }
            UiEvent::TrackListAction(ref event) => {
                let track_list_command: Task<UiEvent>;

                if !self.track_list_loaded {
                    track_list_command = Task::batch(vec![
                        self.track_list
                            .update(track_list::Event::GetThumbnailHandles)
                            .map(UiEvent::TrackListAction),
                        self.track_list
                            .update(event.clone())
                            .map(UiEvent::TrackListAction),
                    ]);
                    self.track_list_loaded = true;
                } else {
                    track_list_command = self
                        .track_list
                        .update(event.clone())
                        .map(UiEvent::TrackListAction);
                }
                match event {
                    track_list::Event::PlayTrack(
                        video_id,
                        display_name,
                        duration,
                        handle,
                        tracks,
                    ) => {
                        self.playback_sender
                            .send(playback::AudioEvent::Queue(
                                video_id.clone().to_string(),
                                tracks.clone(),
                            ))
                            .expect("Failed to send play command");

                        if self.rpc_enabled {
                            self.rpc_sender
                                .as_ref()
                                .unwrap()
                                .send(rpc::RpcEvent::Set(
                                    display_name.clone(),
                                    duration.to_string(),
                                ))
                                .expect("Failed to send rpc command");
                        }

                        Task::batch(vec![
                            self.controls
                                .update(components::control_bar::Event::InitiatePlay(
                                    self.controls.active_video_id.clone(),
                                    display_name.to_string(),
                                    *duration,
                                    handle.clone(),
                                    self.controls.tracks.clone(),
                                ))
                                .map(UiEvent::ControlsAction),
                            track_list_command,
                        ])
                    }
                    track_list::Event::Submit => return Task::batch(vec![
                        self.track_list
                            .update(track_list::Event::GetThumbnailHandles)
                            .map(UiEvent::TrackListAction),
                        track_list_command,
                    ]),
                    _ => track_list_command,
                }
            }

            UiEvent::SidebarAction(event) => {
                let sidebar_command = self
                    .sidebar
                    .update(event.clone())
                    .map(UiEvent::SidebarAction);

                match event {
                    components::sidebar::Event::OpenDownload => self.current_page = Page::AddMusic,
                    components::sidebar::Event::OpenPlaylists => {
                        return {
                            self.current_page = Page::Playlist;
                            self.playlist
                                .update(playlist::Event::OpenInListMode)
                                .map(UiEvent::PlaylistAction)
                        }
                    }
                    components::sidebar::Event::OpenSettings => {
                        return {
                            self.current_page = Page::Settings;
                            self.settings
                                .update(settings::Event::LoadSettings)
                                .map(UiEvent::SettingsAction)
                        }
                    }
                    components::sidebar::Event::OpenTrackList => {
                        self.current_page = Page::TrackList
                    }
                    components::sidebar::Event::CreatePlaylist => {
                        return {
                            self.current_page = Page::Playlist;
                            self.playlist
                                .update(playlist::Event::OpenInCreateMode)
                                .map(UiEvent::PlaylistAction)
                        }
                    }
                    components::sidebar::Event::OpenPlaylist(index) => {
                        return {
                            self.current_page = Page::Playlist;
                            self.playlist
                                .update(playlist::Event::OpenPlaylist(index))
                                .map(UiEvent::PlaylistAction)
                        }
                    }
                    _ => (),
                }

                sidebar_command
            }

            UiEvent::ControlsAction(event) => {
                let controls_command = self
                    .controls
                    .update(event.clone())
                    .map(UiEvent::ControlsAction);

                match event {
                    components::control_bar::Event::ProgressChanged(value) => {
                        self.playback_sender
                            .send(playback::AudioEvent::SeekTo(value as u64))
                            .expect("Failed to send seek command");

                        controls_command
                    }
                    components::control_bar::Event::PauseToggleAction => {
                        self.playback_sender
                            .send(playback::AudioEvent::PauseToggle)
                            .expect("Failed to send pause command");

                        controls_command
                    }
                    components::control_bar::Event::VolumeChanged(value) => {
                        self.playback_sender
                            .send(playback::AudioEvent::SetVolume(value))
                            .expect("Failed to send volume command");

                        controls_command
                    }
                    components::control_bar::Event::Mute => {
                        self.playback_sender
                            .send(playback::AudioEvent::Mute)
                            .expect("Failed to send mute command");

                        controls_command
                    }
                    components::control_bar::Event::Unmute => {
                        self.playback_sender
                            .send(playback::AudioEvent::Unmute)
                            .expect("Failed to send unmute command");

                        controls_command
                    }
                    components::control_bar::Event::BackwardPressed => {
                        self.playback_sender
                            .send(playback::AudioEvent::Backward)
                            .expect("Failed to send backward command");

                        if self.controls.tracks.is_empty() {
                            return controls_command;
                        }

                        self.controls
                            .update(control_bar::Event::SeekTo(0.0))
                            .map(UiEvent::ControlsAction)
                    }
                    components::control_bar::Event::ForwardPressed => {
                        self.playback_sender
                            .send(playback::AudioEvent::Forward)
                            .expect("Failed to send forward command");

                        if self.controls.tracks.is_empty() {
                            return controls_command;
                        }

                        let index = self
                            .controls
                            .tracks
                            .iter()
                            .position(|x| {
                                x.get("video_id").unwrap() == &self.controls.active_video_id
                            })
                            .unwrap();
                        let next_index = index + 1;
                        let next_track = &self.controls.tracks[next_index];

                        let video_id = next_track.get("video_id").unwrap();
                        let display_name = next_track.get("display_name").unwrap();
                        let duration = next_track.get("duration").unwrap().parse::<u64>().unwrap();

                        self.controls
                            .update(control_bar::Event::InitiatePlay(
                                video_id.to_string(),
                                display_name.to_string(),
                                duration,
                                None,
                                self.controls.tracks.clone(),
                            ))
                            .map(UiEvent::ControlsAction)
                    }
                    components::control_bar::Event::Tick => {
                        if self.rpc_enabled {
                            if !self.controls.is_paused {
                                self.rpc_sender
                                    .as_ref()
                                    .unwrap()
                                    .send(rpc::RpcEvent::SetProgress(
                                        self.controls.display_name.clone(),
                                        self.controls.seconds_passed.to_string(),
                                        self.controls.total_duration.to_string().clone(),
                                    ))
                                    .expect("Failed to send tick command");
                            }
                        }
                        controls_command
                    }
                    _ => controls_command,
                }
            }
        }
    }

    pub fn view(&self) -> iced::Element<UiEvent> {
        match &self.current_page {
            Page::Playlist => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.playlist.view().map(UiEvent::PlaylistAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::FFmpeg => {
                let content = self.ffmpeg.view().map(UiEvent::FFmpegAction);

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::TrackList => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.track_list.view().map(UiEvent::TrackListAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::AddMusic => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.add_music.view().map(UiEvent::AddMusicAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }

            Page::Settings => {
                let content = column![
                    self.nav.view().map(UiEvent::NavAction),
                    row![
                        self.sidebar.view().map(UiEvent::SidebarAction),
                        self.settings.view().map(UiEvent::SettingsAction),
                    ],
                    self.controls.view().map(UiEvent::ControlsAction),
                ];

                toast::Manager::new(content, &self.toasts, UiEvent::CloseToast).into()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<UiEvent> {
        Subscription::batch(vec![
            event::listen().map(UiEvent::KeyboardEvent),
            self.track_list.subscription().map(UiEvent::TrackListAction),
            self.controls.subscription().map(UiEvent::ControlsAction),
            self.ffmpeg.subscription().map(UiEvent::FFmpegAction),
        ])
    }

    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}

impl Default for Pages {
    fn default() -> Self {
        Self::new()
    }
}
