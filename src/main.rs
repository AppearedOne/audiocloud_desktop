use audiocloud_lib::*;
#[cfg(not(target_arch = "wasm32"))]
use clipboard_rs::{Clipboard, ClipboardContext};

use editor::Editor;
use helpers::hash_sample;
use iced::event::{self, Event};
use iced::widget::tooltip::Position;
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, text,
    text_input, tooltip, vertical_space,
};
use iced::window;
use iced::{alignment, Alignment, Command, Element, Font, Length, Padding, Subscription, Theme};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rodio::{source::Source, Decoder};
use std::fs::File;
use std::io::BufReader;
use std::path::*;
use std::time::Instant;
use widgets::player_widget;

pub mod audio;

pub mod bootstrap;
use bootstrap::*;

pub mod editor;
pub mod helpers;
pub mod request;
pub mod settings;
pub mod themes;
pub mod views;
pub mod waveform;
pub mod widgets;

const ARRAYLEN: i32 = 800;
const ICON_FONT_BYTES: &[u8] = include_bytes!("assets/icons.ttf");

//#[cfg_attr(target_arch = "wasm32", tokio::main(flavor = "current_thread"))]
//#[cfg_attr(not(target_arch = "wasm32"), tokio::main)]
fn main() -> iced::Result {
    iced::program(AudioCloud::title, AudioCloud::update, AudioCloud::view)
        .theme(AudioCloud::theme)
        .font(ICON_FONT_BYTES)
        .subscription(AudioCloud::subscription)
        .load(|| {
            Command::perform(
                settings::load_from_file("settings.json"),
                Message::SettingsLoaded,
            )
        })
        .font(ICON_FONT_BYTES)
        .exit_on_close_request(false)
        .run()
}

pub struct AudioCloud {
    input: String,
    view: ViewControl,
    results: Option<SearchResult>,
    server_status: Option<bool>,
    audio_devices: Option<audio::Handlers>,
    theme_state: combo_box::State<Theme>,
    selected_theme: Option<Theme>,

    show_oneshots: bool,
    show_loops: bool,
    show_only_favourites: bool,
    show_all_favourites: bool,
    pack_meta: Vec<PackInfo>,

    settings: settings::Settings,
    status_message: String,

    player: widgets::Player,

    editor: Editor,
}

#[derive(Clone, Debug)]
pub enum ViewControl {
    Main,
    Settings,
    Editor,
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    Exit(()),
    RecivedHandle,

    GoView(ViewControl),
    EditorSessionDL(Sample),
    EditorSession(Sample, String),

    CopySample(String),
    DragPerformed,

    InputChanged(String),
    CreateTask,
    SettingsButtonToggled,
    SearchResultRecived(SearchResult),
    ServerStatusUpdate(bool),
    ServerUrlSubmited(String),

    PlaySample(String, String),
    TempAudioLoaded(String),
    DownloadSample(String),
    SampleAudioDownloaded(String),
    SamplePlayDone(Instant),
    TogglePlayer,
    VolumeChanged(f32),

    ThemeSelected(Theme),

    ShowOneshotsCheckbox(bool),
    ShowLoopsCheckbox(bool),
    ShowOnlyFavouritesToggled(bool),
    ShowAllFavourites,

    MaxRequestsChanged(i32),

    LoadSettings,
    SaveSettings,
    SettingsLoaded(settings::Settings),
    SettingsSaved(()),
    PacksMetaRecived(Vec<PackInfo>),
    ResetSettings,
    ResetCache,
    CacheReset(Vec<String>),

