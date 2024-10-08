#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use audiocloud_lib::*;
#[cfg(not(target_arch = "wasm32"))]
use clipboard_rs::{Clipboard, ClipboardContext};

use editor::{Editor, EditorEvent};
use helpers::hash_sample;
use iced::event::{self, Event};
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, text,
    text_input, tooltip, vertical_space,
};
use iced::window;
use iced::{alignment, Alignment, Element, Font, Length, Padding, Subscription, Task, Theme};
use rand::seq::SliceRandom;
use rand::thread_rng;
use request::get_editor_audio;
use rodio::{source::Source, Decoder};
use settings::{settings_changed, SettingsChanged};
use std::fs::File;
use std::io::BufReader;
use std::path::*;
use std::time::Instant;

pub mod audio;
pub mod bootstrap;
pub mod editor;
pub mod error;
pub mod helpers;
pub mod overlay_anchor;
pub mod request;
pub mod search;
pub mod settings;
pub mod themes;
pub mod waveform;
pub mod widgets;

const ARRAYLEN: i32 = 1200;

#[cfg_attr(target_arch = "wasm32", tokio::main(flavor = "current_thread"))]
#[cfg_attr(not(target_arch = "wasm32"), tokio::main)]
async fn main() -> iced::Result {
    iced::application(AudioCloud::title, AudioCloud::update, AudioCloud::view)
        .theme(AudioCloud::theme)
        .font(bootstrap::ICON_FONT_BYTES)
        .subscription(AudioCloud::subscription)
        .exit_on_close_request(false)
        .transparent(true)
        .run_with(|| {
            (
                AudioCloud::new().0,
                Task::perform(
                    settings::load_from_file("settings.json"),
                    Message::SettingsLoaded,
                ),
            )
        })
}

pub struct AudioCloud {
    input: String,
    view: ViewControl,
    results: Option<SearchResult>,
    server_status: Option<bool>,
    audio_devices: Option<audio::Handlers>,
    theme_state: combo_box::State<Theme>,
    selected_theme: Option<Theme>,

    search_options: search::SearchOptions,
    search_view_state: search::SearchViewState,

    settings_state: settings::SettingsState,

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
    Nothing(()),
    EventOccurred(Event),
    Exit(()),
    RecivedHandle,

    GoView(ViewControl),
    EditorSessionDL(Sample),
    EditorSession((Sample, String)),
    Editor(EditorEvent),

    SearchView(search::SearchView),

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
    PacksMetaRecived(Result<Vec<PackInfo>, error::Error>),
    ResetSettings,
    ResetCache,
    CacheReset(Vec<String>),

    ToggleFavourite(Sample),
    ShuffleResults,

    Settings(SettingsChanged),
}

const ICON_FONT: Font = Font::with_name("bootstrap-icons");

fn perform_search(params: SearchParams, path: String) -> Task<Message> {
    Task::perform(
        request::get_result(params, path),
        Message::SearchResultRecived,
    )
}
fn send_file_preview_dl(server_url: String, path: String) -> Task<Message> {
    Task::perform(
        request::get_temp_audio(server_url, path),
        Message::TempAudioLoaded,
    )
}
fn send_file_dl(server_url: String, path: String) -> Task<Message> {
    Task::perform(
        request::dl_sample(server_url, path),
        Message::SampleAudioDownloaded,
    )
}

