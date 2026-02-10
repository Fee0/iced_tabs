//! Displays a [`TabBar`] to select the content to be displayed.
//!
//! You have to manage the logic to show the content by yourself or you may want
//! to use the [`Tabs`](super::tabs::Tabs) widget instead.
//!
//! *This API requires the following crate features to be activated: `tab_bar`*

pub mod tab_content;
pub mod tab_label;
pub mod tab_row;

use iced::{
    Border, Color, Element, Event, Font, Length, Padding, Pixels, Rectangle, Shadow, Size,
    mouse::{self, Cursor},
};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget,
    layout::{Limits, Node},
    renderer,
    widget::{Operation, Tree, tree},
};
use iced::widget::{self, Container, Scrollable, container, scrollable};

use std::marker::PhantomData;
use std::sync::Arc;

pub use tab_content::TabBarContent;
pub use tab_label::TabLabel;
use crate::status::{Status, StyleFn};
use crate::style::{Catalog, Style};

/// The default icon size.
const DEFAULT_ICON_SIZE: f32 = 16.0;
/// The default text size.
const DEFAULT_TEXT_SIZE: f32 = 16.0;
/// The default size of the close icon.
const DEFAULT_CLOSE_SIZE: f32 = 16.0;
/// The default padding between the tabs.
const DEFAULT_PADDING: Padding = Padding::new(5.0);
/// The default spacing around the tabs.
const DEFAULT_SPACING: Pixels = Pixels::ZERO;
/// The default spacing for the embedded scrollbar (when not floating).
const DEFAULT_SCROLLBAR_SPACING: Pixels = Pixels(4.0);
/// Factor to convert vertical scroll lines to horizontal pixels (matches iced's scroll speed).
const VERTICAL_TO_HORIZONTAL_SCROLL_FACTOR: f32 = 60.0;

/// State for the `TabBar` widget tree (used for diff tag).
#[allow(missing_docs)]
struct TabBarState;

/// A tab bar to show tabs.
///
/// # Example
/// ```ignore
/// # use iced_aw::{TabLabel, TabBar};
/// #
/// #[derive(Debug, Clone)]
/// enum Message {
///     TabSelected(TabId),
/// }
///
/// #[derive(PartialEq, Hash)]
/// enum TabId {
///    One,
///    Two,
///    Three,
/// }
///
/// let tab_bar = TabBar::new(
///     Message::TabSelected,
/// )
/// .push(TabId::One, TabLabel::Text(String::from("One")))
/// .push(TabId::Two, TabLabel::Text(String::from("Two")))
/// .push(TabId::Three, TabLabel::Text(String::from("Three")))
/// .set_active_tab(&TabId::One);
/// ```
#[allow(missing_debug_implementations)]
pub struct TabBar<'a, Message, TabId, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: Catalog,
    TabId: Eq + Clone,
{
    /// The index of the currently active tab.
    active_tab: usize,
    /// The vector containing the labels of the tabs.
    tab_labels: Vec<TabLabel>,
    /// The vector containing the indices of the tabs.
    tab_indices: Vec<TabId>,
    /// The statuses of the [`TabLabel`] and cross
    tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    /// The function that produces the message when a tab is selected.
    on_select: Arc<dyn Fn(TabId) -> Message>,
    /// The function that produces the message when the close icon was pressed.
    on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
    /// The width of the [`TabBar`].
    width: Length,
    /// The width of the tabs of the [`TabBar`].
    tab_width: Length,
    /// The height of the [`TabBar`].
    height: Length,
    /// The maximum height of the [`TabBar`].
    max_height: f32,
    /// The icon size.
    icon_size: f32,
    /// The text size.
    text_size: f32,
    /// The size of the close icon.
    close_size: f32,
    /// The padding of the tabs of the [`TabBar`].
    padding: Padding,
    /// The spacing of the tabs of the [`TabBar`].
    spacing: Pixels,
    /// The optional icon font of the [`TabBar`].
    font: Option<Font>,
    /// The optional text font of the [`TabBar`].
    text_font: Option<Font>,
    /// The style of the [`TabBar`].
    class: <Theme as Catalog>::Class<'a>,
    /// Where the icon is placed relative to text
    position: Position,
    /// Scrollbar spacing: `None` = floating (overlays content), `Some` = embedded (own row).
    scrollbar_spacing: Option<Pixels>,
    #[allow(clippy::missing_docs_in_private_items)]
    _renderer: PhantomData<Renderer>,
}