    ToggleFavourite(Sample),
    ShuffleResults,
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
fn send_file_dl(server_url: String, path: String) -> Command<Message> {
    Command::perform(
        request::dl_sample(server_url, path),
        Message::SampleAudioDownloaded,
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
        return perform_search(params, self.settings.server_url.clone());
    }
    fn new() -> (Self, Command<Message>) {
        (
            Self {
                input: String::from(""),
                view: ViewControl::Main,
                results: None,
                server_status: None,
                audio_devices: match audio::init_audio() {
                    Ok(handlers) => Some(handlers),
                    Err(_) => None,
                },
                theme_state: combo_box::State::new(Theme::ALL.to_vec()),
                selected_theme: None,
                show_oneshots: true,
                show_loops: true,
                show_only_favourites: false,
                show_all_favourites: false,
                pack_meta: vec![],

                settings: settings::Settings {
                    max_results: 50,
                    server_url: "http://127.0.0.1:4040/".to_string(),
                    theme: "Dark".to_string(),
                    favourite_samples: vec![],
                    dl_samples_hash: vec![],
                },
                status_message: String::from("idle... "),

                player: widgets::Player {
                    is_playing: false,
                    name: "None".to_string(),
                    volume: 1.0,
                    last_update_playing: Instant::now(),
                },
                editor: Editor::empty(),
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
                if let Event::Window(_id, window::Event::CloseRequested) = event {
                    let mut set = self.settings.clone();
                    match &self.selected_theme {
                        Some(theme) => set.theme = theme.to_string(),
                        None => set.theme = "Dark".to_string(),
                    }

                    return Command::perform(
                        settings::save_to_file(set, "settings.json"),
                        Message::Exit,
                    );
                } else {
                    return Command::none();
                }
            }
            Message::Exit(_) => {
                println!("Saved!");
                return window::close(window::Id::MAIN);
            }
            Message::RecivedHandle => {}
            Message::InputChanged(val) => {
                self.input = val;
                return self.create_request_command(self.input.clone());
            }
            Message::CreateTask => {}
            Message::SettingsButtonToggled => match self.view {
                ViewControl::Settings => self.view = ViewControl::Main,
                _ => self.view = ViewControl::Settings,
            },
            Message::SearchResultRecived(val) => {
                if val.samples.len() > 0 {
                    self.results = Some(val)
                } else if !self.show_all_favourites {
                    self.results = None
                }
            }
            Message::ServerUrlSubmited(url) => {
                self.settings.server_url = url;
                if !self.settings.server_url.ends_with("/") {
                    self.settings.server_url.push('/');
                }
                return Command::perform(
                    request::check_connection(self.settings.server_url.clone()),
                    Message::ServerStatusUpdate,
                );
            }
            Message::ServerStatusUpdate(status) => {
                self.server_status = Some(status);
            }

            Message::PlaySample(path, name) => {
                self.player.name = name;
                return send_file_preview_dl(self.settings.server_url.clone(), path);
            }
            Message::TempAudioLoaded(path) => {
                let file = BufReader::new(File::open(&path).expect("Couldnt open file"));
                let source = Decoder::new(file).expect("Couldnt build source from file");
                match &self.audio_devices {
                    Some(devs) => {
                        if !devs.sink.empty() {
                            devs.sink.clear();
                        }

                        let dur = &source.total_duration();
                        devs.sink.set_volume(self.player.volume);
                        devs.sink.append(source);
                        devs.sink.play();
                        self.player.is_playing = true;

                        let now = Instant::now();
                        self.player.last_update_playing = now;
                        return audio::wait_playback_end(dur.clone(), now);
                    }
                    None => println!("Error loading devices from option"),
                }
            }
            Message::SamplePlayDone(mod_stamp) => {
                if mod_stamp == self.player.last_update_playing {
                    self.player.is_playing = false;
                    self.player.name = String::from("None");
                }
            }
            Message::TogglePlayer => match &self.audio_devices {
                Some(devs) => {
                    if !devs.sink.empty() {
                        if devs.sink.is_paused() {
                            devs.sink.play();
                        } else {
                            devs.sink.pause();
                        }
                        self.player.is_playing = !self.player.is_playing;
                        self.player.last_update_playing = Instant::now();
                    }
                }
                None => println!("Error loading devices from option"),
            },
            Message::VolumeChanged(val) => {
                self.player.volume = val;
                match &self.audio_devices {
                    Some(devs) => {
                        devs.sink.set_volume(val);
                    }
                    None => (),
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
            Message::ShowOnlyFavouritesToggled(val) => {
                self.show_only_favourites = val;
            }
            Message::ShowAllFavourites => {
                self.show_all_favourites = !self.show_all_favourites;
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
                return Command::perform(
                    request::get_packs_meta(self.settings.server_url.clone()),
                    Message::PacksMetaRecived,
                );
            }
            Message::PacksMetaRecived(metas) => {
                self.pack_meta = metas;
                self.status_message = String::from("Recived PackIDs");
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
            Message::MaxRequestsChanged(val) => {
                self.settings.max_results = val;
            }
            Message::DownloadSample(path) => {
                return send_file_dl(self.settings.server_url.clone(), path)
            }
            Message::SampleAudioDownloaded(path) => {
                self.settings.add_dl_entry(&path);
                self.status_message = "Downloaded sample ".to_string();
            }
            Message::CopySample(path) => {
                /*return iced::window::run_with_handle(iced::window::Id::MAIN, |window| {
                    start_drag(
                        &window,
                        DragItem::Files(vec![PathBuf::from(path)]),
                        Image::Raw(include_bytes!("../icon.png").to_vec()),
                        |result: DragResult, cursor_pos: CursorPosition| {
                            println!(
                                "--> Drop Result: [{:?}], Cursor Pos:[{:?}]",
                                result, cursor_pos
                            );
                        },
                        Options {
                            skip_animatation_on_cancel_or_failure: false,
                        },
                    )
                    .expect("Couldnt start_drag");
                    Message::DragPerformed
                });*/
                let rel_str = String::from("cached/".to_string() + &hash_sample(&path) + ".wav");
                let relative_path = Path::new(&rel_str);
                let mut absolute_path = std::env::current_dir().unwrap();
                absolute_path.push(relative_path);
                let abs_str: String = absolute_path.to_str().unwrap().to_string();
                let filepath = vec![abs_str];
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let ctx = ClipboardContext::new().expect("Couldnt init clipboard");
                    ctx.set_files(filepath).expect("couldnt set to clipboard");
                    self.status_message = String::from("Copied sample ");
                    return Command::none();
                }
                #[cfg(target_arch = "wasm32")]
                {
                    self.status_message = String::from("Copying on WASM unsupported");
                }
            }
            Message::DragPerformed => {}
            Message::ResetSettings => {
                self.settings = settings::Settings {
                    max_results: 50,
                    server_url: "http://127.0.0.1:4040/".to_string(),
                    theme: "Dark".to_string(),
                    favourite_samples: vec![],
                    dl_samples_hash: vec![],
                };
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
            Message::ResetCache => {
                return Command::perform(helpers::clear_cached(), Message::CacheReset);
            }
            Message::CacheReset(val) => {
                self.settings.dl_samples_hash = val;
            }
            Message::GoView(v) => self.view = v,
            Message::EditorSessionDL(sample) => {}
            Message::EditorSession(sample, audio) => {}
            Message::ShuffleResults => match &mut self.results {
                Some(res) => res.samples.shuffle(&mut thread_rng()),
                None => (),
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let status_text = text(&self.status_message).style(themes::text_fg);
        match self.view {
            ViewControl::Main => {
                let settings = button(text(icon_to_string(Bootstrap::GearFill)).font(ICON_FONT))
                    .on_press(Message::SettingsButtonToggled)
                    .padding([5, 10, 5, 10]);
                let status_bar = container(
                    row![horizontal_space(), status_text, settings]
                        .spacing(10)
                        .align_items(Alignment::Center),
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

                let fav_all_text = match self.show_all_favourites {
                    true => text(icon_to_string(Bootstrap::StarFill)).style(|theme: &Theme| {
                        text::Style {
                            color: Some(theme.palette().success),
                        }
                    }),
                    false => text(icon_to_string(Bootstrap::Star)),
                };
                let fav_all = tooltip(
                    button(fav_all_text.font(ICON_FONT).size(22))
                        .style(button::text)
                        .on_press(Message::ShowAllFavourites),
                    text("Show all favourites"),
                    Position::Left,
                )
                .gap(10)
                .style(container::rounded_box);
                let shuffle_order = tooltip(
                    button(
                        text(icon_to_string(Bootstrap::Shuffle))
                            .font(ICON_FONT)
                            .size(22),
                    )
                    .style(button::text)
                    .on_press(Message::ShuffleResults),
                    text("Shuffle results"),
                    Position::Left,
                )
                .gap(10)
                .style(container::rounded_box);

                let tempo_filter_button = button(text("Tempo"));

                let sample_type_selector = row![
                    checkbox("OneShots", self.show_oneshots)
                        .on_toggle(Message::ShowOneshotsCheckbox)
                        .size(22),
                    checkbox("Loops", self.show_loops)
                        .on_toggle(Message::ShowLoopsCheckbox)
                        .size(22),
                    checkbox("Favourites", self.show_only_favourites)
                        .on_toggle(Message::ShowOnlyFavouritesToggled)
                        .size(22)
                        .style(checkbox::success),
                    tempo_filter_button,
                    horizontal_space(),
                    shuffle_order,
                    fav_all
                ]
                .align_items(Alignment::Center)
                .padding(Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 0.0,
                    left: 20.0,
                })
                .spacing(15);

                let filter_label = container(text("Filters").size(22)).padding(Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 0.0,
                    left: 20.0,
                });
                let filters = container(column![filter_label, sample_type_selector]).padding(10);

                let mut result_row = column![];
                match &self.results {
                    Some(val) => {
                        let samples = match self.show_all_favourites {
                            true => &self.settings.favourite_samples,
                            false => &val.samples,
                        };
                        for sample in samples {
                            if self.show_only_favourites && !self.settings.is_favourite(sample) {
                                continue;
                            }
                            let name = helpers::remove_brackets(&sample.name.replace(".wav", ""));

                            let type_text = match sample.sampletype {
                                SampleType::OneShot => {
                                    row![
                                        text(icon_to_string(Bootstrap::Soundwave))
                                            .font(ICON_FONT)
                                            .style(themes::text_fg),
                                        text(" OneShot").style(themes::text_fg),
                                    ]
                                }
                                SampleType::Loop(tempo) => {
                                    row![
                                        text(icon_to_string(Bootstrap::ArrowRepeat))
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

                            let edit_text = icon_to_string(Bootstrap::VinylFill);
                            let edit_button = button(text(edit_text).size(20).font(ICON_FONT))
                                .on_press(Message::EditorSessionDL(sample.clone()))
                                .style(button::text);

                            let fav_text = match self.settings.is_favourite(sample) {
                                true => text(icon_to_string(Bootstrap::StarFill)).style(
                                    |theme: &Theme| text::Style {
                                        color: Some(theme.palette().success),
                                    },
                                ),
                                false => text(icon_to_string(Bootstrap::Star)),
                            };
                            let fav_button = button(fav_text.font(ICON_FONT).size(20))
                                .style(button::text)
                                .on_press(Message::ToggleFavourite(sample.clone()));

                            let dl_text = match self.settings.is_downloaded(&sample.path) {
                                false => text(icon_to_string(Bootstrap::Download)).style(
                                    |theme: &Theme| text::Style {
                                        color: Some(theme.extended_palette().primary.strong.color),
                                    },
                                ),
                                true => text(icon_to_string(Bootstrap::BoxArrowUpRight)).style(
                                    |theme: &Theme| text::Style {
                                        color: Some(theme.palette().success),
                                    },
                                ),
                            };
                            let dl_button = match self.settings.is_downloaded(&sample.path) {
                                false => button(dl_text.font(ICON_FONT).size(20))
                                    .style(button::text)
                                    .on_press(Message::DownloadSample(sample.path.clone())),
                                true => button(dl_text.font(ICON_FONT).size(20))
                                    .style(button::text)
                                    .on_press(Message::CopySample(sample.path.clone())),
                            };

                            let sample_entry = container(
                                row![
                                    button(
                                        text(icon_to_string(Bootstrap::PlayFill))
                                            .font(ICON_FONT)
                                            .size(25)
                                    )
                                    .style(|theme, status| button::text(theme, status))
                                    .padding(20)
                                    .on_press(
                                        Message::PlaySample(
                                            sample.path.to_string(),
                                            sample.name.clone()
                                        )
                                    ),
                                    column![text(name).size(25), type_label],
                                    horizontal_space(),
                                    dl_button,
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
                .padding(widgets::padding_now(10));

                let spacer = vertical_space().height(Length::Fixed(20.0));
                let spacer2 = vertical_space().height(Length::Fixed(20.0));
                let spacer3 = vertical_space().height(Length::Fixed(1.0));
                column![
                    status_bar,
                    vertical_space().height(Length::Fixed(20.0)),
                    title,
                    spacer,
                    input,
                    spacer2,
                    filters,
                    spacer3,
                    result_scollable.height(Length::Fill),
                    player_widget(&self)
                ]
                .into()
            }
            ViewControl::Settings => views::settings(self),
            ViewControl::Editor => editor::view(self),
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
