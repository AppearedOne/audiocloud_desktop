use audiocloud_lib::PackInfo;
use iced::widget::tooltip::Position;
use iced::widget::{
    button, checkbox, column, container, horizontal_space, overlay, row, scrollable, stack, text,
    text_input, tooltip, vertical_space,
};
use iced::{alignment, Alignment, Element, Length, Padding, Subscription, Task, Theme};

use crate::settings::SearchViewTitle;
use crate::{bootstrap::*, request, ICON_FONT};
use crate::{helpers, themes, widgets, AudioCloud, Message, SampleType};
use crate::{overlay_anchor::anchored_overlay, widgets::*};

pub fn searchview(app: &AudioCloud) -> Element<Message> {
    let status_text = app.status.statusbar_text();
    let settings = button(text(icon_to_string(Bootstrap::GearFill)).font(ICON_FONT))
        .on_press(Message::SettingsButtonToggled)
        .padding([5, 10]);
    let status_bar = container(
        row![horizontal_space(), status_text, settings]
            .spacing(10)
            .align_y(Alignment::Center),
    );

    let title = match &app.settings.searchview_title {
        None => container(vertical_space().height(0)),
        Some(val) => match val {
            SearchViewTitle::Spacing => container(vertical_space().height(20)),
            SearchViewTitle::Nothing => container(vertical_space().height(0)),
            SearchViewTitle::Title => container(
                text("Audiocloud")
                    .width(Length::Fill)
                    .size(35)
                    .align_x(alignment::Horizontal::Center),
            ),
        },
    };

    let input_text = text_input("Some query...", &app.input)
        .on_input(Message::InputChanged)
        .on_submit(Message::CreateTask)
        .width(Length::FillPortion(6))
        .size(30)
        .style(themes::searchbar_text_only);

    let inputstyle = if app.settings.searchbar_gradient {
        themes::container_focus
    } else {
        container::transparent
    };
    let input = container(input_text)
        .padding(Padding {
            top: 5.0,
            bottom: 0.0,
            left: 50.0,
            right: 50.0,
        })
        .style(inputstyle)
        .align_y(Alignment::Center);

    let fav_all_text = match app.search_options.show_all_favourites {
        true => text(icon_to_string(Bootstrap::StarFill)).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().success),
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

    let tempo_filter_button = button(text("Tempo")).style(button::secondary);

    let pack_label = row![
        text(icon_to_string(Bootstrap::BoxSeam)).font(ICON_FONT),
        text("Packs")
    ]
    .spacing(10);
    let pack_filter_button = button(pack_label)
        .style(button::secondary)
        .on_press(Message::SearchView(SearchView::PackOverlay));
    let packfilter = if app.search_view_state.show_pack_overlay {
        container(anchored_overlay(
            pack_filter_button,
            pack_selector(&app.pack_meta),
            crate::overlay_anchor::Anchor::BelowTopCentered,
            10.0,
        ))
    } else {
        container(pack_filter_button)
    };

    let sample_type_selector = row![
        checkbox("OneShots", app.search_options.show_oneshots)
            .on_toggle(Message::ShowOneshotsCheckbox)
            .size(22),
        checkbox("Loops", app.search_options.show_loops)
            .on_toggle(Message::ShowLoopsCheckbox)
            .size(22),
        checkbox("Favourites", app.search_options.show_only_favourites)
            .on_toggle(Message::ShowOnlyFavouritesToggled)
            .size(22)
            .style(checkbox::success),
        tempo_filter_button,
        packfilter,
        horizontal_space(),
        shuffle_order,
        fav_all
    ]
    .align_y(Alignment::Center)
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
    match &app.results {
        Some(val) => {
            let samples = match app.search_options.show_all_favourites {
                true => &app.settings.favourite_samples,
                false => &val.samples,
            };
            for sample in samples {
                if app.search_options.show_only_favourites && !app.settings.is_favourite(sample) {
                    continue;
                }
                let name =
                    helpers::remove_brackets(&sample.name.replace(".wav", "").replace("_", " "));

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
                            text(" Loop").style(themes::text_fg),
                            text(tempo.to_string()).style(themes::text_fg).size(10),
                            text("bpm").size(10).style(themes::text_fg),
                        ]
                    }
                };

                let type_label = container(type_text).align_y(alignment::Vertical::Center);

                let edit_text = icon_to_string(Bootstrap::VinylFill);
                let edit_button = button(text(edit_text).size(20).font(ICON_FONT))
                    .on_press(Message::EditorSessionDL(sample.clone()))
                    .style(button::text);

                let fav_text = match app.settings.is_favourite(sample) {
                    true => text(icon_to_string(Bootstrap::StarFill)).style(|theme: &Theme| {
                        text::Style {
                            color: Some(theme.palette().success),
                        }
                    }),
                    false => text(icon_to_string(Bootstrap::Star)),
                };
                let fav_button = button(fav_text.font(ICON_FONT).size(20))
                    .style(button::text)
                    .on_press(Message::ToggleFavourite(sample.clone()));

                let dl_text = match app.settings.is_downloaded(&sample.path) {
                    false => text(icon_to_string(Bootstrap::Download)).style(|theme: &Theme| {
                        text::Style {
                            color: Some(theme.extended_palette().primary.strong.color),
                        }
                    }),
                    true => {
                        text(icon_to_string(Bootstrap::BoxArrowUpRight)).style(|theme: &Theme| {
                            text::Style {
                                color: Some(theme.palette().success),
                            }
                        })
                    }
                };
                let dl_button = match app.settings.is_downloaded(&sample.path) {
                    false => button(dl_text.font(ICON_FONT).size(20))
                        .style(button::text)
                        .on_press(Message::DownloadSample(sample.path.clone())),
                    true => button(dl_text.font(ICON_FONT).size(20))
                        .style(button::text)
                        .on_press(Message::DragSample(sample.path.clone())),
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
                        .on_press(Message::PlaySample(
                            sample.path.to_string(),
                            sample.name.clone()
                        )),
                        column![text(name).size(25), type_label],
                        horizontal_space(),
                        dl_button,
                        fav_button,
                        edit_button,
                    ]
                    .align_y(Alignment::Center),
                );
                result_row = result_row.push(sample_entry.align_y(alignment::Vertical::Center));
            }
        }
        None => {
            let no_samples_text = container(text("No results").size(30));
            result_row = result_row.push(no_samples_text);
        }
    }

    result_row = result_row.spacing(5);
    let result_scollable = container(
        scrollable(result_row).style(|theme, status| themes::scrollbar_invis(theme, status)),
    )
    .padding(widgets::padding_now(10));

    let spacer = vertical_space().height(Length::Fixed(20.0));
    let spacer2 = vertical_space().height(Length::Fixed(20.0));
    let spacer3 = vertical_space().height(Length::Fixed(1.0));
    column![
        status_bar,
        title,
        spacer,
        input,
        spacer2,
        filters,
        spacer3,
        result_scollable.height(Length::Fill),
        player_widget(&app)
    ]
    .into()
}

