use std::any;

use iced::border::Radius;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, scrollable::*, text, text_input,
    Scrollable,
};
use iced::{
    alignment, executor, Alignment, Background, Border, Color, Element, Executor, Font, Length,
    Padding, Task, Theme,
};

pub fn text_fg(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().primary.strong.color),
    }
}

pub fn string_to_theme(theme_str: &str) -> Option<Theme> {
    for theme_type in Theme::ALL {
        if theme_type.to_string() == theme_str {
            return Some(theme_type.clone());
        }
    }
    None
}

pub fn searchbar(
    theme: &Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let palette = theme.extended_palette();
    let mut style = text_input::default(theme, status);
    style.border = Border {
        color: palette.primary.strong.color,
        width: 0.0,
        radius: Radius::from(0.0),
    };
    style
}

pub fn scrollbar_invis(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let scrollbar = Scrollbar {
        background: None,
        border: Border::rounded(2),
        scroller: Scroller {
            color: palette.background.strong.color,
            border: Border::rounded(2),
        },
    };

    match status {
        Status::Active => Style {
            container: container::Style::default(),
            vertical_scrollbar: scrollbar,
            horizontal_scrollbar: scrollbar,
            gap: None,
        },
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        } => {
            let hovered_scrollbar = Scrollbar {
                scroller: Scroller {
                    color: palette.primary.strong.color,
                    ..scrollbar.scroller
                },
                ..scrollbar
            };

            Style {
                container: container::Style::default(),
                vertical_scrollbar: if is_vertical_scrollbar_hovered {
                    hovered_scrollbar
                } else {
                    scrollbar
                },
                horizontal_scrollbar: if is_horizontal_scrollbar_hovered {
                    hovered_scrollbar
                } else {
                    scrollbar
                },
                gap: None,
            }
        }
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        } => {
            let dragged_scrollbar = Scrollbar {
                scroller: Scroller {
                    color: palette.primary.base.color,
                    ..scrollbar.scroller
                },
                ..scrollbar
            };

            Style {
                container: container::Style::default(),
                vertical_scrollbar: if is_vertical_scrollbar_dragged {
                    dragged_scrollbar
                } else {
                    scrollbar
                },
                horizontal_scrollbar: if is_horizontal_scrollbar_dragged {
                    dragged_scrollbar
                } else {
                    scrollbar
                },
                gap: None,
            }
        }
    }
}

pub fn round_button(theme: &Theme, status: iced::widget::button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    let radius = 10.0;
    style.border.radius = Radius::from(radius);
    style
}
