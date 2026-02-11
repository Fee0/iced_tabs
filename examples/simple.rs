// This example demonstrates how to use the tab bar widget
//
// It was written by Kaiden42 <gitlab@tinysn.com>

use iced::{
    widget::{Button, Column, Row, Text, TextInput},
    Alignment, Element,
};

use iced_fonts::CODICON_FONT_BYTES;
use iced_tabs::{dark, ScrollMode, TabBar, TabLabel};

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
    TabLabelInputChanged(String),
    TabContentInputChanged(String),
    NewTab,
    ScrollModeChanged(ScrollMode),
}

#[derive(Debug, Default)]
struct TabBarExample {
    active_tab: usize,
    new_tab_label: String,
    new_tab_content: String,
    tabs: Vec<(String, String)>,
    scroll_mode: ScrollMode,
}

impl TabBarExample {
    fn update(&mut self, message: Message) {
        match message {
            Message::TabSelected(index) => {
                println!("Tab selected: {}", index);
                self.active_tab = index
            }
            Message::TabClosed(index) => {
                self.tabs.remove(index);
                println!("active tab before: {}", self.active_tab);
                self.active_tab = if self.tabs.is_empty() {
                    0
                } else {
                    usize::max(0, usize::min(self.active_tab, self.tabs.len() - 1))
                };
                println!("active tab after: {}", self.active_tab);
            }
            Message::TabLabelInputChanged(value) => self.new_tab_label = value,
            Message::TabContentInputChanged(value) => self.new_tab_content = value,
            Message::NewTab => {
                println!("New");
                if !self.new_tab_label.is_empty() && !self.new_tab_content.is_empty() {
                    println!("Create");
                    self.tabs.push((
                        self.new_tab_label.to_owned(),
                        self.new_tab_content.to_owned(),
                    ));
                    self.new_tab_label.clear();
                    self.new_tab_content.clear();
                }
            }
            Message::ScrollModeChanged(mode) => {
                self.scroll_mode = mode;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        Column::new()
            .push(
                Row::new()
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
                    .push(Text::new("Scroll mode:"))
                    .push(
                        Row::new()
                            .spacing(4.0)
                            .push(
                                Button::new(Text::new("Floating"))
                                    .on_press(Message::ScrollModeChanged(
                                        ScrollMode::Floating,
                                    )),
                            )
                            .push(
                                Button::new(Text::new("Embedded"))
                                    .on_press(Message::ScrollModeChanged(
                                        ScrollMode::Embedded(4.0.into()),
                                    )),
                            )
                            .push(
                                Button::new(Text::new("Buttons"))
                                    .on_press(Message::ScrollModeChanged(
                                        ScrollMode::ButtonsOnly,
                                    )),
                            ),
                    )
                    .align_y(Alignment::Center)
                    .padding(10.0)
                    .spacing(5.0),
            )
            .push({
                let tab_bar = self
                    .tabs
                    .iter()
                    .fold(
                        TabBar::new(Message::TabSelected),
                        |tab_bar, (tab_label, _)| {
                            // manually create a new index for the new tab
                            // starting from 0, when there is no tab created yet
                            let idx = tab_bar.size();
                            tab_bar.push(idx, TabLabel::Text(tab_label.to_owned()))
                        },
                    )
                    .set_active_tab(&self.active_tab)
                    .on_close(Message::TabClosed)
                    .spacing(5.0)
                    .padding(5.0)
                    .text_size(32.0)
                    .style(dark)
                    .scroll_mode(self.scroll_mode);
                tab_bar
            })
            .push(
                if let Some((_, content)) = self.tabs.get(self.active_tab) {
                    Text::new(content)
                } else {
                    Text::new("Please create a new tab")
                }
                .size(25),
            )
            .into()
    }
}
