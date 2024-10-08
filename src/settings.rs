use audiocloud_lib::Sample;
use core::fmt;
use iced::Theme;
use serde_derive::*;
use std::fs;
use std::path::Path;

use crate::helpers::{self, hash_sample};
use crate::AudioCloud;
use crate::Message;
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, slider,
    text, text_input, toggler,
};
use iced::Task;
use iced::{alignment, Alignment, Element, Length, Padding};

use crate::bootstrap::*;
use crate::settings;
use crate::themes;
use crate::ICON_FONT;

#[derive(Debug, Clone)]
pub enum SettingsChanged {
    ShowGradient(bool),
    TitleSetting(SearchViewTitle),
}

pub struct SettingsState {
    title_mode_state: combo_box::State<SearchViewTitle>,
}
impl SettingsState {
    pub fn new() -> Self {
        SettingsState {
            title_mode_state: combo_box::State::new(SearchViewTitle::all()),
        }
    }
}

pub fn settings_changed(app: &mut AudioCloud, message: SettingsChanged) -> Task<Message> {
    match message {
        SettingsChanged::ShowGradient(val) => {
            app.settings.searchbar_gradient = val;
        }
        SettingsChanged::TitleSetting(set) => {
            app.settings.searchview_title = Some(set);
        }
    }
    Task::none()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SearchViewTitle {
    Title,
    Spacing,
    Nothing,
}
impl SearchViewTitle {
    fn all() -> Vec<Self> {
        vec![
            SearchViewTitle::Title,
            SearchViewTitle::Spacing,
            SearchViewTitle::Nothing,
        ]
    }
}
impl std::fmt::Display for SearchViewTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchViewTitle::Title => write!(f, "Title"),
            SearchViewTitle::Nothing => write!(f, "None"),
            SearchViewTitle::Spacing => write!(f, "Space"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub theme: String,
    pub searchbar_gradient: bool,
    pub searchview_title: Option<SearchViewTitle>,
    pub server_url: String,
    pub max_results: i32,
    pub favourite_samples: Vec<Sample>,
    pub dl_samples_hash: Vec<String>,
}
pub async fn load_from_file(path: &str) -> Settings {
    if !Path::new(path).exists() {
        return Settings::default();
    }
    let filecontent = fs::read_to_string(path).expect("Couldn't read file");
    let settings: Settings = serde_json::from_str(&filecontent).expect("Couldnt parse file");
    settings
}

pub async fn save_to_file(settings: Settings, path: &str) {
    let content = serde_json::to_string_pretty(&settings).unwrap();
    let _ = fs::write(path, content);
}

impl Settings {
    pub fn is_favourite(&self, sample: &Sample) -> bool {
        for s in &self.favourite_samples {
            if s.path == sample.path {
                return true;
            }
        }
        false
    }
    pub fn add_favourite(&mut self, sample: Sample) {
        self.favourite_samples.push(sample);
    }
    pub fn rem_favourite(&mut self, sample_id: &str) {
        for i in 0..self.favourite_samples.len() {
            if self.favourite_samples[i].path.eq(sample_id) {
                self.favourite_samples.remove(i);
                break;
            }
        }
    }
    pub fn is_downloaded(&self, path: &str) -> bool {
        for entry in &self.dl_samples_hash {
            if entry == &hash_sample(&path.replace(".wav", "")) {
                return true;
            }
        }
        false
    }
    pub fn add_dl_entry(&mut self, path: &str) {
        self.dl_samples_hash.push(helpers::hash_sample(path));
        println!("added dl entry: {}", helpers::hash_sample(path));
    }
    pub fn default() -> Self {
        Settings {
            searchbar_gradient: false,
            searchview_title: Some(SearchViewTitle::Spacing),
            max_results: 50,
            server_url: "http://127.0.0.1:4040/".to_string(),
            theme: "Dark".to_string(),
            favourite_samples: vec![],
            dl_samples_hash: vec![],
        }
    }
}

pub fn settings(app: &AudioCloud) -> Element<Message> {
    let status_text = text(&app.status_message).style(themes::text_fg);
    let settings_button = button(text(icon_to_string(Bootstrap::XLg)).font(ICON_FONT))
        .on_press(Message::SettingsButtonToggled)
        .padding([5, 10]);
    let status_bar = row![horizontal_space(), status_text, settings_button]
        .spacing(10)
        .align_y(Alignment::Center);

    let title = text("Settings")
        .width(Length::Fill)
        .size(35)
        .align_x(alignment::Horizontal::Center);

    let connection_status = match app.server_status {
        Some(status) => match status {
            true => text("Server connected").style(|theme: &Theme| text::Style {
                color: Some(theme.palette().success),
            }),
            false => text("Server unreachable").style(|theme: &Theme| text::Style {
                color: Some(theme.palette().danger),
            }),
        },
        None => text("Unknown status"),
    };
    let settings = column![
        row![
            text("Server URL:"),
            container(
                text_input("http://127.0.0.1:4040/", &app.settings.server_url)
                    .on_input(Message::ServerUrlSubmited)
            )
            .padding({
                Padding {
                    bottom: 0.0,
                    top: 0.0,
                    left: 5.0,
                    right: 5.0,
                }
            }),
            connection_status,
        ]
        .align_y(Alignment::Center)
        .padding(20)
        .spacing(15),
        row![
            text("Theme:"),
            combo_box(
                &app.theme_state,
                "No theme selected",
                app.selected_theme.as_ref(),
                Message::ThemeSelected
            ),
            toggler(
                String::from("Searchbar gradient"),
                app.settings.searchbar_gradient,
                |val: bool| { Message::Settings(settings::SettingsChanged::ShowGradient(val)) }
            ),
            combo_box(
                &app.settings_state.title_mode_state,
                "None",
                app.settings.searchview_title.as_ref(),
                |val| Message::Settings(SettingsChanged::TitleSetting(val)),
            )
        ]
        .align_y(Alignment::Center)
        .padding(20)
        .spacing(15),
        row![
            text("Max search results:"),
            text(app.settings.max_results),
            slider(
                std::ops::RangeInclusive::new(1, 100),
                app.settings.max_results,
                Message::MaxRequestsChanged
            )
        ]
        .align_y(Alignment::Center)
        .spacing(15)
        .padding(20),
        row![
            button(text("Save settings"))
                .on_press(Message::SaveSettings)
                .style(button::success),
            button(text("Reload settings")).on_press(Message::LoadSettings),
            button(text("Reset settings"))
                .style(button::danger)
                .on_press(Message::ResetSettings),
            button(text("Reset downloads"))
                .style(button::danger)
                .on_press(Message::ResetCache)
        ]
        .spacing(15)
        .align_y(Alignment::Center)
        .padding(20),
    ];

    column![status_bar, title, settings].spacing(20).into()
}
