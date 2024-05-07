use audiocloud_lib::*;
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, text,
    text_input,
};
use iced::{
    alignment, executor, Alignment, Command, Element, Executor, Font, Length, Padding, Theme,
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

#[tokio::main]
async fn main() -> iced::Result {
    iced::program(AudioCloud::title, AudioCloud::update, AudioCloud::view)
        .theme(AudioCloud::theme)
        .font(BOOTSTRAP_FONT_BYTES)
        .run()
}

struct AudioCloud {
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
}

enum ViewControl {
    Main,
    Settings,
}

#[derive(Debug, Clone)]
enum Message {
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
            max_results: Some(50),
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
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Audiocloud")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
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
                if !self.show_loops && !self.show_oneshots {
                    self.show_loops = true;
                    self.show_oneshots = true;
                }
                return self.create_request_command(self.input.clone());
            }
            Message::ShowLoopsCheckbox(val) => {
                self.show_loops = val;
                if !self.show_loops && !self.show_oneshots {
                    self.show_loops = true;
                    self.show_oneshots = true;
                }
                return self.create_request_command(self.input.clone());
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.view {
            ViewControl::Main => {
                let settings =
                    button(text(icon_to_string(BootstrapIcon::GearFill)).font(ICON_FONT))
                        .on_press(Message::SettingsButtonToggled)
                        .padding([5, 10, 5, 10]);
                let status_bar = container(row![horizontal_space(), settings]);

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
            ViewControl::Settings => {
                let settings_button =
                    button(text(icon_to_string(BootstrapIcon::XLg)).font(ICON_FONT))
                        .on_press(Message::SettingsButtonToggled)
                        .padding([5, 10, 5, 10]);
                let status_bar = row![horizontal_space(), settings_button];

                let title = text("Settings")
                    .width(Length::Fill)
                    .size(35)
                    .horizontal_alignment(alignment::Horizontal::Center);

                let connection_status = match self.server_status {
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
                        text("Server URL: "),
                        container(
                            text_input("http://127.0.0.1:4040/", &self.server_url)
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
                    .align_items(Alignment::Center)
                    .padding(20),
                    row![
                        text("Theme: "),
                        combo_box(
                            &self.theme_state,
                            "No theme selected",
                            self.selected_theme.as_ref(),
                            Message::ThemeSelected
                        )
                    ]
                    .align_items(Alignment::Center)
                    .padding(20),
                ];

                column![status_bar, title, settings].spacing(20).into()
            }
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
