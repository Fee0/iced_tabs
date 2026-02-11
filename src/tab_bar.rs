//! Displays a [`TabBar`] to select the content to be displayed.
//!
//! You have to manage the logic to show the content yourself.
//!
//! *This API requires the following crate features to be activated: `tab_bar`*

use iced::advanced::{
    layout::{Limits, Node},
    mouse, renderer,
    widget::{operation::Scrollable as ScrollableOp, tree, Operation, Tree},
    Clipboard, Layout, Shell, Widget,
};
use iced::widget::{scrollable, text, Row, Scrollable, Space};
use iced::{
    Border, Color, Element, Event, Font, Length, Padding, Pixels, Rectangle, Shadow, Size, Vector,
};

use crate::status::{Status, StyleFn};
use crate::style::{Catalog, Style};
use crate::tab;
use crate::tab::TabLabel;
use iced::mouse::Cursor;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

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
/// Default width (in logical pixels) reserved for scroll buttons in `ButtonsOnly` mode.
const BUTTONS_AREA_WIDTH: f32 = 48.0;
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
    /// The statuses of the [`TabLabel`] and cross
    tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    /// The function that produces the message when a tab is selected.
    on_select: Arc<dyn Fn(TabId) -> Message>,
    /// The function that produces the message when the close icon was pressed.
    on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
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
    #[allow(clippy::missing_docs_in_private_items)]
    _renderer: PhantomData<Renderer>,
}

/// The [`Position`] of the icon relative to text, this enum is only relative if [`TabLabel::IconText`] is used.
#[derive(Clone, Copy, Debug, Default)]
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

/// Scroll behavior of the [`TabBar`].
///
/// This controls how overflowing tabs can be scrolled and how (or if) the
/// scrollbar is displayed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollMode {
    /// Scrollbar overlays the content when visible.
    Floating,
    /// Scrollbar is embedded in its own row below the tabs with the given spacing.
    Embedded(Pixels),
    /// Scrollbar is hidden; scrolling is done via mouse wheel and right-side `<` / `>` buttons.
    ButtonsOnly,
}

impl Default for ScrollMode {
    fn default() -> Self {
        Self::Embedded(DEFAULT_SCROLLBAR_SPACING)
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
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog + scrollable::Catalog,
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

    /// Sets the scroll behavior of the [`TabBar`].
    ///
    /// Use [`ScrollMode::Floating`] for a floating scrollbar,
    /// [`ScrollMode::Embedded`] for an always-visible embedded scrollbar,
    /// or [`ScrollMode::ButtonsOnly`] to hide the scrollbar and use `<` / `>`
    /// buttons on the right side instead.
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
        let scrollbar = match self.scroll_mode {
            ScrollMode::Floating => scrollable::Scrollbar::default(),
            ScrollMode::Embedded(spacing) => scrollable::Scrollbar::default().spacing(spacing),
            ScrollMode::ButtonsOnly => scrollable::Scrollbar::hidden(),
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
            &self.class,
        )
    }

    /// Returns the Scrollable wrapping TabBarContent (used to deliver scroll events in ButtonsOnly mode).
    fn scrollable_element(&self) -> Element<Message, Theme, Renderer> {
        Element::new(
            Scrollable::with_direction(
                Element::new(self.tab_content()),
                self.scrollbar_direction(),
            )
            .width(self.width)
            .height(self.height),
        )
    }

    /// Returns the inner element (Scrollable wrapping TabBarContent).
    pub(crate) fn wrapper_element(&self) -> Element<Message, Theme, Renderer> {
        let content = self.tab_content();
        let scrollable =
            Scrollable::with_direction(Element::new(content), self.scrollbar_direction())
                .width(self.width)
                .height(self.height);

        if matches!(self.scroll_mode, ScrollMode::ButtonsOnly) {
            // In buttons-only mode, reserve a fixed-width area on the right for the `<` / `>` buttons.
            let buttons_space = Space::new()
                .width(Length::Fixed(BUTTONS_AREA_WIDTH))
                .height(Length::Fill);
            let row = Row::new()
                .push(scrollable.width(Length::Fill))
                .push(buttons_space)
                .width(self.width)
                .height(self.height);
            Element::new(row)
        } else {
            Element::new(scrollable)
        }
    }
}

