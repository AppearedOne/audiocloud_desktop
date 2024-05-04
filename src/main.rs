use audiocloud_lib::*;
use iced::font;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input, toggler,
    Column, Row, Toggler,
};
use iced::{
    alignment, executor, Alignment, Application, Command, Element, Length, Padding, Settings, Theme,
};
use iced_aw::graphics::icons::*;
use reqwest::*;

pub mod request;

#[tokio::main]
async fn main() -> iced::Result {
    Sampler::run(Settings::default())
}

struct Sampler {
    input: String,
    view: ViewControl,
    results: Option<SearchResult>,
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
    FontLoaded,
    SearchResultRecived(SearchResult),
    PlaySample(String),
}

fn perform_search(params: SearchParams) -> Command<Message> {
    Command::perform(get_result(params), Message::SearchResultRecived)
}

async fn get_result(params: SearchParams) -> SearchResult {
    let client = Client::new();
    let response = client
        .post("http://127.0.0.1:4040/search")
        .json(&params)
        .send()
        .await
        .expect("Couldnt do reqwest")
        .text()
        .await
        .unwrap();

    let out: SearchResult = serde_json::from_str(&response).expect("Couldnt parse response");
    out
}
impl Application for Sampler {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                input: String::from(""),
                view: ViewControl::Main,
                results: None,
            },
            Command::batch(vec![
                font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(|_| Message::FontLoaded)
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Audiocloud")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputChanged(val) => {
                self.input = val;
                if self.input.is_empty() {
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
                return perform_search(params);
            }
            Message::CreateTask => {}
            Message::SettingsButtonToggled => match self.view {
                ViewControl::Settings => self.view = ViewControl::Main,
                ViewControl::Main => self.view = ViewControl::Settings,
            },
            Message::FontLoaded => {}
            Message::SearchResultRecived(val) => self.results = Some(val),
            Message::PlaySample(path) => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        match self.view {
            ViewControl::Main => {
                let settings =
                    button(text(icon_to_string(BootstrapIcon::GearFill)).font(BOOTSTRAP_FONT))
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
                            let name = &sample.name.replace(".wav", "");
                            let sample_entry = container(row![
                                button(
                                    text(icon_to_string(BootstrapIcon::PlayFill))
                                        .font(BOOTSTRAP_FONT)
                                        .size(25)
                                )
                                .on_press(Message::PlaySample(sample.name.to_string())),
                                text(name).size(25),
                                horizontal_space(),
                            ]);
                            result_row = result_row.push(sample_entry);
                        }
                    }
                    None => {
                        let no_samples_text = container(text("No results").size(30));
                        result_row = result_row.push(no_samples_text);
                    }
                }

                result_row = result_row;
                let result_scollable = container(scrollable(result_row)).padding(10);

                column![status_bar, title, input, result_scollable]
                    .spacing(20)
                    .into()
            }
            ViewControl::Settings => {
                let settings_button =
                    button(text(icon_to_string(BootstrapIcon::XLg)).font(BOOTSTRAP_FONT))
                        .on_press(Message::SettingsButtonToggled)
                        .padding([5, 10, 5, 10]);
                let status_bar = row![horizontal_space(), settings_button];

                let title = text("Settings")
                    .width(Length::Fill)
                    .size(35)
                    .horizontal_alignment(alignment::Horizontal::Center);

                column![status_bar, title].spacing(20).into()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::KanagawaDragon
    }
}
