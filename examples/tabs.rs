//! Demonstrates the tab bar widget with interactive controls.

use iced::{
    Alignment, Element,
    widget::{Button, Column, Container, Row, Slider, Text, TextInput, Toggler, pick_list},
};
use std::fmt;
use std::time::Duration;

use iced_fonts::CODICON_FONT_BYTES;
use iced_tabs::{Position, ScrollMode, TabBar, TabLabel};

const TAB_ICONS: &[char] = &[
    '\u{eb51}', // gear
    '\u{eb06}', // home
    '\u{eb1e}', // bookmark
    '\u{eb35}', // bell
    '\u{ea7b}', // file
];

fn main() -> iced::Result {
    iced::application(
        TabBarExample::default,
        TabBarExample::update,
        TabBarExample::view,
    )
    .font(CODICON_FONT_BYTES)
    .run()
}

#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
    TabReordered(usize, usize),
    TabLabelInputChanged(String),
    TabContentInputChanged(String),
    NewTab,
    ScrollModeChanged(ScrollMode),
    TabSpacingChanged(f32),
    TabPaddingChanged(f32),
    TextSizeChanged(f32),
    IconSizeChanged(f32),
    CloseSizeChanged(f32),
    TabHeightChanged(f32),
    LabelSpacingChanged(f32),
    ShowCloseButtonToggled(bool),
    ReorderableToggled(bool),
    LabelTypeChanged(LabelTypeChoice),
    IconPositionChanged(PositionChoice),
    TooltipDelayChanged(f32),
}

/// Local enum for the scroll mode dropdown (maps to iced_tabs::ScrollMode).
#[derive(Debug, Clone, Copy, PartialEq)]
enum ScrollModeChoice {
    Floating,
    Below,
    NoScrollbar,
}

impl fmt::Display for ScrollModeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScrollModeChoice::Floating => write!(f, "Floating"),
            ScrollModeChoice::Below => write!(f, "Below"),
            ScrollModeChoice::NoScrollbar => write!(f, "No Scrollbar"),
        }
    }
}

impl From<ScrollModeChoice> for ScrollMode {
    fn from(c: ScrollModeChoice) -> Self {
        match c {
            ScrollModeChoice::Floating => ScrollMode::Floating,
            ScrollModeChoice::Below => ScrollMode::Below(4.0.into()),
            ScrollModeChoice::NoScrollbar => ScrollMode::NoScrollbar,
        }
    }
}

fn scroll_mode_to_choice(mode: &ScrollMode) -> ScrollModeChoice {
    match mode {
        ScrollMode::Floating => ScrollModeChoice::Floating,
        ScrollMode::Below(_) => ScrollModeChoice::Below,
        ScrollMode::NoScrollbar => ScrollModeChoice::NoScrollbar,
    }
}

/// Which kind of [`TabLabel`] to use for every tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LabelTypeChoice {
    #[default]
    Text,
    Icon,
    IconText,
}

impl fmt::Display for LabelTypeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabelTypeChoice::Text => write!(f, "Text"),
            LabelTypeChoice::Icon => write!(f, "Icon"),
            LabelTypeChoice::IconText => write!(f, "Icon + Text"),
        }
    }
}

/// Wraps [`Position`] so we can implement `Display` and use it in a pick list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PositionChoice {
    Top,
    Right,
    Bottom,
    #[default]
    Left,
}

impl fmt::Display for PositionChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionChoice::Top => write!(f, "Top"),
            PositionChoice::Right => write!(f, "Right"),
            PositionChoice::Bottom => write!(f, "Bottom"),
            PositionChoice::Left => write!(f, "Left"),
        }
    }
}

impl From<PositionChoice> for Position {
    fn from(c: PositionChoice) -> Self {
        match c {
            PositionChoice::Top => Position::Top,
            PositionChoice::Right => Position::Right,
            PositionChoice::Bottom => Position::Bottom,
            PositionChoice::Left => Position::Left,
        }
    }
}

#[derive(Debug)]
struct TabBarExample {
    active_tab: usize,
    new_tab_label: String,
    new_tab_content: String,
    tabs: Vec<(String, String, usize)>,

    // Runtime-configurable settings
    scroll_mode: ScrollMode,
    tab_spacing: f32,
    tab_padding: f32,
    text_size: f32,
    icon_size: f32,
    close_size: f32,
    tab_height: f32,
    label_spacing: f32,
    show_close_button: bool,
    reorderable: bool,
    label_type: LabelTypeChoice,
    icon_position: PositionChoice,
    tooltip_delay_ms: f32,
}

