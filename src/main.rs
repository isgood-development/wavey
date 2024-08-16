use core::db;
use core::json;

use iced::{window, Element, Font, Settings, Subscription, Task};

use image::GenericImageView;
use log;
use log4rs;

mod core;
mod state;
mod ui;

pub fn main() -> iced::Result {
    // Setting the app icon.
    static ICON: &[u8] = include_bytes!("../assets/main.ico");

    let image = image::load_from_memory(ICON).unwrap();
    let (width, height) = image.dimensions();
    let rgba = image.into_rgba8();
    let icon = window::icon::from_rgba(rgba.into_raw(), width, height).unwrap();

    let app_settings = Settings {
        default_font: Font::with_name("Nunito"),
        ..Default::default()
    };

    let win_settings = window::Settings {
        icon: Some(icon),
        ..Default::default()
    };

    iced::application("wavey", Wavey::update, Wavey::view)
        .settings(app_settings)
        .subscription(Wavey::subscription)
        .window(win_settings)
        .window_size((1000.0, 730.0))
        .font(include_bytes!("../assets/icons.ttf").as_slice())
        .font(include_bytes!("../assets/font.ttf").as_slice())
        .theme(Wavey::theme)
        .run_with(Wavey::new)
}

#[derive(Debug, Clone)]
enum Message {
    Pages(ui::UiEvent),
    SettingsLoaded(Option<state::AppSettings>),
}

struct Wavey {
    pages: ui::Pages,
}

impl Wavey {
    fn new() -> (Self, Task<Message>) {
        log4rs::init_file("logging_config.yaml", Default::default()).unwrap();

        log::info!("Starting Wavey.");
        // Creates the database if it doesn't exist.
        if !db::check_database_exists() {
            log::info!("Creating database because it does not exist.");
            let _ = db::create_database_tables();
        }

        // Verifies validity of the data in the database.
        let _ = db::verify_data_integrity();

        if !json::check_exists() {
            log::info!("Creating settings file because it does not exist.");
            let _ = json::create_file();
        }

        (
            Self {
                pages: Default::default(),
            },
            Task::perform(json::load_settings(), Message::SettingsLoaded),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Pages(x) => self.pages.update(x).map(Message::Pages),
            Message::SettingsLoaded(settings) => {
                if let Some(settings) = settings {
                    self.pages
                        .update(ui::UiEvent::SettingsLoaded(settings))
                        .map(Message::Pages);
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        self.pages.view().map(Message::Pages).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        self.pages.subscription().map(Message::Pages)
    }

    fn theme(&self) -> iced::Theme {
        self.pages.theme()
    }
}
