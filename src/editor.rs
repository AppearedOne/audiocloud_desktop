use crate::bootstrap::*;
use crate::themes;
use crate::waveform::waveform;
use crate::{AudioCloud, Message, ViewControl, ARRAYLEN, ICON_FONT};
use audiocloud_lib::*;
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, stack, text,
    text_input, tooltip, vertical_space,
};
use iced::Color;
use iced::Element;
use iced::{Alignment, Padding, Subscription, Task, Theme};

pub struct Editor {
    pub sample: Sample,
    pub audio: Vec<f32>,
    pub wav: [f32; ARRAYLEN as usize],
    pub highpass: Option<f32>,
    pub lowpass: Option<f32>,
}
impl Editor {
    pub fn load_audio(&mut self) {}
    pub fn load_sample(&mut self, sample: Sample) {
        self.sample = sample;
    }
    pub fn load_full(&mut self) {}
    pub fn empty() -> Self {
        Editor {
            sample: Sample {
                name: "none".to_string(),
                path: "none".to_string(),
                sampletype: SampleType::OneShot,
            },
            audio: vec![],
            wav: [0.0; ARRAYLEN as usize],
            highpass: None,
            lowpass: None,
        }
    }
}
pub fn view(app: &AudioCloud) -> Element<Message> {
    let status_text = text(&app.status_message).style(themes::text_fg);
    let settings = button(text(icon_to_string(Bootstrap::HouseFill)).font(ICON_FONT))
        .on_press(Message::GoView(ViewControl::Main))
        .padding([5, 10]);
    let status_bar = container(
        row![horizontal_space(), status_text, settings]
            .spacing(10)
            .align_y(Alignment::Center),
    );
    let wav = waveform(app.editor.wav).color(Color::WHITE);
    column![status_bar, text("ah"), wav].into()
}