impl Default for TabBarExample {
    fn default() -> Self {
        Self {
            active_tab: 0,
            new_tab_label: String::new(),
            new_tab_content: String::new(),
            tabs: Vec::new(),
            scroll_mode: ScrollMode::default(),
            tab_spacing: 8.0,
            tab_padding: 8.0,
            text_size: 20.0,
            icon_size: 16.0,
            close_size: 16.0,
            tab_height: 35.0,
            label_spacing: 15.0,
            show_close_button: true,
            reorderable: true,
            label_type: LabelTypeChoice::default(),
            icon_position: PositionChoice::default(),
            tooltip_delay_ms: 700.0,
        }
    }
}

impl TabBarExample {
    fn update(&mut self, message: Message) {
        match message {
            Message::TabSelected(index) => {
                self.active_tab = index;
            }
            Message::TabClosed(index) => {
                self.tabs.remove(index);
                self.active_tab = if self.tabs.is_empty() {
                    0
                } else {
                    self.active_tab.min(self.tabs.len() - 1)
                };
            }
            Message::TabReordered(from, to) => {
                if from < self.tabs.len() && to < self.tabs.len() {
                    let tab = self.tabs.remove(from);
                    self.tabs.insert(to, tab);

                    // Keep the active tab tracking the same logical tab.
                    if self.active_tab == from {
                        self.active_tab = to;
                    } else if from < self.active_tab && to >= self.active_tab {
                        self.active_tab = self.active_tab.saturating_sub(1);
                    } else if from > self.active_tab && to <= self.active_tab {
                        self.active_tab = (self.active_tab + 1).min(self.tabs.len() - 1);
                    }
                }
            }
            Message::TabLabelInputChanged(value) => self.new_tab_label = value,
            Message::TabContentInputChanged(value) => self.new_tab_content = value,
            Message::NewTab => {
                if !self.new_tab_label.is_empty() && !self.new_tab_content.is_empty() {
                    let icon_idx = self.tabs.len() % TAB_ICONS.len();
                    self.tabs.push((
                        self.new_tab_label.to_owned(),
                        self.new_tab_content.to_owned(),
                        icon_idx,
                    ));
                    self.new_tab_label.clear();
                    self.new_tab_content.clear();
                }
            }
            Message::ScrollModeChanged(mode) => self.scroll_mode = mode,

            // Numeric sliders
            Message::TabSpacingChanged(v) => self.tab_spacing = v,
            Message::TabPaddingChanged(v) => self.tab_padding = v,
            Message::TextSizeChanged(v) => self.text_size = v,
            Message::IconSizeChanged(v) => self.icon_size = v,
            Message::CloseSizeChanged(v) => self.close_size = v,
            Message::TabHeightChanged(v) => self.tab_height = v,
            Message::LabelSpacingChanged(v) => self.label_spacing = v,

            // Toggles
            Message::ShowCloseButtonToggled(v) => self.show_close_button = v,
            Message::ReorderableToggled(v) => self.reorderable = v,

            // Enum pick lists
            Message::LabelTypeChanged(v) => self.label_type = v,
            Message::IconPositionChanged(v) => self.icon_position = v,

            Message::TooltipDelayChanged(v) => self.tooltip_delay_ms = v,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // -- Row 1: new-tab inputs + button -------------------------------------
        let new_tab_row = Row::new()
            .push(
                TextInput::new("Tab label", &self.new_tab_label)
                    .on_input(Message::TabLabelInputChanged)
                    .size(16)
                    .padding(5.0),
            )
            .push(
                TextInput::new("Tab content", &self.new_tab_content)
                    .on_input(Message::TabContentInputChanged)
                    .size(16)
                    .padding(5.0),
            )
            .push(Button::new(Text::new("New")).on_press(Message::NewTab))
            .align_y(Alignment::Center)
            .padding(10.0)
            .spacing(5.0);

        // -- Row 2: pick lists + toggles ----------------------------------------
        let controls_row = Row::new()
            .push(labeled(
                "Scroll mode:",
                pick_list(
                    [
                        ScrollModeChoice::Floating,
                        ScrollModeChoice::Below,
                        ScrollModeChoice::NoScrollbar,
                    ],
                    Some(scroll_mode_to_choice(&self.scroll_mode)),
                    |c| Message::ScrollModeChanged(c.into()),
                )
                .width(130),
            ))
            .push(labeled(
                "Label type:",
                pick_list(
                    [
                        LabelTypeChoice::Text,
                        LabelTypeChoice::Icon,
                        LabelTypeChoice::IconText,
                    ],
                    Some(self.label_type),
                    Message::LabelTypeChanged,
                )
                .width(130),
            ))
            .push(labeled(
                "Icon position:",
                pick_list(
                    [
                        PositionChoice::Top,
                        PositionChoice::Right,
                        PositionChoice::Bottom,
                        PositionChoice::Left,
                    ],
                    Some(self.icon_position),
                    Message::IconPositionChanged,
                )
                .width(110),
            ))
            .push(
                Toggler::new(self.show_close_button)
                    .on_toggle(Message::ShowCloseButtonToggled)
                    .label("Close button")
                    .size(20.0),
            )
            .push(
                Toggler::new(self.reorderable)
                    .on_toggle(Message::ReorderableToggled)
                    .label("Reorderable")
                    .size(20.0),
            )
            .align_y(Alignment::Center)
            .padding(10.0)
            .spacing(15.0);

        // -- Row 3: sliders -----------------------------------------------------
        let sliders_row = Row::new()
            .push(slider_control(
                "Spacing",
                self.tab_spacing,
                0.0,
                30.0,
                Message::TabSpacingChanged,
            ))
            .push(slider_control(
                "Padding",
                self.tab_padding,
                0.0,
                30.0,
                Message::TabPaddingChanged,
            ))
            .push(slider_control(
                "Text size",
                self.text_size,
                8.0,
                40.0,
                Message::TextSizeChanged,
            ))
            .push(slider_control(
                "Icon size",
                self.icon_size,
                8.0,
                40.0,
                Message::IconSizeChanged,
            ))
            .push(slider_control(
                "Close size",
                self.close_size,
                8.0,
                40.0,
                Message::CloseSizeChanged,
            ))
            .push(slider_control(
                "Height",
                self.tab_height,
                20.0,
                120.0,
                Message::TabHeightChanged,
            ))
            .push(slider_control(
                "Label spacing",
                self.label_spacing,
                0.0,
                30.0,
                Message::LabelSpacingChanged,
            ))
            .push(slider_control(
                "Tooltip delay",
                self.tooltip_delay_ms,
                0.0,
                2000.0,
                Message::TooltipDelayChanged,
            ))
            .align_y(Alignment::Center)
            .padding(10.0)
            .spacing(10.0);

        // -- Tab bar ------------------------------------------------------------
        let mut tab_bar = self
            .tabs
            .iter()
            .fold(
                TabBar::new(Message::TabSelected),
                |tab_bar, (tab_label, tab_content, icon_idx)| {
                    let idx = tab_bar.size();
                    let icon = TAB_ICONS[*icon_idx];
                    let label = match self.label_type {
                        LabelTypeChoice::Text => TabLabel::Text(tab_label.clone()),
                        LabelTypeChoice::Icon => TabLabel::Icon(icon),
                        LabelTypeChoice::IconText => TabLabel::IconText(icon, tab_label.clone()),
                    };
                    let tooltip = format!("{tab_label}: {tab_content}");
                    tab_bar.push_with_tooltip(idx, label, tooltip)
                },
            )
            .set_active_tab(&self.active_tab)
            .spacing(self.tab_spacing)
            .padding(self.tab_padding)
            .text_size(self.text_size)
            .icon_size(self.icon_size)
            .close_size(self.close_size)
            .height(self.tab_height)
            .label_spacing(self.label_spacing)
            .set_position(self.icon_position.into())
            .scroll_mode(self.scroll_mode)
            .tooltip_delay(Duration::from_millis(self.tooltip_delay_ms as u64));

        if self.show_close_button {
            tab_bar = tab_bar.on_close(Message::TabClosed);
        }
        if self.reorderable {
            tab_bar = tab_bar.on_reorder(Message::TabReordered);
        }

        // -- Content area -------------------------------------------------------
        let content = if let Some((_, content, _)) = self.tabs.get(self.active_tab) {
            Text::new(content)
        } else {
            Text::new("Please create a new tab")
        }
        .size(25);

        let content_area = Container::new(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        // -- Assemble -----------------------------------------------------------
        Column::new()
            .push(new_tab_row)
            .push(controls_row)
            .push(sliders_row)
            .push(tab_bar)
            .push(content_area)
            .into()
    }
}

/// A label followed by a widget, arranged horizontally.
fn labeled<'a>(label: &str, widget: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    Row::new()
        .push(Text::new(label.to_owned()).size(14))
        .push(widget)
        .align_y(Alignment::Center)
        .spacing(4.0)
        .into()
}

/// A vertical column containing a label with the current value and a slider.
fn slider_control<'a>(
    label: &str,
    value: f32,
    min: f32,
    max: f32,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    Column::new()
        .push(Text::new(format!("{label}: {value:.0}")).size(12))
        .push(
            Slider::new(min..=max, value, on_change)
                .step(1.0)
                .width(110),
        )
        .spacing(2.0)
        .align_x(Alignment::Center)
        .into()
}
