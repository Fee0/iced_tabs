//! Displays a [`TabBar`] to select the content to be displayed.
//!
//! You have to manage the logic to show the content yourself.

use iced::advanced::{
    Clipboard, Layout, Shell, Widget,
    layout::{Limits, Node},
    mouse, renderer,
    widget::{Operation, Tree, tree},
};
use iced::widget::{Scrollable, container, scrollable, text};
use iced::{Border, Color, Element, Event, Font, Length, Padding, Pixels, Rectangle, Size};

use crate::style::{Catalog, Style};
use crate::{tab, Status, StyleFn};
use crate::tab::TabLabel;
use iced::mouse::Cursor;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

const DEFAULT_ICON_SIZE: f32 = 16.0;
const DEFAULT_TEXT_SIZE: f32 = 16.0;
const DEFAULT_CLOSE_SIZE: f32 = 16.0;
const DEFAULT_PADDING: Padding = Padding::new(5.0);
const DEFAULT_SPACING: Pixels = Pixels::ZERO;
const DEFAULT_LABEL_SPACING: f32 = 4.0;
/// The default spacing for the embedded scrollbar (when not floating).
const DEFAULT_SCROLLBAR_SPACING: Pixels = Pixels(4.0);
/// Factor to convert vertical scroll lines to horizontal pixels (matches iced's scroll speed).
const VERTICAL_TO_HORIZONTAL_SCROLL_FACTOR: f32 = 60.0;

/// State for the `TabBar` widget tree (used for diff tag).
#[allow(missing_docs)]
pub(crate) struct TabBarState;

/// A tab bar to show tabs.
///
/// # Example
/// ```ignore
/// # use iced_tabs::{TabLabel, TabBar};
/// #
/// #[derive(Debug, Clone)]
/// enum Message {
///     TabSelected(TabId),
/// }
///
/// #[derive(PartialEq, Hash, Clone)]
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
    /// Per-tab status and close-button hover state.
    tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    /// The function that produces the message when a tab is selected.
    on_select: Arc<dyn Fn(TabId) -> Message>,
    /// The function that produces the message when the close icon was pressed.
    on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
    /// The function that produces the message when a tab is dragged to a new position.
    /// Takes `(from_index, to_index)`.
    on_reorder: Option<Arc<dyn Fn(usize, usize) -> Message>>,
    /// The width of the [`TabBar`].
    width: Length,
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
    /// Spacing between a tab's label content and its close button.
    label_spacing: f32,
    /// The optional icon font of the [`TabBar`].
    font: Option<Font>,
    /// The optional text font of the [`TabBar`].
    text_font: Option<Font>,
    /// The style of the [`TabBar`].
    class: <Theme as Catalog>::Class<'a>,
    /// Where the icon is placed relative to text
    position: Position,
    /// Scroll behavior and scrollbar visibility for the tab bar.
    scroll_mode: ScrollMode,
    _renderer: PhantomData<Renderer>,
}

/// Icon position relative to text. Only meaningful when using [`TabLabel::IconText`].
#[derive(Clone, Copy, Debug, Default)]
pub enum Position {
    /// Icon is placed above the text.
    Top,
    /// Icon is placed to the right of the text.
    Right,
    /// Icon is placed below the text.
    Bottom,
    #[default]
    /// Icon is placed to the left of the text (default).
    Left,
}

impl Position {
    /// Whether the icon and text are stacked vertically (Top/Bottom).
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// Whether the icon appears before the text in layout order (Top/Left).
    pub fn is_icon_first(self) -> bool {
        matches!(self, Self::Top | Self::Left)
    }
}

/// Scroll behavior of the [`TabBar`].
///
/// This controls how overflowing tabs can be scrolled and how (or if) the
/// scrollbar is displayed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollMode {
    /// Scrollbar overlays the content when visible.
    Floating,
    /// Scrollbar is embedded in its own row below the tabs with the given spacing.
    Below(Pixels),
    /// Scrollbar is hidden; scrolling is only possible via mouse wheel.
    NoScrollbar,
}

impl Default for ScrollMode {
    fn default() -> Self {
        Self::Below(DEFAULT_SCROLLBAR_SPACING)
    }
}

