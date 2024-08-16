use std::path::PathBuf;

use crate::core::file;
use crate::core::json;
use crate::core::request;

use iced::widget::{button, column, container, progress_bar, row, text};
use iced::{Alignment, Length, Subscription, Task};

pub struct State {
    id: usize,
    state: DownloadState,
    install_pressed: bool,
    in_progress: bool,
}

#[derive(Debug)]
enum DownloadState {
    Idle,
    Downloading { progress: f32 },
    Finished,
    Errored,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    InstallFFmpeg,
    ManuallySpecify,
    Continue,
    DownloadProgressed((usize, request::Progress)),
    PathSpecified(Result<PathBuf, file::FileError>),
}

impl State {
    pub fn new() -> Self {
        Self {
            id: 0,
            state: DownloadState::Idle,
            install_pressed: false,
            in_progress: false,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            DownloadState::Idle { .. }
            | DownloadState::Finished { .. }
            | DownloadState::Errored { .. } => {
                self.state = DownloadState::Downloading { progress: 0.0 };
            }
            DownloadState::Downloading { .. } => {}
        }
    }

    pub fn progress(&mut self, new_progress: request::Progress) {
        if let DownloadState::Downloading { progress } = &mut self.state {
            match new_progress {
                request::Progress::Started => {
                    *progress = 0.0;
                }
                request::Progress::Advanced(percentage) => {
                    *progress = percentage;
                }
                request::Progress::Finished => {
                    self.state = DownloadState::Finished;
                }
                request::Progress::Errored => {
                    self.state = DownloadState::Errored;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Event> {
        match self.state {
            DownloadState::Downloading { .. } => request::download_file(
                self.id,
                "https://github.com/isgood-dev/wavey/releases/download/ffmpeg/ffmpeg.exe",
            )
            .map(Event::DownloadProgressed),
            _ => Subscription::none(),
        }
    }

    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::Continue => Task::none(),
            Event::DownloadProgressed((_id, progress)) => {
                self.progress(progress);

                self.in_progress = true;

                Task::none()
            }

            Event::InstallFFmpeg => {
                self.start();
                self.install_pressed = true;

                Task::perform(
                    json::save_settings(move |settings| {
                        settings.ffmpeg_path = "ffmpeg.exe".to_string();
                    }),
                    |_| Event::Continue,
                )
            }
            Event::ManuallySpecify => Task::perform(file::pick_file(), Event::PathSpecified),
            Event::PathSpecified(Ok(path)) => {
                let path_clone = path.clone();

                Task::perform(
                    json::save_settings(move |settings| {
                        settings.ffmpeg_path = path_clone
                            .to_str()
                            .expect("Path is not valid Unicode")
                            .to_string();
                    }),
                    |_| Event::Continue,
                )
            }
            Event::PathSpecified(Err(e)) => {
                match e {
                    file::FileError::DialogClosed => {
                        // Do nothing
                    }
                };

                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
        let progress = match &self.state {
            DownloadState::Idle { .. } => 0.0,
            DownloadState::Downloading { progress } => *progress,
            DownloadState::Finished { .. } => 100.0,
            DownloadState::Errored { .. } => 0.0,
        };

        if self.in_progress {
            if progress == 100.0 {
                return container(
                    column![
                        text("Success!").size(20),
                        text("You're ready to go."),
                        button("Proceed to wavey").on_press(Event::Continue)
                    ]
                    .align_x(Alignment::Center)
                    .spacing(10),
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .padding(10)
                .into();
            }
            let progress_bar = progress_bar(0.0..=100.0, progress);

            let content = container(
                column![
                    text("Downloading FFmpeg").size(20),
                    text("Please wait while FFmpeg is being downloaded."),
                    progress_bar,
                    text(format!("Progress: {:.0}%", progress)),
                ]
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(10);

            return content.into();
        } else {
            let content = container(
                column![
                    text("FFmpeg not installed").size(20),
                    text("FFmpeg was not found in your system path. It is required for important operations of wavey."),
                    text("Would you like me to install it, or specify the path to the FFmpeg executable?"),
                    row![
                        button("Install FFmpeg").on_press(Event::InstallFFmpeg),
                        button("Manually locate").on_press(Event::ManuallySpecify).style(button::secondary)
                    ].spacing(10),
                ]
                .spacing(10)
                .align_x(Alignment::Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(10);

            content.into()
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
