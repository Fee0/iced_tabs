//! Style and theme for the [`TabBar`](crate::TabBar).
//!
//! You have to manage the logic to show the content yourself.
//!
//! *This API requires the following crate features to be activated: `tab_bar`*

use crate::status::{Status, StyleFn};
use iced::{border::Radius, Background, Color, Shadow, Theme, Vector};

/// The appearance of the outer tab bar container.
#[derive(Clone, Copy, Debug)]
pub struct BarStyle {
    /// The background of the tab bar.
    pub background: Option<Background>,

    /// The border color of the tab bar.
    pub border_color: Option<Color>,

    /// The border width of the tab bar.
    pub border_width: f32,

    /// The border radius of the tab bar.
    pub border_radius: Radius,

    /// Shadow applied to the outer bar.
    pub shadow: Shadow,
}

impl Default for BarStyle {
    fn default() -> Self {
        Self {
            background: None,
            border_color: None,
            border_width: 0.0,
            border_radius: Radius::default(),
            shadow: Shadow::default(),
        }
    }
}

/// The appearance of individual tabs and their labels.
#[derive(Clone, Copy, Debug)]
pub struct TabStyle {
    /// The border radius of a tab.
    pub border_radius: Radius,

    /// The background of the tab labels.
    pub label_background: Background,

    /// The border color of the tab labels.
    pub label_border_color: Color,

    /// The border width of the tab labels.
    pub label_border_width: f32,

    /// The icon color of the tab labels.
    pub icon_color: Color,

    /// The background of the closing icon.
    pub icon_background: Option<Background>,

    /// How soft/hard the corners of the icon border are.
    pub icon_border_radius: Radius,

    /// The text color of the tab labels.
    pub text_color: Color,

    /// Optional background for the scroll buttons (`<` / `>`) on hover. When `None`, scroll
    /// buttons have no visible background and blend into the tab bar (default).
    pub scroll_button_hover_background: Option<Background>,

    /// Shadow applied to each tab.
    pub shadow: Shadow,
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            border_radius: Radius::new(10.0),
            label_background: Background::Color([0.87, 0.87, 0.87].into()),
            label_border_color: [0.7, 0.7, 0.7].into(),
            label_border_width: 1.0,
            icon_color: Color::BLACK,
            icon_background: Some(Background::Color(Color::from_rgb(1.0, 0.0, 0.0))),
            icon_border_radius: 4.0.into(),
            text_color: Color::BLACK,
            scroll_button_hover_background: Some(Background::Color(Color::from_rgb(1.0, 0.0, 0.0))),
            shadow: Shadow {
                color: Color::BLACK,
                offset: Vector::new(5.0, 5.0),
                blur_radius: 5.0,
            },
        }
    }
}

/// Combined style used by the [`TabBar`](crate::TabBar).
#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// Style of the outer bar container.
    pub bar: BarStyle,

    /// Style of individual tabs.
    pub tab: TabStyle,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            bar: BarStyle::default(),
            tab: TabStyle::default(),
        }
    }
}
/// The Catalog of a [`TabBar`](crate::TabBar).
pub trait Catalog {
    ///Style for the trait to use.
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self, Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The primary theme of a [`TabBar`](crate::TabBar).
#[must_use]
pub fn primary(theme: &Theme, status: Status) -> Style {
    let mut base = Style::default();
    let palette = theme.extended_palette();

    base.tab.text_color = palette.background.base.text;

    match status {
        Status::Disabled => {
            base.tab.label_background = Background::Color(palette.background.strong.color);
        }
        Status::Hovered => {
            base.tab.label_background = Background::Color(palette.primary.strong.color);
        }
        Status::Active => {
            base.tab.label_background = Background::Color(palette.primary.base.color);
        }
    }

    base
}

/// The dark theme of a [`TabBar`](crate::TabBar).
#[must_use]
pub fn dark(_theme: &Theme, status: Status) -> Style {
    let mut base = Style::default();
    base.tab.label_background = Background::Color([0.1, 0.1, 0.1].into());
    base.tab.label_border_color = [0.3, 0.3, 0.3].into();
    base.tab.icon_color = Color::WHITE;
    base.tab.text_color = Color::WHITE;

    if status == Status::Disabled {
        base.tab.label_background = Background::Color([0.13, 0.13, 0.13].into());
    }

    base
}