impl<'a, Message, TabId, Theme, Renderer> fmt::Debug for TabBar<'a, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: Catalog,
    TabId: Eq + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabBar")
            .field("active_tab", &self.active_tab)
            .field("tab_labels", &self.tab_labels)
            .field("size", &self.tab_indices.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("position", &self.position)
            .finish()
    }
}

impl<'a, Message, TabId, Theme, Renderer> TabBar<'a, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer
        + iced::advanced::text::Renderer<Font = Font>
        + iced::advanced::svg::Renderer,
    Theme: Catalog + text::Catalog + scrollable::Catalog + container::Catalog,
    TabId: Eq + Clone,
{
    /// Creates a new empty [`TabBar`].
    ///
    /// It expects the function that will be called if a tab is selected by
    /// the user. The function receives the id of the selected tab.
    pub fn new<F>(on_select: F) -> Self
    where
        F: 'static + Fn(TabId) -> Message,
    {
        Self::with_tab_labels(Vec::new(), on_select)
    }

    /// Similar to [`new`](Self::new) but with a given vector of [`TabLabel`]s.
    ///
    /// It expects:
    /// - A vector of `(TabId, TabLabel)` pairs for the initial tabs.
    /// - The function that will be called if a tab is selected by the user.
    ///   The function receives the id of the selected tab.
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
            on_reorder: None,
            width: Length::Fill,
            height: Length::Shrink,
            max_height: u32::MAX as f32,
            icon_size: DEFAULT_ICON_SIZE,
            text_size: DEFAULT_TEXT_SIZE,
            close_size: DEFAULT_CLOSE_SIZE,
            padding: DEFAULT_PADDING,
            spacing: DEFAULT_SPACING,
            label_spacing: DEFAULT_LABEL_SPACING,
            font: None,
            text_font: None,
            class: <Theme as Catalog>::default(),
            position: Position::default(),
            scroll_mode: ScrollMode::default(),
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

    /// Sets the message that will be produced when a tab is dragged to a new position.
    ///
    /// The callback receives `(from_index, to_index)` — the original position of
    /// the dragged tab and the position it should be moved to. The consumer is
    /// responsible for reordering their data accordingly.
    ///
    /// Setting this enables drag-and-drop reordering of tabs.
    #[must_use]
    pub fn on_reorder<F>(mut self, on_reorder: F) -> Self
    where
        F: 'static + Fn(usize, usize) -> Message,
    {
        self.on_reorder = Some(Arc::new(on_reorder));
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

    /// Sets the spacing between a tab's label content and its close button.
    #[must_use]
    pub fn label_spacing(mut self, label_spacing: f32) -> Self {
        self.label_spacing = label_spacing;
        self
    }

    /// Sets the scroll behavior of the [`TabBar`].
    ///
    /// Use [`ScrollMode::Floating`] for a floating scrollbar,
    /// [`ScrollMode::Below`] for an always-visible embedded scrollbar,
    /// or [`ScrollMode::NoScrollbar`] to hide the scrollbar entirely (mouse wheel only).
    #[must_use]
    pub fn scroll_mode(mut self, mode: ScrollMode) -> Self {
        self.scroll_mode = mode;
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

    /// Sets the icon position relative to text. Only applies to [`TabLabel::IconText`].
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

    /// Sets the style class of the [`TabBar`].
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
        let scrollbar = match self.scroll_mode {
            ScrollMode::Floating => scrollable::Scrollbar::default(),
            ScrollMode::Below(spacing) => scrollable::Scrollbar::default().spacing(spacing),
            ScrollMode::NoScrollbar => scrollable::Scrollbar::hidden(),
        };
        scrollable::Direction::Horizontal(scrollbar)
    }

    fn tab_content(&self) -> tab::Tab<'_, 'a, Message, TabId, Theme, Renderer> {
        tab::Tab::new(
            self.tab_labels.clone(),
            self.tab_statuses.clone(),
            self.tab_indices.clone(),
            self.icon_size,
            self.text_size,
            self.close_size,
            self.label_spacing,
            self.padding,
            self.spacing,
            self.font,
            self.text_font,
            self.height,
            self.position,
            self.on_close.is_some(),
            self.active_tab
                .min(self.tab_indices.len().saturating_sub(1)),
            Arc::clone(&self.on_select),
            self.on_close.as_ref().map(Arc::clone),
            self.on_reorder.as_ref().map(Arc::clone),
            &self.class,
        )
    }

    /// Returns the inner element (Scrollable wrapping TabBarContent).
    pub(crate) fn wrapper_element(&self) -> Element<'_, Message, Theme, Renderer> {
        let content = self.tab_content();
        let scrollable_height = match self.scroll_mode {
            ScrollMode::Below(_) => Length::Shrink,
            _ => self.height,
        };
        let scrollable =
            Scrollable::with_direction(Element::new(content), self.scrollbar_direction())
                .width(self.width)
                .height(scrollable_height);

        Element::new(scrollable)
    }
}

