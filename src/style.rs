//! Style and theme for the [`TabBar`](crate::TabBar).

use iced::{Background, Color, Padding, Shadow, Theme, border::Radius};

/// Combined style used by the [`TabBar`](crate::TabBar).
#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    /// Style of the outer bar container.
    pub bar: BarStyle,
    /// Style of individual tabs.
    pub tab: TabStyle,
    /// Style of tab tooltips.
    pub tooltip: TooltipStyle,
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
            background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
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
    /// The background of the close icon.
    pub icon_background: Option<Background>,
    /// Border radius of the close icon.
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

/// The appearance of tab tooltips.
#[derive(Clone, Copy, Debug)]
pub struct TooltipStyle {
    /// The background of the tooltip.
    pub background: Background,
    /// The text color of the tooltip.
    pub text_color: Color,
    /// The border radius of the tooltip.
    pub border_radius: Radius,
    /// The border width of the tooltip.
    pub border_width: f32,
    /// The border color of the tooltip.
    pub border_color: Color,
    /// The padding inside the tooltip.
    pub padding: Padding,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.95)),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            border_radius: Radius::new(4.0),
            border_width: 1.0,
            border_color: Color::from_rgba(0.4, 0.4, 0.4, 0.8),
            padding: Padding::new(6.0).left(10.0).right(10.0),
        }
    }
}

/// The interaction status of a tab, used to select the appropriate style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The currently selected tab.
    Active,
    /// A tab that is not selected.
    Inactive,
    /// The cursor is hovering over the tab.
    Hovered,
    /// The tab is currently being dragged.
    Dragging,
}

/// A closure that maps a theme and status to a [`Style`].
pub type StyleFn<'a, Theme, Style> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

/// The Catalog of a [`TabBar`](crate::TabBar).
pub trait Catalog {
    /// The style class type.
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

/// The default style for a [`TabBar`](crate::TabBar).
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
            base.tab.background = Background::Color(Color::from_rgb(0.4, 0.4, 0.4));
        }
    }

    base
}
