use audiocloud_lib::*;
use iced::event::{self, Event};
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, text,
    text_input,
};
use iced::window;
use iced::{
    alignment, Alignment, Command, Element, Executor, Font, Length, Padding, Subscription, Theme,
};
use iced_aw::graphics::icons::{bootstrap::icon_to_string, BootstrapIcon, BOOTSTRAP_FONT_BYTES};
use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

pub mod audio;
pub mod helpers;
pub mod request;
pub mod settings;
pub mod themes;
pub mod views;

#[tokio::main]
async fn main() -> iced::Result {
    iced::program(AudioCloud::title, AudioCloud::update, AudioCloud::view)
        .theme(AudioCloud::theme)
        .font(BOOTSTRAP_FONT_BYTES)
        .subscription(AudioCloud::subscription)
        .load(|| {
            Command::perform(
                settings::load_from_file("settings.json"),
                Message::SettingsLoaded,
            )
        })
        .exit_on_close_request(false)
        .run()
}

pub struct AudioCloud {
    input: String,
    view: ViewControl,
    results: Option<SearchResult>,
    server_url: String,
    server_status: Option<bool>,
    audio_devices: Option<audio::Handlers>,
    theme_state: combo_box::State<Theme>,
    selected_theme: Option<Theme>,

    show_oneshots: bool,
    show_loops: bool,

    settings: settings::Settings,
    status_message: String,
}

enum ViewControl {
    Main,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    Exit(()),

    InputChanged(String),
    CreateTask,
    SettingsButtonToggled,
    SearchResultRecived(SearchResult),
    ServerStatusUpdate(bool),
    ServerUrlSubmited(String),

    PlaySample(String),
    TempAudioLoaded(String),

    ThemeSelected(Theme),

    ShowOneshotsCheckbox(bool),
    ShowLoopsCheckbox(bool),

    LoadSettings,
    SaveSettings,
    SettingsLoaded(settings::Settings),
    SettingsSaved(()),

    ToggleFavourite(Sample),
}

const ICON_FONT: Font = Font::with_name("bootstrap-icons");

fn perform_search(params: SearchParams, path: String) -> Command<Message> {
    Command::perform(
        request::get_result(params, path),
        Message::SearchResultRecived,
    )
}
fn send_file_preview_dl(server_url: String, path: String) -> Command<Message> {
    Command::perform(
        request::get_temp_audio(server_url, path),
        Message::TempAudioLoaded,
    )
}

