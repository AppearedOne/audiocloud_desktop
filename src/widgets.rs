use crate::bootstrap::*;
use crate::{AudioCloud, Message, ICON_FONT};
use iced::border::Radius;
use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_rule, horizontal_space, row, rule,
    scrollable, slider, text, text_input,
};
use iced::{
    alignment, Alignment, Element, Executor, Font, Length, Padding, Subscription, Task, Theme,
};
use std::ops::RangeInclusive;
use std::time::Instant;

pub struct Player {
    pub is_playing: bool,
    pub name: String,
    pub volume: f32,
    pub last_update_playing: Instant,
}

pub fn padding_now(num: i32) -> Padding {
    Padding {
        top: num as f32,
        right: num as f32,
        bottom: 0.0,
        left: num as f32,
    }
}

pub fn player_widget(app: &AudioCloud) -> Element<Message> {
    let icon = match app.player.is_playing {
        true => Bootstrap::Pause,
        false => Bootstrap::Play,
    };
    let play_button = button(text(icon_to_string(icon)).font(ICON_FONT).size(25))
        .style(button::text)
        .on_press(Message::TogglePlayer);

    let vol_icon: Bootstrap;
    let vol = app.player.volume;
    if vol == 0.0 {
        vol_icon = Bootstrap::VolumeMute;
    } else if vol < 0.3 {
        vol_icon = Bootstrap::VolumeOff;
    } else if vol < 1.0 {
        vol_icon = Bootstrap::VolumeDown;
    } else {
        vol_icon = Bootstrap::VolumeUp;
    }

    let vol_slider = slider(
        RangeInclusive::new(0.0, 2.0),
        app.player.volume,
        Message::VolumeChanged,
    )
    .width(Length::Fixed(160.0))
    .step(0.01);

    let row = row![
        play_button,
        text(&app.player.name),
        horizontal_space().width(Length::Fill),
        text(icon_to_string(vol_icon)).font(ICON_FONT).size(20),
        vol_slider,
    ]
    .align_y(Alignment::Center)
    .spacing(20);
    container(column![
        horizontal_rule(20).style(|theme: &Theme| rule::Style {
            color: theme.extended_palette().primary.strong.color,
            width: 4,
            radius: Radius::from(4.0),
            fill_mode: rule::FillMode::Full,
        }),
        row
    ])
    .padding(5)
    .into()
}