/// Widget implementation for [`TabBar`](TabBar).
impl<Message, TabId, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TabBar<'_, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog + scrollable::Catalog,
    TabId: Eq + Clone,
{
    fn size(&self) -> iced::Size<Length> {
        iced::Size::new(self.width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let mut element = self.wrapper_element();
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

        // In buttons-only mode, draw the `<` / `>` scroll buttons in the reserved right-side area.
        if matches!(self.scroll_mode, ScrollMode::ButtonsOnly) {
            // Layout: [ Scrollable | ButtonsSpace ]
            let mut children = layout.children();
            let _scrollable_layout = children.next();
            if let Some(buttons_layout) = children.next() {
                let buttons_bounds = buttons_layout.bounds();
                let (left_bounds, right_bounds) = split_buttons_area(buttons_bounds);

                // Slightly expand right hover area so the boundary pixel counts as ">" (some
                // backends use exclusive right edge for the left rect).
                let right_hover_bounds = Rectangle {
                    x: right_bounds.x - 1.0,
                    width: right_bounds.width + 1.0,
                    ..right_bounds
                };
                let is_left_hovered =
                    cursor.is_over(left_bounds) && !cursor.is_over(right_hover_bounds);
                let is_right_hovered = cursor.is_over(right_hover_bounds);

                // Use the same font as tab labels for the button glyphs so they are always visible
                let button_font = self.text_font.unwrap_or_default();

                draw_scroll_button(
                    renderer,
                    theme,
                    &self.class,
                    button_font,
                    '<',
                    left_bounds,
                    is_left_hovered,
                    viewport,
                );
                draw_scroll_button(
                    renderer,
                    theme,
                    &self.class,
                    button_font,
                    '>',
                    right_bounds,
                    is_right_hovered,
                    viewport,
                );
            }
        }
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
            // In buttons-only mode, treat clicks on the `<` / `>` buttons as horizontal scroll events.
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if matches!(self.scroll_mode, ScrollMode::ButtonsOnly) =>
            {
                if let Some(cursor_pos) = cursor.position() {
                    let mut children = layout.children();
                    let _scrollable_layout = children.next();
                    if let Some(buttons_layout) = children.next() {
                        let buttons_bounds = buttons_layout.bounds();
                        let (left_bounds, right_bounds) = split_buttons_area(buttons_bounds);
                        // Use same right_hover_bounds as in draw so hover and click areas match.
                        let right_hover_bounds = Rectangle {
                            x: right_bounds.x - 1.0,
                            width: right_bounds.width + 1.0,
                            ..right_bounds
                        };

                        const BUTTON_SCROLL_PIXELS: f32 = 120.0;

                        let delta_x = if left_bounds.contains(cursor_pos)
                            && !right_hover_bounds.contains(cursor_pos)
                        {
                            -BUTTON_SCROLL_PIXELS
                        } else if right_hover_bounds.contains(cursor_pos) {
                            BUTTON_SCROLL_PIXELS
                        } else {
                            0.0
                        };

                        if delta_x != 0.0 {
                            let modified = mouse::ScrollDelta::Pixels { x: delta_x, y: 0.0 };
                            Some(Event::Mouse(mouse::Event::WheelScrolled {
                                delta: modified,
                            }))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        // In buttons-only mode, request redraw when cursor moves so scroll button hover updates
        // (appears on enter, clears on leave).
        if matches!(self.scroll_mode, ScrollMode::ButtonsOnly) {
            if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
                shell.request_redraw();
            }
        }

        let event_ref = transformed_event.as_ref().unwrap_or(event);
        let did_transform = transformed_event.is_some();
        let transformed_from_click = did_transform
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            );

        if transformed_from_click {
            // Scroll programmatically via Operation so the Scrollable actually scrolls (its update()
            // ignores WheelScrolled when the cursor is outside its bounds, i.e. over the button).
            let delta_x = match event_ref {
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x, .. },
                }) => *x,
                _ => 0.0,
            };
            let mut layout_children = layout.children();
            let scrollable_layout = layout_children.next();
            let scroll_tree = state
                .children
                .get_mut(0)
                .and_then(|row| row.children.get_mut(0));
            if let (Some(scroll_layout), Some(scroll_tree)) = (scrollable_layout, scroll_tree) {
                let mut scrollable = self.scrollable_element();
                scroll_tree.diff(scrollable.as_widget_mut());
                let mut op = ScrollByOperation { delta_x };
                scrollable
                    .as_widget_mut()
                    .operate(scroll_tree, scroll_layout, renderer, &mut op);
                shell.request_redraw();
            }
            // Then update the Row with the original click so the event is consumed; we capture below.
            let mut element = self.wrapper_element();
            let tab_tree = if let Some(child_tree) = state.children.get_mut(0) {
                child_tree.diff(element.as_widget_mut());
                child_tree
            } else {
                let child_tree = Tree::new(element.as_widget());
                state.children.insert(0, child_tree);
                &mut state.children[0]
            };
            element.as_widget_mut().update(
                tab_tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
            shell.capture_event();
        } else {
            let mut element = self.wrapper_element();
            let tab_tree = if let Some(child_tree) = state.children.get_mut(0) {
                child_tree.diff(element.as_widget_mut());
                child_tree
            } else {
                let child_tree = Tree::new(element.as_widget());
                state.children.insert(0, child_tree);
                &mut state.children[0]
            };
            element.as_widget_mut().update(
                tab_tree, event_ref, layout, cursor, renderer, clipboard, shell, viewport,
            );
            if did_transform {
                shell.capture_event();
            }
        }

        // Sync tab_statuses from TabBarContent's tree state (correct cursor for hover in both layouts)
        if let Some(wrapper_tree) = state.children.get_mut(0) {
            // Structure differs depending on scroll mode:
            // - Floating/Embedded: wrapper_element = Scrollable -> TabBarContent
            // - ButtonsOnly: wrapper_element = Row -> Scrollable -> TabBarContent
            let content_state_opt: Option<&tab::TabBarContentState> = match self.scroll_mode {
                ScrollMode::ButtonsOnly => wrapper_tree
                    .children
                    .get_mut(0) // Row's first child = Scrollable
                    .and_then(|scroll_tree| scroll_tree.children.get_mut(0))
                    .map(|content_tree| {
                        content_tree.state.downcast_ref::<tab::TabBarContentState>()
                    }),
                _ => wrapper_tree.children.get_mut(0).map(|content_tree| {
                    content_tree.state.downcast_ref::<tab::TabBarContentState>()
                }),
            };

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
    Renderer: 'a + renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: 'a + Catalog + text::Catalog + scrollable::Catalog,
    Message: 'a,
    TabId: 'a + Eq + Clone,
{
    fn from(tab_bar: TabBar<'a, Message, TabId, Theme, Renderer>) -> Self {
        Element::new(tab_bar)
    }
}

/// Operation that scrolls the first Scrollable by a horizontal delta (used for button clicks).
struct ScrollByOperation {
    delta_x: f32,
}

impl Operation<()> for ScrollByOperation {
    fn scrollable(
        &mut self,
        _id: Option<&iced::widget::Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn ScrollableOp,
    ) {
        state.scroll_by(
            scrollable::AbsoluteOffset {
                x: self.delta_x,
                y: 0.0,
            },
            bounds,
            content_bounds,
        );
    }

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }
}

/// Split the reserved buttons area into two equal rectangles for `<` and `>` buttons.
fn split_buttons_area(bounds: Rectangle) -> (Rectangle, Rectangle) {
    let half_width = bounds.width / 2.0;

    let left = Rectangle {
        x: bounds.x,
        y: bounds.y,
        width: half_width,
        height: bounds.height,
    };

    let right = Rectangle {
        x: bounds.x + half_width,
        y: bounds.y,
        width: half_width,
        height: bounds.height,
    };

    (left, right)
}

#[allow(clippy::too_many_arguments)]
fn draw_scroll_button<Theme, Renderer>(
    renderer: &mut Renderer,
    theme: &Theme,
    class: &<Theme as Catalog>::Class<'_>,
    font: Font,
    label: char,
    bounds: Rectangle,
    is_hovered: bool,
    _viewport: &Rectangle,
) where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog,
{
    use iced::advanced::widget::text::{LineHeight, Wrapping};

    let style = Catalog::style(
        theme,
        class,
        if is_hovered {
            Status::Hovered
        } else {
            Status::Disabled
        },
    );

    // Optional subtle background on hover.
    if is_hovered {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: style.tab.icon_border_radius,
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                ..renderer::Quad::default()
            },
            style
                .tab
                .icon_background
                .unwrap_or_else(|| style.tab.label_background),
        );
    }

    // Draw the `<` / `>` glyph centered in the bounds.
    renderer.fill_text(
        iced::advanced::text::Text {
            content: label.to_string(),
            bounds: Size::new(bounds.width, bounds.height),
            size: Pixels(16.0),
            font,
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Center,
            line_height: LineHeight::Relative(1.3),
            shaping: text::Shaping::Advanced,
            wrapping: Wrapping::default(),
        },
        iced::Point::new(bounds.center_x(), bounds.center_y()),
        style.tab.text_color,
        bounds,
    );
}
