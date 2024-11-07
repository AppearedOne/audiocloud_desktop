use crate::bootstrap::*;
use crate::themes;
use crate::waveform::*;
use crate::{AudioCloud, Message, ViewControl, ARRAYLEN, ICON_FONT};
use audiocloud_lib::*;
use rodio::{source::Source, Decoder, OutputStream};

use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, stack, text,
    text_input, tooltip, vertical_space,
};
use iced::Element;
use iced::{Alignment, Padding, Subscription, Task, Theme};
use std::io::BufReader;
use std::sync::{Arc, RwLock};

pub struct Editor {
    pub sample: Sample,
    pub audio: Arc<RwLock<Vec<f32>>>,
    pub wav: [f32; ARRAYLEN as usize],
    pub highpass: Option<f32>,
    pub lowpass: Option<f32>,
}

impl Editor {
    pub fn load_sample(&mut self, sample: Sample) {
        self.sample = sample;
    }
    pub fn empty() -> Self {
        Editor {
            sample: Sample {
                name: "none".to_string(),
                path: "none".to_string(),
                sampletype: SampleType::OneShot,
            },
            audio: Arc::new(RwLock::new(vec![])),
            wav: [0.0; ARRAYLEN as usize],
            highpass: None,
            lowpass: None,
        }
    }
}

pub async fn load_editor_audio(audioref: Arc<RwLock<Vec<f32>>>) {
    let file = BufReader::new(std::fs::File::open("editor.wav").expect("Couldnt open file"));
    {
        let mut write_audio = match audioref.write() {
            Ok(val) => val,
            Err(_) => {
                return;
            }
        };
        *write_audio = Decoder::new(file)
            .expect("Couldnt parse audio")
            .convert_samples()
            .collect();
    }
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
    PlaybackStart,
    ReloadWaveform,
    WaveformReloaded([f32; ARRAYLEN as usize]),
}
pub fn editor_event(app: &mut AudioCloud, event: EditorEvent) -> Task<Message> {
    match event {
        EditorEvent::ReloadWaveform => {
            return Task::perform(get_waveform_tk(app.editor.audio.clone()), |val| {
                Message::Editor(EditorEvent::WaveformReloaded(val))
            })
        }
        EditorEvent::WaveformReloaded(val) => {
            app.editor.wav = val;
            app.status
                .set(crate::StatusBarLevel::Succes, "Loaded Waveform");
        }
        EditorEvent::PlaybackStart => {
            //let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            //let data = app.editor.audio.clone().read().unwrap().iter().copied();
            //stream_handle.play_once(*data);
        }
    }
    Task::none()
}

//fn transport_bar() -> Element<Message> {}

pub fn view(app: &AudioCloud) -> Element<Message> {
    let status_text = app.status.statusbar_text();
    let settings = button(text(icon_to_string(Bootstrap::HouseFill)).font(ICON_FONT))
        .on_press(Message::GoView(ViewControl::Main))
        .padding([5, 10]);
    let status_bar = container(
        row![horizontal_space(), status_text, settings]
            .spacing(10)
            .align_y(Alignment::Center),
    );

    let transport_bar = row![
        button(text(icon_to_string(Bootstrap::Play)).font(ICON_FONT))
            .on_press(Message::Editor(EditorEvent::PlaybackStart))
    ];

    let wav = container(
        waveform(app.editor.wav).color(
            app.selected_theme
                .clone()
                .unwrap()
                .extended_palette()
                .primary
                .base
                .color,
        ),
    )
    .style(themes::container_front);

    let eq = column![text("Equalizer").size(25),];

    column![status_bar, text("Editor"), wav, eq, transport_bar].into()
}
