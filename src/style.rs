//! Style and theme for the [`TabBar`](crate::TabBar).

use crate::status::{Status, StyleFn};
use iced::{Background, Color, Shadow, Theme, border::Radius};

/// Combined style used by the [`TabBar`](crate::TabBar).
#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    /// Style of the outer bar container.
    pub bar: BarStyle,
    /// Style of individual tabs.
    pub tab: TabStyle,
}

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
    /// The background of the tab labels.
    pub background: Background,
    /// The border color of the tab labels.
    pub border_color: Color,
    /// The border width of the tab labels.
    pub border_width: f32,
    /// The border radius of a tab.
    pub border_radius: Radius,
    /// The icon color of the tab labels.
    pub icon_color: Color,
    /// The background of the closing icon.
    pub icon_background: Option<Background>,
    /// How soft/hard the corners of the icon border are.
    pub icon_border_radius: Radius,
    /// The text color of the tab labels.
    pub text_color: Color,
    /// Shadow applied to each tab.
    pub shadow: Shadow,
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::from_rgb(0.36, 0.39, 0.39)),
            border_color: [0.5, 0.5, 0.5].into(),
            border_radius: Radius::new(5.0),
            border_width: 1.0,
            icon_color: [0.5, 0.5, 0.5].into(),
            icon_background: Some(Background::Color(Color::from_rgba(1.0, 0.0, 0.0, 0.9))),
            icon_border_radius: 4.0.into(),
            text_color: [0.9, 0.9, 0.9].into(),
            shadow: Shadow::default(),
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
pub fn primary(_theme: &Theme, status: Status) -> Style {
    let mut base = Style::default();

    match status {
        Status::Inactive => {
            base.tab.background = Background::Color(Color::TRANSPARENT);
            base.tab.border_width = 0.0;
        }
        Status::Hovered => {
            base.tab.background = Background::Color(Color::from_rgba(0.7, 0.7, 0.7, 0.2));
            base.tab.border_width = 0.0;
        }
        Status::Active | Status::Dragging => {
            base.tab.background = Background::Color(Color::from_rgba(0.4, 0.4, 0.4, 0.9));
        }
    }

    base
}
