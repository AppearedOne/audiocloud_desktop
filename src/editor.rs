use crate::{AudioCloud, Message};
use audiocloud_lib::*;
use iced::widget::{row, text};
use iced::Element;

pub struct Editor {
    pub sample: Sample,
    pub highpass: Option<f32>,
    pub lowpass: Option<f32>,
}

pub fn view(app: &AudioCloud) -> Element<Message> {
    row![text("Hello")].into()
}