impl AudioCloud {
    fn create_request_command(&self, input: String) -> Command<Message> {
        if input.is_empty() || input.eq("-") {
            return Command::none();
        }

        let sample_type_filter = if self.show_loops == self.show_oneshots {
            None
        } else {
            if self.show_oneshots {
                Some(SampleType::OneShot)
            } else {
                Some(SampleType::Loop(0))
            }
        };

        let params = SearchParams {
            query: input.clone(),
            sample_type: sample_type_filter,
            max_tempo: None,
            min_tempo: None,
            pack_id: None,
            max_results: Some(self.settings.max_results),
        };
        return perform_search(params, self.server_url.clone());
    }
    fn new() -> (Self, Command<Message>) {
        (
            Self {
                input: String::from(""),
                view: ViewControl::Main,
                results: None,
                server_url: "http://127.0.0.1:4040/".to_string(),
                server_status: None,
                audio_devices: match audio::init_audio() {
                    Ok(handlers) => Some(handlers),
                    Err(_) => None,
                },
                theme_state: combo_box::State::new(Theme::ALL.to_vec()),
                selected_theme: None,
                show_oneshots: true,
                show_loops: true,

                settings: settings::Settings {
                    max_results: 50,
                    theme: "Dark".to_string(),
                    favourite_samples: vec![],
                },
                status_message: String::from("idle... "),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Audiocloud")
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) => {
                if let Event::Window(id, window::Event::CloseRequested) = event {
                    let mut set = self.settings.clone();
                    match &self.selected_theme {
                        Some(theme) => set.theme = theme.to_string(),
                        None => set.theme = "Dark".to_string(),
                    }

                    println!("sending saving instructions");
                    return Command::perform(
                        settings::save_to_file(set, "settings.json"),
                        Message::Exit,
                    );
                } else {
                    return Command::none();
                }
            }
            Message::Exit(_) => {
                println!("exiting");
                return window::close(window::Id::MAIN);
            }
            Message::InputChanged(val) => {
                self.input = val;
                return self.create_request_command(self.input.clone());
            }
            Message::CreateTask => {}
            Message::SettingsButtonToggled => match self.view {
                ViewControl::Settings => self.view = ViewControl::Main,
                ViewControl::Main => self.view = ViewControl::Settings,
            },
            Message::SearchResultRecived(val) => {
                if val.samples.len() > 0 {
                    self.results = Some(val)
                } else {
                    self.results = None
                }
            }
            Message::ServerUrlSubmited(url) => {
                self.server_url = url;
                return Command::perform(
                    request::check_connection(self.server_url.clone()),
                    Message::ServerStatusUpdate,
                );
            }
            Message::ServerStatusUpdate(status) => {
                self.server_status = Some(status);
            }

            Message::PlaySample(path) => {
                return send_file_preview_dl(self.server_url.clone(), path)
            }
            Message::TempAudioLoaded(path) => {
                let file = BufReader::new(File::open(path).expect("Couldnt open file"));
                let source = Decoder::new(file).expect("Couldnt build source from file");
                match &self.audio_devices {
                    Some(devs) => {
                        if !devs.sink.empty() {
                            devs.sink.clear();
                        }

                        devs.sink.append(source);
                        devs.sink.play();
                    }
                    None => println!("Error loading devices from option"),
                }
            }
            Message::ThemeSelected(theme) => {
                self.selected_theme = Some(theme);
            }
            Message::ShowOneshotsCheckbox(val) => {
                self.show_oneshots = val;
                return self.create_request_command(self.input.clone());
            }
            Message::ShowLoopsCheckbox(val) => {
                self.show_loops = val;
                return self.create_request_command(self.input.clone());
            }
            Message::LoadSettings => {
                return Command::perform(
                    settings::load_from_file("settings.json"),
                    Message::SettingsLoaded,
                )
            }
            Message::SettingsLoaded(val) => {
                self.settings = val;
                self.selected_theme = themes::string_to_theme(&self.settings.clone().theme);
                self.status_message = "Loaded settings ".to_string();
            }
            Message::SaveSettings => {
                let mut set = self.settings.clone();
                match &self.selected_theme {
                    Some(theme) => set.theme = theme.to_string(),
                    None => set.theme = "Dark".to_string(),
                }

                return Command::perform(
                    settings::save_to_file(set, "settings.json"),
                    Message::SettingsSaved,
                );
            }
            Message::SettingsSaved(_) => {
                self.status_message = String::from("Settings saved ");
            }
            Message::ToggleFavourite(sample) => {
                if self.settings.is_favourite(&sample) {
                    self.settings.rem_favourite(&sample.path);
                } else {
                    self.settings.add_favourite(sample);
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let status_text = text(&self.status_message).style(themes::text_fg);
        match self.view {
            ViewControl::Main => {
                let settings =
                    button(text(icon_to_string(BootstrapIcon::GearFill)).font(ICON_FONT))
                        .on_press(Message::SettingsButtonToggled)
                        .padding([5, 10, 5, 10]);
                let status_bar = container(
                    row![horizontal_space(), status_text, settings].align_items(Alignment::Center),
                );

                let title = text("Audiocloud")
                    .width(Length::Fill)
                    .size(35)
                    .horizontal_alignment(alignment::Horizontal::Center);

                let input_text = text_input("Some query...", &self.input)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::CreateTask)
                    .width(Length::FillPortion(6))
                    .size(30);
                let input = container(input_text).padding(Padding {
                    top: 5.0,
                    bottom: 0.0,
                    left: 50.0,
                    right: 50.0,
                });

                let sample_type_selector = row![
                    checkbox("OneShots ", self.show_oneshots)
                        .on_toggle(Message::ShowOneshotsCheckbox)
                        .size(22),
                    checkbox("Loops", self.show_loops)
                        .on_toggle(Message::ShowLoopsCheckbox)
                        .size(22)
                ];
                let filters =
                    container(column![text("Filters").size(25), sample_type_selector]).padding(10);

                let mut result_row = column![];
                match &self.results {
                    Some(val) => {
                        for sample in &val.samples {
                            let name = helpers::remove_brackets(&sample.name.replace(".wav", ""));

                            let type_text = match sample.sampletype {
                                SampleType::OneShot => {
                                    row![
                                        text(icon_to_string(BootstrapIcon::Soundwave))
                                            .font(ICON_FONT)
                                            .style(themes::text_fg),
                                        text(" OneShot").style(themes::text_fg),
                                    ]
                                }
                                SampleType::Loop(tempo) => {
                                    row![
                                        text(icon_to_string(BootstrapIcon::ArrowRepeat))
                                            .font(ICON_FONT)
                                            .style(themes::text_fg),
                                        text(" Loop ").style(themes::text_fg),
                                        text(tempo.to_string()).style(themes::text_fg).size(10),
                                        text("bpm").size(10).style(themes::text_fg),
                                    ]
                                }
                            };

                            let type_label =
                                container(type_text).align_y(alignment::Vertical::Center);

                            let edit_text = icon_to_string(BootstrapIcon::VinylFill);
                            let edit_button = button(text(edit_text).size(20).font(ICON_FONT))
                                .style(button::text);

                            let fav_text = match self.settings.is_favourite(sample) {
                                true => text(icon_to_string(BootstrapIcon::StarFill)).style(
                                    |theme: &Theme| text::Style {
                                        color: Some(theme.palette().success),
                                    },
                                ),
                                false => text(icon_to_string(BootstrapIcon::Star)),
                            };
                            let fav_button = button(fav_text.font(ICON_FONT).size(20))
                                .style(button::text)
                                .on_press(Message::ToggleFavourite(sample.clone()));

                            let sample_entry = container(
                                row![
                                    button(
                                        text(icon_to_string(BootstrapIcon::PlayFill))
                                            .font(ICON_FONT)
                                            .size(25)
                                    )
                                    .style(|theme, status| button::text(theme, status))
                                    .padding(20)
                                    .on_press(Message::PlaySample(sample.path.to_string())),
                                    column![text(name).size(25), type_label],
                                    horizontal_space(),
                                    fav_button,
                                    edit_button,
                                ]
                                .align_items(Alignment::Center),
                            );
                            result_row =
                                result_row.push(sample_entry.align_y(alignment::Vertical::Center));
                        }
                    }
                    None => {
                        let no_samples_text = container(text("No results").size(30));
                        result_row = result_row.push(no_samples_text);
                    }
                }

                result_row = result_row.spacing(5);
                let result_scollable = container(
                    scrollable(result_row)
                        .style(|theme, status| themes::scrollbar_invis(theme, status)),
                )
                .padding(10);

                column![status_bar, title, input, filters, result_scollable]
                    .spacing(20)
                    .into()
            }
            ViewControl::Settings => views::settings(self),
        }
    }

    fn theme(&self) -> Theme {
        match &self.selected_theme {
            Some(theme) => theme.clone(),
            None => Theme::KanagawaDragon,
        }
    }
}

impl Default for AudioCloud {
    fn default() -> Self {
        Self::new().0
    }
}