pub fn search_update(message: SearchView, app: &mut AudioCloud) -> Task<Message> {
    match message {
        SearchView::PackID(id) => {
            app.search_view_state.pack_id = id;
        }
        SearchView::PackOverlay => {
            app.search_view_state.show_pack_overlay = !app.search_view_state.show_pack_overlay;
            if app.pack_meta.is_empty() {
                return Task::perform(request::nothing(), |()| {
                    Message::SearchView(SearchView::GetPackIDS)
                });
            }
        }
        SearchView::GetPackIDS => {
            app.status
                .set(crate::StatusBarLevel::Neutral, "Getting IDs");
            return Task::perform(
                request::get_packs_meta(app.settings.server_url.clone()),
                Message::PacksMetaRecived,
            );
        }
    }
    Task::none()
}

#[derive(Clone, Debug)]
pub enum SearchView {
    PackID(Option<String>),
    PackOverlay,
    GetPackIDS,
}

pub struct SearchViewState {
    pub show_pack_overlay: bool,
    pub pack_id: Option<String>,
}
impl SearchViewState {
    pub fn new() -> Self {
        SearchViewState {
            show_pack_overlay: false,
            pack_id: None,
        }
    }
}

pub struct SearchOptions {
    pub show_oneshots: bool,
    pub show_loops: bool,
    pub show_only_favourites: bool,
    pub show_all_favourites: bool,
}
impl SearchOptions {
    pub fn new() -> Self {
        SearchOptions {
            show_oneshots: true,
            show_loops: true,
            show_only_favourites: false,
            show_all_favourites: false,
        }
    }
}

fn pack_selector(meta: &Vec<PackInfo>) -> Element<Message> {
    let mut list = column![];
    for pack in meta {
        list = list.push(
            button(
                column![text(pack.name.clone()), text(pack.description.clone())]
                    .padding(10)
                    .spacing(10),
            )
            .style(button::text)
            .on_press(Message::Nothing(())),
        )
    }
    container(list.push(text("All packs")))
        .style(container::rounded_box)
        .padding(10)
        .into()
}
