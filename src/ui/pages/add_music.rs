use std::collections::HashMap;

use super::super::helpers::helper;
use super::super::helpers::icons;
use crate::core::request;
use crate::core::youtube;

use iced::widget::{column, container, image as image_widget, row, scrollable, text, text_input};
use iced::Alignment;
use iced::Task;

pub struct State {
    query: String,
    results: Option<Vec<HashMap<String, String>>>,
    thumbnails: Vec<iced::advanced::image::Handle>,
    loading: bool,
    url_requested: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    SearchQuery(String),
    Submit,
    DownloadPressed(String),

    ThumbnailReceived(Result<Vec<Vec<u8>>, request::RequestError>),
    DownloadComplete(Result<(), youtube::StatusError>),
    SearchQueryReceived(Result<Vec<HashMap<String, String>>, youtube::StatusError>),
    UrlResult(Result<(), youtube::StatusError>),
}

impl State {
    pub fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::ThumbnailReceived(response) => {
                match response {
                    Ok(data) => {
                        for thumbnail in data {
                            let handle = iced::advanced::image::Handle::from_bytes(thumbnail);
                            self.thumbnails.push(handle);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get thumbnails: {:?}", e);
                    }
                };

                self.loading = false;

                Task::none()
            }

            Event::SearchQueryReceived(Ok(data)) => {
                self.results = Some(data.clone());

                Task::perform(
                    request::request_all_thumbnails(data),
                    Event::ThumbnailReceived,
                )
            }
            Event::SearchQueryReceived(Err(e)) => {
                log::error!("Failed to get search results: {:?}", e);

                Task::none()
            }
            Event::UrlResult(Ok(_)) => {
                self.url_requested = false;
                self.loading = false;
                self.results = None;
                self.thumbnails = Vec::new();

                Task::none()
            }
            Event::UrlResult(Err(e)) => {
                log::error!("Failed to download from URL: {:?}", e);

                Task::none()
            }

            Event::Submit => {
                self.loading = true;

                self.url_requested = false;
                self.results = None;
                self.thumbnails = Vec::new();

                let re = regex::Regex::new(r"(https?://)?(www\.)?(youtube\.com|youtu\.?be)/.+$")
                    .unwrap();

                if re.is_match(&self.query) {
                    self.url_requested = true;

                    let yt_url = self.query.clone();

                    Task::perform(youtube::download_from_url(yt_url), Event::UrlResult)
                } else {
                    Task::perform(
                        youtube::get_search_results(self.query.clone()),
                        Event::SearchQueryReceived,
                    )
                }
            }

            Event::DownloadPressed(video_id) => Task::perform(
                youtube::download_from_url(video_id),
                Event::DownloadComplete,
            ),

            Event::DownloadComplete(status) => {
                match status {
                    Ok(_) => {
                        log::info!("Downloaded successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to download: {:?}", e);
                    }
                }

                Task::none()
            }
            Event::SearchQuery(query) => {
                self.query = query;

                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<Event> {
        let mut col = column![].spacing(10).padding(10).push(row![
            text_input("Enter YouTube URL or search query", &self.query)
                .on_input(Event::SearchQuery),
            helper::action(icons::search_icon(), "Search", Some(Event::Submit)),
        ]);

        if self.loading {
            col = col.push(text("Loading..."));

            return container(col).into();
        }

        if self.results.is_some() && !self.loading {
            if let Some(results) = &self.results {
                for (index, result) in results.iter().enumerate() {
                    let title = result.get("title").unwrap();
                    let channel = result.get("channel").unwrap();
                    let heading = format!("{} - {}", title, channel);

                    let row = row![
                        helper::action(
                            icons::download_icon(),
                            "Download",
                            Some(Event::DownloadPressed(
                                result.get("video_id").unwrap().to_string()
                            ))
                        ),
                        image_widget(self.thumbnails[index].clone())
                            .width(130)
                            .height(100),
                        text(heading).size(16),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10);

                    col = col.push(row);
                }
            } else {
                log::error!("Search query returned nothing.");
                col = col.push(text(
                    "Sorry, nothing was found. Try altering the search query.",
                ))
            }
        } else {
            col = col.push(text("Results will appear here."))
        }

        container(scrollable(col)).into()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            query: String::new(),
            results: None,
            loading: false,
            url_requested: false,
            thumbnails: Vec::new(),
        }
    }
}