/// The [`Position`] of the icon relative to text, this enum is only relative if [`TabLabel::IconText`] is used.
#[derive(Clone, Copy, Default)]
pub enum Position {
    /// Icon is placed above of the text.
    Top,
    /// Icon is placed right of the text.
    Right,
    /// Icon is placed below of the text.
    Bottom,
    #[default]
    /// Icon is placed left of the text, the default.
    Left,
}

impl<'a, Message, TabId, Theme, Renderer> TabBar<'a, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + widget::text::Catalog,
    TabId: Eq + Clone,
{
    /// Creates a new [`TabBar`] with the index of the selected tab and a specified
    /// message which will be send when a tab is selected by the user.
    ///
    /// It expects:
    ///     * the index of the currently active tab.
    ///     * the function that will be called if a tab is selected by the user.
    ///         It takes the index of the selected tab.
    pub fn new<F>(on_select: F) -> Self
    where
        F: 'static + Fn(TabId) -> Message,
    {
        Self::with_tab_labels(Vec::new(), on_select)
    }

    /// Similar to [`new`](Self::new) but with a given Vector of the [`TabLabel`]s.
    ///
    /// It expects:
    ///     * the index of the currently active tab.
    ///     * a vector containing the [`TabLabel`]s of the [`TabBar`].
    ///     * the function that will be called if a tab is selected by the user.
    ///         It takes the index of the selected tab.
    pub fn with_tab_labels<F>(tab_labels: Vec<(TabId, TabLabel)>, on_select: F) -> Self
    where
        F: 'static + Fn(TabId) -> Message,
    {
        Self {
            active_tab: 0,
            tab_indices: tab_labels.iter().map(|(id, _)| id.clone()).collect(),
            tab_statuses: tab_labels.iter().map(|_| (None, None)).collect(),
            tab_labels: tab_labels.into_iter().map(|(_, label)| label).collect(),
            on_select: Arc::new(on_select),
            on_close: None,
            width: Length::Fill,
            tab_width: Length::Fill,
            height: Length::Shrink,
            max_height: u32::MAX as f32,
            icon_size: DEFAULT_ICON_SIZE,
            text_size: DEFAULT_TEXT_SIZE,
            close_size: DEFAULT_CLOSE_SIZE,
            padding: DEFAULT_PADDING,
            spacing: DEFAULT_SPACING,
            font: None,
            text_font: None,
            class: <Theme as Catalog>::default(),
            position: Position::default(),
            scrollbar_spacing: Some(DEFAULT_SCROLLBAR_SPACING),
            _renderer: PhantomData,
        }
    }

    /// Sets the size of the close icon of the
    /// [`TabLabel`]s of the [`TabBar`].
    #[must_use]
    pub fn close_size(mut self, close_size: f32) -> Self {
        self.close_size = close_size;
        self
    }

    /// Gets the id of the currently active tab on the [`TabBar`].
    #[must_use]
    pub fn get_active_tab_id(&self) -> Option<&TabId> {
        self.tab_indices.get(self.active_tab)
    }

    /// Gets the index of the currently active tab on the [`TabBar`].
    #[must_use]
    pub fn get_active_tab_idx(&self) -> usize {
        self.active_tab
    }

    /// Gets the height of the [`TabBar`].
    #[must_use]
    pub fn get_height(&self) -> Length {
        self.height
    }

    /// Gets the width of the [`TabBar`].
    #[must_use]
    pub fn get_width(&self) -> Length {
        self.width
    }

    /// Sets the height of the [`TabBar`].
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the font of the icons of the
    /// [`TabLabel`]s of the [`TabBar`].
    #[must_use]
    pub fn icon_font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the icon size of the [`TabLabel`]s of the [`TabBar`].
    #[must_use]
    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size;
        self
    }

    /// Sets the maximum height of the [`TabBar`].
    #[must_use]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the message that will be produced when the close icon of a tab
    /// on the [`TabBar`] is pressed.
    ///
    /// Setting this enables the drawing of a close icon on the tabs.
    #[must_use]
    pub fn on_close<F>(mut self, on_close: F) -> Self
    where
        F: 'static + Fn(TabId) -> Message,
    {
        self.on_close = Some(Arc::new(on_close));
        self
    }

    /// Sets the padding of the tabs of the [`TabBar`].
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Pushes a [`TabLabel`] to the [`TabBar`].
    #[must_use]
    pub fn push(mut self, id: TabId, tab_label: TabLabel) -> Self {
        self.tab_labels.push(tab_label);
        self.tab_indices.push(id);
        self.tab_statuses.push((None, None));
        self
    }

    /// Gets the amount of tabs on the [`TabBar`].
    #[must_use]
    pub fn size(&self) -> usize {
        self.tab_indices.len()
    }

    /// Sets the spacing between the tabs of the [`TabBar`].
    #[must_use]
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into();
        self
    }

    /// Sets the scrollbar to floating mode (overlays content when visible).
    /// Clicking the scrollbar may interact with underlying tabs.
    #[must_use]
    pub fn scrollbar_floating(mut self) -> Self {
        self.scrollbar_spacing = None;
        self
    }

    /// Sets the scrollbar to embedded mode with the given spacing.
    /// The scrollbar is placed in its own row below the tabs, avoiding overlap.
    #[must_use]
    pub fn scrollbar_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.scrollbar_spacing = Some(spacing.into());
        self
    }

    /// Sets the font of the text of the
    /// [`TabLabel`]s of the [`TabBar`].
    #[must_use]
    pub fn text_font(mut self, text_font: Font) -> Self {
        self.text_font = Some(text_font);
        self
    }

    /// Sets the text size of the [`TabLabel`]s of the [`TabBar`].
    #[must_use]
    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size;
        self
    }

    /// Sets the width of a tab on the [`TabBar`].
    #[must_use]
    pub fn tab_width(mut self, width: Length) -> Self {
        self.tab_width = width;
        self
    }

    /// Sets up the active tab on the [`TabBar`].
    ///
    /// If the given `TabId` is not found, the active tab index remains unchanged.
    #[must_use]
    pub fn set_active_tab(mut self, active_tab: &TabId) -> Self {
        if let Some(idx) = self.tab_indices.iter().position(|id| id == active_tab) {
            self.active_tab = idx;
        }
        self
    }

    /// Sets the [`Position`] of the Icon next to Text, Only used in [`TabLabel::IconText`]
    #[must_use]
    pub fn set_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Sets the style of the [`TabBar`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme, Style>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme, Style>).into();
        self
    }

    /// Sets the class of the input of the [`TabBar`].
    #[must_use]
    pub fn class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the width of the [`TabBar`].
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    fn scrollbar_direction(&self) -> scrollable::Direction {
        let scrollbar = self
            .scrollbar_spacing
            .map_or_else(scrollable::Scrollbar::default, |spacing| {
                scrollable::Scrollbar::default().spacing(spacing)
            });
        scrollable::Direction::Horizontal(scrollbar)
    }

    fn tab_width_fills(&self) -> bool {
        matches!(self.tab_width, Length::Fill | Length::FillPortion(_))
    }

    fn tab_content(&self) -> TabBarContent<'_, 'a, Message, TabId, Theme, Renderer> {
        TabBarContent::new(
            self.tab_labels.clone(),
            self.tab_statuses.clone(),
            self.tab_indices.clone(),
            self.icon_size,
            self.text_size,
            self.close_size,
            self.padding,
            self.spacing,
            self.font,
            self.text_font,
            self.height,
            self.tab_width,
            self.position,
            self.on_close.is_some(),
            self.active_tab
                .min(self.tab_indices.len().saturating_sub(1)),
            Arc::clone(&self.on_select),
            self.on_close.as_ref().map(Arc::clone),
            &self.class,
        )
    }
}

