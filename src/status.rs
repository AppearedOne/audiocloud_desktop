use iced::widget::{
    button, checkbox, column, container, horizontal_space, overlay, row, scrollable, stack, text,
};
use iced::{alignment, Alignment, Element, Length, Padding, Subscription, Task, Theme};

use crate::{bootstrap::*, request, ICON_FONT};
use crate::{helpers, themes, widgets, AudioCloud, Message, SampleType};

pub struct StatusBar {
    pub level: StatusBarLevel,
    pub text: String,
}
impl StatusBar {
    pub fn set(&mut self, t: StatusBarLevel, txt: &str) {
        self.text = String::from(txt);
        self.level = t;
    }

    pub fn statusbar_text(&self) -> Element<Message> {
        let icon = match self.level {
            StatusBarLevel::Succes => {
                text(icon_to_string(Bootstrap::CheckCircle)).style(text::success)
            }
            StatusBarLevel::Danger => {
                text(icon_to_string(Bootstrap::DashCircle)).style(text::danger)
            }
            StatusBarLevel::Neutral => text(icon_to_string(Bootstrap::Circle)).style(text::primary),
        };
        row![
            icon.font(ICON_FONT),
            text(self.text.clone()).style(themes::text_fg)
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .into()
    }
    pub fn new() -> Self {
        StatusBar {
            level: StatusBarLevel::Neutral,
            text: String::from("Loaded"),
        }
    }
}

pub enum StatusBarLevel {
    Succes,
    Danger,
    Neutral,
}