impl AudioCloud {
    fn create_request_command(&self, input: String) -> Task<Message> {
        if input.is_empty() || input.eq("-") {
            return Task::none();
        }

        let sample_type_filter =
            if self.search_options.show_loops == self.search_options.show_oneshots {
                None
            } else {
                if self.search_options.show_oneshots {
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
            pack_id: self.search_view_state.pack_id.clone(),
            max_results: Some(self.settings.max_results),
        };
        return perform_search(params, self.settings.server_url.clone());
    }
    fn new() -> (Self, Task<Message>) {
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

                search_options: search::SearchOptions::new(),
                search_view_state: search::SearchViewState::new(),

                pack_meta: vec![],

                settings_state: settings::SettingsState::new(),
                settings: settings::Settings::default(),
                status_message: String::from("idle... "),

                player: widgets::Player {
                    is_playing: false,
                    name: "None".to_string(),
                    volume: 1.0,
                    last_update_playing: Instant::now(),
                },
                editor: Editor::empty(),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Audiocloud")
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchView(msg) => return search::search_update(msg, self),
            Message::Nothing(_) => (),
            Message::Editor(event) => return editor::editor_event(self, event),
            Message::Settings(val) => return settings_changed(self, val),
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    let mut set = self.settings.clone();
                    match &self.selected_theme {
                        Some(theme) => set.theme = theme.to_string(),
                        None => set.theme = "Dark".to_string(),
                    }

                    return Task::perform(
                        settings::save_to_file(set, "settings.json"),
                        Message::Exit,
                    );
                } else {
                    return Task::none();
                }
            }
            Message::Exit(_) => {
                println!("Saved!");
                return window::get_latest().and_then(window::close);
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
                } else if !self.search_options.show_all_favourites {
                    self.results = None
                }
            }
            Message::ServerUrlSubmited(url) => {
                self.settings.server_url = url;
                if !self.settings.server_url.ends_with("/") {
                    self.settings.server_url.push('/');
                }
                return Task::perform(
                    request::check_connection(self.settings.server_url.clone()),
                    Message::ServerStatusUpdate,
                );
            }
            Message::ServerStatusUpdate(status) => {
                self.server_status = Some(status);
            }

            Message::PlaySample(path, name) => {
                self.player.name = name;
                println!("{}", path);
                return send_file_preview_dl(self.settings.server_url.clone(), path);
            }
            Message::TempAudioLoaded(path) => {
                let file = BufReader::new(File::open(&path).expect("Couldnt open file"));
                let source_file = Decoder::new(file);
                let source = match source_file {
                    Err(_) => {
                        self.status_message = "Couldnt open downloaded file ".to_string();
                        return Task::none();
                    }
                    Ok(decoder) => decoder,
                };
                //let source_r = source.buffered().reverb(Duration::from_millis(40), 0.7);
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
                self.search_options.show_oneshots = val;
                return self.create_request_command(self.input.clone());
            }
            Message::ShowLoopsCheckbox(val) => {
                self.search_options.show_loops = val;
                return self.create_request_command(self.input.clone());
            }
            Message::ShowOnlyFavouritesToggled(val) => {
                self.search_options.show_only_favourites = val;
            }
            Message::ShowAllFavourites => {
                self.search_options.show_all_favourites = !self.search_options.show_all_favourites;
            }
            Message::LoadSettings => {
                return Task::perform(
                    settings::load_from_file("settings.json"),
                    Message::SettingsLoaded,
                )
            }
            Message::SettingsLoaded(val) => {
                self.settings = val;
                self.selected_theme = themes::string_to_theme(&self.settings.clone().theme);
                self.status_message = "Loaded settings ".to_string();
                return Task::perform(
                    request::get_packs_meta(self.settings.server_url.clone()),
                    Message::PacksMetaRecived,
                );
            }
            Message::PacksMetaRecived(m) => match m {
                Err(_) => self.status_message = "Failed to get IDs".to_string(),
                Ok(metas) => {
                    self.pack_meta = metas;
                    self.status_message = String::from("Recived PackIDs");
                }
            },
            Message::SaveSettings => {
                let mut set = self.settings.clone();
                match &self.selected_theme {
                    Some(theme) => set.theme = theme.to_string(),
                    None => set.theme = "Dark".to_string(),
                }

                return Task::perform(
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
                return send_file_dl(self.settings.server_url.clone(), path);
            }
            Message::SampleAudioDownloaded(path) => {
                self.settings.add_dl_entry(&path);
                self.status_message = "Downloaded sample ".to_string();
            }
            Message::CopySample(path) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let rel_str =
                        String::from("cached/".to_string() + &hash_sample(&path) + ".wav");
                    let relative_path = Path::new(&rel_str);
                    let mut absolute_path = std::env::current_dir().unwrap();
                    absolute_path.push(relative_path);
                    let abs_str: String = absolute_path.to_str().unwrap().to_string();
                    let filepath = vec![abs_str];
                    let ctx = ClipboardContext::new().expect("Couldnt init clipboard");
                    ctx.set_files(filepath).expect("couldnt set to clipboard");
                    self.status_message = String::from("Copied sample");
                    return Task::none();
                }
                #[cfg(target_arch = "wasm32")]
                {
                    self.status_message = String::from("Copying on WASM unsupported");
                }
            }
            Message::DragPerformed => {}
            Message::ResetSettings => {
                self.settings = settings::Settings::default();
                let mut set = self.settings.clone();
                match &self.selected_theme {
                    Some(theme) => set.theme = theme.to_string(),
                    None => set.theme = "Dark".to_string(),
                }

                return Task::perform(
                    settings::save_to_file(set, "settings.json"),
                    Message::SettingsSaved,
                );
            }
            Message::ResetCache => {
                return Task::perform(helpers::clear_cached(), Message::CacheReset);
            }
            Message::CacheReset(val) => {
                self.settings.dl_samples_hash = val;
            }
            Message::GoView(v) => self.view = v,
            Message::EditorSessionDL(sample) => {
                return Task::perform(
                    get_editor_audio(sample, self.settings.server_url.clone()),
                    Message::EditorSession,
                )
            }
            Message::EditorSession((nsample, _path)) => {
                self.editor.sample = nsample;
                self.editor.lowpass = None;
                self.editor.highpass = None;
                self.status_message = "Loading editor...".to_string();
                self.view = ViewControl::Editor;

                return Task::perform(
                    editor::load_editor_audio(self.editor.audio.clone()),
                    Message::Nothing,
                )
                .chain(Task::perform(
                    waveform::get_waveform_tk(self.editor.audio.clone()),
                    |val| Message::Editor(EditorEvent::WaveformReloaded(val)),
                ));
            }
            Message::ShuffleResults => match &mut self.results {
                Some(res) => res.samples.shuffle(&mut thread_rng()),
                None => (),
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        match self.view {
            ViewControl::Main => search::searchview(self),
            ViewControl::Settings => settings::settings(self),
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