/// Ensures that `children` has a first entry synchronised with `element`.
///
/// If the child already exists it is diffed; otherwise a fresh tree is created
/// and inserted. Returns a mutable reference to that child tree.
///
/// Accepts `&mut Vec<Tree>` (i.e. `&mut tree.children`) rather than `&mut Tree`
/// so callers can split-borrow `tree.state` and `tree.children` independently.
pub(crate) fn ensure_child_tree<'a, Message, Theme, Renderer>(
    children: &'a mut Vec<Tree>,
    element: &mut Element<'_, Message, Theme, Renderer>,
) -> &'a mut Tree
where
    Renderer: renderer::Renderer,
{
    if children.is_empty() {
        children.insert(0, Tree::new(element.as_widget()));
    } else {
        children[0].diff(element.as_widget_mut());
    }
    &mut children[0]
}

impl<Message, TabId, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TabBar<'_, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer
        + iced::advanced::text::Renderer<Font = Font>
        + iced::advanced::svg::Renderer,
    Theme: Catalog + text::Catalog + scrollable::Catalog + container::Catalog,
    TabId: Eq + Clone,
{
    fn size(&self) -> Size<Length> {
        let height = match self.scroll_mode {
            ScrollMode::Below(_) => Length::Shrink,
            _ => self.height,
        };
        Size::new(self.width, height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let mut element = self.wrapper_element();
        let tab_tree = ensure_child_tree(&mut tree.children, &mut element);

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
            Catalog::style(theme, &self.class, Status::Inactive)
        };

        if bounds.intersects(viewport) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet.bar.border_radius,
                        width: style_sheet.bar.border_width,
                        color: style_sheet.bar.border_color.unwrap_or(Color::TRANSPARENT),
                    },
                    shadow: style_sheet.bar.shadow,
                    ..renderer::Quad::default()
                },
                style_sheet
                    .bar
                    .background
                    .unwrap_or_else(|| Color::TRANSPARENT.into()),
            );
        }

        let element = self.wrapper_element();
        element.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarState>()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.wrapper_element().as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        let element = self.wrapper_element();
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

        let mut element = self.wrapper_element();
        let tab_tree = ensure_child_tree(&mut tree.children, &mut element);

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
        let transformed_event = match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta_x = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => {
                        *y * VERTICAL_TO_HORIZONTAL_SCROLL_FACTOR
                    }
                    mouse::ScrollDelta::Pixels { x, y } => *x + *y,
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

        let event_ref = transformed_event.as_ref().unwrap_or(event);
        let did_transform = transformed_event.is_some();

        {
            let mut element = self.wrapper_element();
            let tab_tree = ensure_child_tree(&mut state.children, &mut element);
            element.as_widget_mut().update(
                tab_tree, event_ref, layout, cursor, renderer, clipboard, shell, viewport,
            );
            if did_transform {
                shell.capture_event();
            }
        }

        if let Some(wrapper_tree) = state.children.get_mut(0) {
            let content_state_opt: Option<&tab::TabBarContentState> = wrapper_tree
                .children
                .get_mut(0)
                .map(|content_tree| content_tree.state.downcast_ref::<tab::TabBarContentState>());

            if let Some(content_state) = content_state_opt {
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
        let element = self.wrapper_element();
        element.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message, TabId, Theme, Renderer> From<TabBar<'a, Message, TabId, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a
        + renderer::Renderer
        + iced::advanced::text::Renderer<Font = Font>
        + iced::advanced::svg::Renderer,
    Theme: 'a + Catalog + text::Catalog + scrollable::Catalog + container::Catalog,
    Message: 'a,
    TabId: 'a + Eq + Clone,
{
    fn from(tab_bar: TabBar<'a, Message, TabId, Theme, Renderer>) -> Self {
        Element::new(tab_bar)
    }
}