impl<Message, TabId, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TabBar<'_, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + widget::text::Catalog + scrollable::Catalog + container::Catalog,
    TabId: Eq + Clone,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let tab_content = self.tab_content();
        let mut element: Element<Message, Theme, Renderer> = if self.tab_width_fills() {
            Element::new(
                Container::new(Element::new(tab_content))
                    .width(Length::Fill)
                    .height(self.height),
            )
        } else {
            let scrollable =
                Scrollable::with_direction(Element::new(tab_content), self.scrollbar_direction())
                    .width(self.width)
                    .height(self.height);
            Element::new(scrollable)
        };

        let tab_tree = if let Some(child_tree) = tree.children.get_mut(0) {
            child_tree.diff(element.as_widget_mut());
            child_tree
        } else {
            let child_tree = Tree::new(element.as_widget());
            tree.children.insert(0, child_tree);
            &mut tree.children[0]
        };

        let limits = limits.max_height(self.max_height);
        element.as_widget_mut().layout(tab_tree, renderer, &limits)
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = cursor.position().is_some_and(|pos| bounds.contains(pos));
        let style_sheet = if is_mouse_over {
            Catalog::style(theme, &self.class, Status::Hovered)
        } else {
            Catalog::style(theme, &self.class, Status::Disabled)
        };

        if bounds.intersects(viewport) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: 0.0.into(),
                        width: style_sheet.border_width,
                        color: style_sheet.border_color.unwrap_or(Color::TRANSPARENT),
                    },
                    shadow: Shadow::default(),
                    ..renderer::Quad::default()
                },
                style_sheet
                    .background
                    .unwrap_or_else(|| Color::TRANSPARENT.into()),
            );
        }

        let tab_content = self.tab_content();

        if self.tab_width_fills() {
            let container = Container::new(Element::new(tab_content))
                .width(Length::Fill)
                .height(self.height);
            Widget::draw(
                &container,
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        } else {
            let scrollable =
                Scrollable::with_direction(Element::new(tab_content), self.scrollbar_direction())
                    .width(self.width)
                    .height(self.height);

            Widget::draw(
                &scrollable,
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarState>()
    }

    fn children(&self) -> Vec<Tree> {
        let tab_content = self.tab_content();
        let element: Element<Message, Theme, Renderer> = if self.tab_width_fills() {
            Element::new(
                Container::new(Element::new(tab_content))
                    .width(Length::Fill)
                    .height(self.height),
            )
        } else {
            let scrollable =
                Scrollable::with_direction(Element::new(tab_content), self.scrollbar_direction())
                    .width(self.width)
                    .height(self.height);
            Element::new(scrollable)
        };
        vec![Tree::new(element.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        let tab_content = self.tab_content();
        let element: Element<Message, Theme, Renderer> = if self.tab_width_fills() {
            Element::new(
                Container::new(Element::new(tab_content))
                    .width(Length::Fill)
                    .height(self.height),
            )
        } else {
            let scrollable =
                Scrollable::with_direction(Element::new(tab_content), self.scrollbar_direction())
                    .width(self.width)
                    .height(self.height);
            Element::new(scrollable)
        };
        tree.diff_children(std::slice::from_ref(&element));
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds());

        let tab_content = self.tab_content();

        let mut element: Element<Message, Theme, Renderer> = if self.tab_width_fills() {
            let container = Container::new(Element::new(tab_content))
                .width(Length::Fill)
                .height(self.height);

            Element::new(container)
        } else {
            let scrollable =
                Scrollable::with_direction(Element::new(tab_content), self.scrollbar_direction())
                    .width(self.width)
                    .height(self.height);

            Element::new(scrollable)
        };

        let tab_tree = if let Some(child_tree) = tree.children.get_mut(0) {
            child_tree.diff(element.as_widget_mut());
            child_tree
        } else {
            let child_tree = Tree::new(element.as_widget());
            tree.children.insert(0, child_tree);
            &mut tree.children[0]
        };

        element
            .as_widget_mut()
            .operate(tab_tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.tab_width_fills() {
            let tab_content = self.tab_content();
            let mut container = Container::new(Element::new(tab_content))
                .width(Length::Fill)
                .height(self.height);
            Widget::update(
                &mut container,
                &mut state.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        } else {
            // When cursor over tab bar and scrollable is used, map vertical wheel to horizontal scroll
            let event_to_pass = match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    let delta_x = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            *y * VERTICAL_TO_HORIZONTAL_SCROLL_FACTOR
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => *y,
                    };
                    if delta_x != 0.0
                        && cursor
                            .position()
                            .is_some_and(|p| layout.bounds().contains(p))
                    {
                        let modified = mouse::ScrollDelta::Pixels { x: delta_x, y: 0.0 };
                        Some(Event::Mouse(mouse::Event::WheelScrolled {
                            delta: modified,
                        }))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let (event_ref, did_transform) =
                event_to_pass.as_ref().map_or((event, false), |e| (e, true));

            let tab_content = self.tab_content();
            let mut scrollable =
                Scrollable::with_direction(Element::new(tab_content), self.scrollbar_direction())
                    .width(self.width)
                    .height(self.height);

            Widget::update(
                &mut scrollable,
                &mut state.children[0],
                event_ref,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if did_transform {
                shell.capture_event();
            }
        }

        // Sync tab_statuses from TabBarContent's tree state (correct cursor for hover in both layouts)
        if let Some(child_tree) = state.children.get_mut(0) {
            let content_tree = if self.tab_width_fills() {
                // Container delegates to TabBarContent: state is on the direct child
                Some(child_tree)
            } else {
                // Scrollable wraps TabBarContent: state is on the grandchild
                child_tree.children.get_mut(0)
            };
            if let Some(content_tree) = content_tree {
                let content_state = content_tree
                    .state
                    .downcast_ref::<tab_content::TabBarContentState>();
                self.tab_statuses.clone_from(&content_state.tab_statuses);
            }
        }
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let tab_content = self.tab_content();

        if self.tab_width_fills() {
            let container = Container::new(Element::new(tab_content))
                .width(Length::Fill)
                .height(self.height);
            Widget::mouse_interaction(
                &container,
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        } else {
            let scrollable =
                Scrollable::with_direction(Element::new(tab_content), self.scrollbar_direction())
                    .width(self.width)
                    .height(self.height);

            Widget::mouse_interaction(
                &scrollable,
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }
}

impl<'a, Message, TabId, Theme, Renderer> From<TabBar<'a, Message, TabId, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: 'a + Catalog + widget::text::Catalog + scrollable::Catalog + container::Catalog,
    Message: 'a,
    TabId: 'a + Eq + Clone,
{
    fn from(tab_bar: TabBar<'a, Message, TabId, Theme, Renderer>) -> Self {
        Element::new(tab_bar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum TestTabId {
        One,
        Two,
        Three,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    enum TestMessage {
        TabSelected(TestTabId),
        TabClosed(TestTabId),
    }

    type TestTabBar<'a> =
        TabBar<'a, TestMessage, TestTabId, iced::Theme, iced::Renderer>;

    #[test]
    fn tab_bar_new_has_default_values() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected);

        assert_eq!(tab_bar.active_tab, 0);
        assert_eq!(tab_bar.tab_labels.len(), 0);
        assert_eq!(tab_bar.tab_indices.len(), 0);
        assert_eq!(tab_bar.width, Length::Fill);
        assert_eq!(tab_bar.height, Length::Shrink);
        assert!((tab_bar.icon_size - DEFAULT_ICON_SIZE).abs() < f32::EPSILON);
        assert!((tab_bar.text_size - DEFAULT_TEXT_SIZE).abs() < f32::EPSILON);
        assert!((tab_bar.close_size - DEFAULT_CLOSE_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn tab_bar_push_adds_tab() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected)
            .push(TestTabId::One, TabLabel::Text("Tab 1".to_owned()));

        assert_eq!(tab_bar.tab_labels.len(), 1);
        assert_eq!(tab_bar.tab_indices.len(), 1);
        assert_eq!(tab_bar.tab_indices[0], TestTabId::One);
    }

    #[test]
    fn tab_bar_push_multiple_tabs() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected)
            .push(TestTabId::One, TabLabel::Text("Tab 1".to_owned()))
            .push(TestTabId::Two, TabLabel::Text("Tab 2".to_owned()))
            .push(TestTabId::Three, TabLabel::Text("Tab 3".to_owned()));

        assert_eq!(tab_bar.tab_labels.len(), 3);
        assert_eq!(tab_bar.tab_indices.len(), 3);
    }

    #[test]
    fn tab_bar_set_active_tab_sets_correct_index() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected)
            .push(TestTabId::One, TabLabel::Text("Tab 1".to_owned()))
            .push(TestTabId::Two, TabLabel::Text("Tab 2".to_owned()))
            .push(TestTabId::Three, TabLabel::Text("Tab 3".to_owned()))
            .set_active_tab(&TestTabId::Two);

        assert_eq!(tab_bar.active_tab, 1);
        assert_eq!(tab_bar.get_active_tab_id(), Some(&TestTabId::Two));
    }

    #[test]
    fn tab_bar_get_active_tab_idx_returns_index() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected)
            .push(TestTabId::One, TabLabel::Text("Tab 1".to_owned()))
            .push(TestTabId::Two, TabLabel::Text("Tab 2".to_owned()))
            .set_active_tab(&TestTabId::Two);

        assert_eq!(tab_bar.get_active_tab_idx(), 1);
    }

    #[test]
    fn tab_bar_size_returns_number_of_tabs() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected)
            .push(TestTabId::One, TabLabel::Text("Tab 1".to_owned()))
            .push(TestTabId::Two, TabLabel::Text("Tab 2".to_owned()))
            .push(TestTabId::Three, TabLabel::Text("Tab 3".to_owned()));

        assert_eq!(tab_bar.size(), 3);
    }

    #[test]
    fn tab_bar_width_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).width(200);
        assert_eq!(tab_bar.width, Length::Fixed(200.0));
    }

    #[test]
    fn tab_bar_height_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).height(50);
        assert_eq!(tab_bar.height, Length::Fixed(50.0));
    }

    #[test]
    fn tab_bar_icon_size_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).icon_size(24.0);
        assert!((tab_bar.icon_size - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tab_bar_text_size_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).text_size(20.0);
        assert!((tab_bar.text_size - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tab_bar_close_size_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).close_size(18.0);
        assert!((tab_bar.close_size - 18.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tab_bar_on_close_enables_close_button() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).on_close(TestMessage::TabClosed);

        assert!(tab_bar.on_close.is_some());
    }

    #[test]
    fn tab_bar_with_tab_labels_creates_tabs() {
        let labels = vec![
            (TestTabId::One, TabLabel::Text("Tab 1".to_owned())),
            (TestTabId::Two, TabLabel::Text("Tab 2".to_owned())),
        ];

        let tab_bar = TestTabBar::with_tab_labels(labels, TestMessage::TabSelected);

        assert_eq!(tab_bar.tab_labels.len(), 2);
        assert_eq!(tab_bar.tab_indices.len(), 2);
    }

    #[test]
    fn tab_bar_tab_width_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).tab_width(Length::Fixed(100.0));
        assert_eq!(tab_bar.tab_width, Length::Fixed(100.0));
    }

    #[test]
    fn tab_bar_max_height_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).max_height(200.0);
        assert!((tab_bar.max_height - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tab_bar_padding_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).padding(10.0);
        assert_eq!(tab_bar.padding, Padding::from(10.0));
    }

    #[test]
    fn tab_bar_spacing_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).spacing(5.0);
        assert_eq!(tab_bar.spacing, Pixels::from(5.0));
    }

    #[test]
    fn tab_bar_scrollbar_floating_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).scrollbar_floating();
        assert_eq!(tab_bar.scrollbar_spacing, None);
    }

    #[test]
    fn tab_bar_scrollbar_spacing_sets_value() {
        let tab_bar = TestTabBar::new(TestMessage::TabSelected).scrollbar_spacing(8.0);
        assert_eq!(tab_bar.scrollbar_spacing, Some(Pixels::from(8.0)));
    }
}
