use audiocloud_lib::*;
use iced::font;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input,
};
use iced::{alignment, Alignment, Command, Element, Font, Length, Padding, Theme};
use iced_aw::graphics::icons::{bootstrap::icon_to_string, BootstrapIcon, BOOTSTRAP_FONT_BYTES};

pub mod helpers;
pub mod request;

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
    PlaySample(String),
    ServerStatusUpdate(bool),
    ServerUrlSubmited(String),
}

const ICON_FONT: Font = Font::with_name("bootstrap-icons");

fn perform_search(params: SearchParams, path: String) -> Command<Message> {
    Command::perform(
        request::get_result(params, path),
        Message::SearchResultRecived,
    )
}
impl AudioCloud {
    fn new() -> (Self, Command<Message>) {
        (
            Self {
                input: String::from(""),
                view: ViewControl::Main,
                results: None,
                server_url: "http://127.0.0.1:4040/".to_string(),
                server_status: None,
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
                if self.input.is_empty() || self.input.eq("-") {
                    return Command::none();
                }
                let params = SearchParams {
                    query: self.input.clone(),
                    sample_type: None,
                    max_tempo: None,
                    min_tempo: None,
                    pack_id: None,
                    max_results: Some(30),
                };
                return perform_search(params, self.server_url.clone());
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
            Message::PlaySample(_path) => {}
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

                let mut result_row = column![];
                match &self.results {
                    Some(val) => {
                        for sample in &val.samples {
                            let name = helpers::remove_brackets(&sample.name.replace(".wav", ""));

                            let type_text = match sample.sampletype {
                                SampleType::OneShot => {
                                    row![
                                        text(icon_to_string(BootstrapIcon::Soundwave))
                                            .font(ICON_FONT),
                                        text(" OneShot")
                                    ]
                                }
                                SampleType::Loop(tempo) => {
                                    row![
                                        text(icon_to_string(BootstrapIcon::ArrowRepeat))
                                            .font(ICON_FONT),
                                        text(" Loop "),
                                        text(tempo.to_string()).size(10),
                                        text("bpm").size(10)
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
                                    .style(|_theme, _style| button::Style {
                                        background: None,
                                        ..Default::default()
                                    })
                                    .padding(20)
                                    .on_press(Message::PlaySample(sample.name.to_string())),
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
                let result_scollable = container(scrollable(result_row)).padding(10);

                column![status_bar, title, input, result_scollable]
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
                        true => text("Server connected"),
                        false => text("Server unreachable"),
                    },
                    None => text("Unknown status"),
                };
                let settings = column![row![
                    text("Server URL: "),
                    text_input("http://127.0.0.1:4040/", &self.server_url)
                        .on_input(Message::ServerUrlSubmited),
                    connection_status,
                ]
                .align_items(Alignment::Center),];

                column![status_bar, title, settings].spacing(20).into()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::KanagawaDragon
    }
}

impl Default for AudioCloud {
    fn default() -> Self {
        Self::new().0
    }
}
