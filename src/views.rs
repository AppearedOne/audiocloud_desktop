use iced::widget::{
    button, checkbox, column, combo_box, container, horizontal_space, row, scrollable, slider,
    text, text_input,
};
use iced::{alignment, Alignment, Element, Length, Padding, Theme};

use crate::bootstrap::*;
use crate::themes;
use crate::AudioCloud;
use crate::Message;
use crate::ICON_FONT;

pub fn settings(app: &AudioCloud) -> Element<Message> {
    let status_text = text(&app.status_message).style(themes::text_fg);
    let settings_button = button(text(icon_to_string(Bootstrap::XLg)).font(ICON_FONT))
        .on_press(Message::SettingsButtonToggled)
        .padding([5, 10, 5, 10]);
    let status_bar = row![horizontal_space(), status_text, settings_button]
        .spacing(10)
        .align_y(Alignment::Center);

    let title = text("Settings")
        .width(Length::Fill)
        .size(35)
        .horizontal_alignment(alignment::Horizontal::Center);

    let connection_status = match app.server_status {
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
            text("Server URL:"),
            container(
                text_input("http://127.0.0.1:4040/", &app.settings.server_url)
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
        .align_y(Alignment::Center)
        .padding(20)
        .spacing(15),
        row![
            text("Theme:"),
            combo_box(
                &app.theme_state,
                "No theme selected",
                app.selected_theme.as_ref(),
                Message::ThemeSelected
            )
        ]
        .align_y(Alignment::Center)
        .padding(20)
        .spacing(15),
        row![
            text("Max search results:"),
            text(app.settings.max_results),
            slider(
                std::ops::RangeInclusive::new(1, 100),
                app.settings.max_results,
                Message::MaxRequestsChanged
            )
        ]
        .align_y(Alignment::Center)
        .spacing(15)
        .padding(20),
        row![
            button(text("Save settings"))
                .on_press(Message::SaveSettings)
                .style(button::success),
            button(text("Reload settings")).on_press(Message::LoadSettings),
            button(text("Reset settings"))
                .style(button::danger)
                .on_press(Message::ResetSettings),
            button(text("Reset downloads"))
                .style(button::danger)
                .on_press(Message::ResetCache)
        ]
        .spacing(15)
        .align_y(Alignment::Center)
        .padding(20),
    ];

    column![status_bar, title, settings].spacing(20).into()
}
