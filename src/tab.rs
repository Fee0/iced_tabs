//! Content widget for [`TabBar`](super::TabBar) (handles selection/close in content-space for Scrollable).

use crate::Status;
use crate::style::{Catalog, TooltipStyle};
use crate::tab_bar::{Position, ensure_child_tree};
use iced::advanced::svg;
use iced::advanced::{
    Clipboard, Layout, Overlay, Shell, Widget,
    layout::{Limits, Node},
    renderer,
    widget::{Operation, Tree, tree},
};
use iced::widget::{Column, Container, Row, Space, Text, container, text};
use iced::{
    Alignment, Border, Element, Event, Font, Length, Padding, Pixels, Point, Rectangle, Size,
    alignment::{Horizontal, Vertical},
    mouse, touch,
};
use iced_fonts::CODICON_FONT;
use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

/// Offset added to icon/text size during layout to prevent clipping.
const LAYOUT_SIZE_OFFSET: f32 = 1.0;
/// Multiplier for close button hit area (larger than icon for easier clicking).
const CLOSE_HIT_AREA_MULTIPLIER: f32 = 1.3;
const CLOSE_SVG: &[u8] = include_bytes!("../assets/close.svg");
/// Cached SVG handle for the close icon (avoids re-allocating on every draw call).
static CLOSE_SVG_HANDLE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(CLOSE_SVG));
/// The content label displayed on a tab in the [`TabBar`](super::TabBar).
#[derive(Clone, Hash, Debug)]
pub enum TabLabel {
    /// Only an icon.
    Icon(char),

    /// Only text.
    Text(String),

    /// An icon alongside text.
    IconText(char, String),
    // TODO: Support any element as a label.
}

impl From<char> for TabLabel {
    fn from(value: char) -> Self {
        Self::Icon(value)
    }
}

impl From<&str> for TabLabel {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for TabLabel {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<(char, &str)> for TabLabel {
    fn from(value: (char, &str)) -> Self {
        Self::IconText(value.0, value.1.to_owned())
    }
}

impl From<(char, String)> for TabLabel {
    fn from(value: (char, String)) -> Self {
        Self::IconText(value.0, value.1)
    }
}

/// Tracks the state of an in-progress tab drag operation.
#[derive(Debug, Clone)]
pub struct DragState {
    /// Index of the tab being dragged.
    pub tab_index: usize,
    /// Mouse position when the press occurred.
    pub press_origin: Point,
    /// Current mouse position (updated on every move event).
    pub current_pos: Point,
    /// Whether the mouse has moved past the drag threshold.
    pub is_dragging: bool,
    /// Horizontal offset from the tab's left edge to the press point.
    pub tab_offset_x: f32,
    /// Vertical offset from the tab's top edge to the press point.
    pub tab_offset_y: f32,
    /// Size of the dragged tab (set when drag threshold is crossed).
    pub tab_size: Size,
    /// Cursor position in window coordinates (updated at the TabBar level
    /// so it stays current even when the cursor leaves the Scrollable).
    pub overlay_pos: Point,
}

/// Tracks hover timing for a tab tooltip.
#[derive(Debug, Clone)]
pub struct TooltipState {
    /// Index of the tab being hovered.
    pub tab_index: usize,
    /// When the hover started.
    pub hover_start: Instant,
    /// Last-known cursor position (in window coordinates).
    pub cursor_pos: Point,
}

/// State stored in `TabBarContent`'s tree for persisting `tab_statuses`.
#[derive(Debug, Clone, Default)]
pub struct TabBarContentState {
    pub tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    /// Active drag-and-drop state, if any.
    pub drag: Option<DragState>,
    /// Active tooltip hover tracking, if any.
    pub tooltip: Option<TooltipState>,
}

pub struct Tab<'a, 'b, Message, TabId, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: Catalog,
    TabId: Eq + Clone,
{
    tab_labels: Vec<TabLabel>,
    tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    tab_indices: Vec<TabId>,
    icon_size: f32,
    text_size: f32,
    close_size: f32,
    close_spacing: f32,
    icon_spacing: f32,
    padding: Padding,
    spacing: Pixels,
    font: Option<Font>,
    text_font: Option<Font>,
    height: Length,
    position: Position,
    tab_width: Option<f32>,
    drag_threshold: f32,
    has_close: bool,
    on_select: Arc<dyn Fn(TabId) -> Message>,
    on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
    on_reorder: Option<Arc<dyn Fn(usize, usize) -> Message>>,
    active_tab: usize,
    tab_tooltips: Vec<Option<String>>,
    tooltip_delay: Duration,
    class: &'a <Theme as Catalog>::Class<'b>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, 'b, Message, TabId, Theme, Renderer> fmt::Debug
    for Tab<'a, 'b, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: Catalog,
    TabId: Eq + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tab")
            .field("tab_labels", &self.tab_labels)
            .field("tab_indices_len", &self.tab_indices.len())
            .field("active_tab", &self.active_tab)
            .field("has_close", &self.has_close)
            .field("position", &self.position)
            .finish()
    }
}

impl<'a, 'b, Message, TabId, Theme, Renderer> Tab<'a, 'b, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog + container::Catalog,
    TabId: Eq + Clone,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tab_labels: Vec<TabLabel>,
        tab_statuses: Vec<(Option<Status>, Option<bool>)>,
        tab_indices: Vec<TabId>,
        icon_size: f32,
        text_size: f32,
        close_size: f32,
        close_spacing: f32,
        icon_spacing: f32,
        padding: Padding,
        spacing: Pixels,
        font: Option<Font>,
        text_font: Option<Font>,
        height: Length,
        position: Position,
        tab_width: Option<f32>,
        drag_threshold: f32,
        has_close: bool,
        active_tab: usize,
        on_select: Arc<dyn Fn(TabId) -> Message>,
        on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
        on_reorder: Option<Arc<dyn Fn(usize, usize) -> Message>>,
        tab_tooltips: Vec<Option<String>>,
        tooltip_delay: Duration,
        class: &'a <Theme as Catalog>::Class<'b>,
    ) -> Self {
        Self {
            tab_labels,
            tab_statuses,
            tab_indices,
            icon_size,
            text_size,
            close_size,
            close_spacing,
            icon_spacing,
            padding,
            spacing,
            font,
            text_font,
            height,
            position,
            tab_width,
            drag_threshold,
            has_close,
            on_select,
            on_close,
            on_reorder,
            active_tab,
            tab_tooltips,
            tooltip_delay,
            class,
            _renderer: PhantomData,
        }
    }

    fn row_element(&self) -> Row<'_, Message, Theme, Renderer> {
        self.tab_labels
            .iter()
            .fold(Row::<Message, Theme, Renderer>::new(), |row, tab_label| {
                let label_row = build_single_tab_row::<Message, Theme, Renderer>(
                    tab_label,
                    self.icon_size,
                    self.text_size,
                    self.close_size,
                    self.close_spacing,
                    self.icon_spacing,
                    self.padding,
                    self.tab_width,
                    self.height,
                    self.has_close,
                    self.position,
                    self.font,
                    self.text_font,
                );
                row.push(label_row)
            })
            .width(Length::Shrink)
            .height(self.height)
            .spacing(self.spacing)
            .align_y(Alignment::Center)
    }
}

fn layout_icon<Theme, Renderer>(
    icon: &char,
    size: f32,
    font: Option<Font>,
) -> Text<'_, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
    Renderer::Font: From<Font>,
    Theme: text::Catalog,
{
    Text::<Theme, Renderer>::new(icon.to_string())
        .size(size)
        .font(font.unwrap_or_default())
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .shaping(text::Shaping::Advanced)
        .width(Length::Shrink)
}

fn layout_text<Theme, Renderer>(
    text: &str,
    size: f32,
    font: Option<Font>,
) -> Text<'_, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
    Renderer::Font: From<Font>,
    Theme: text::Catalog,
{
    Text::<Theme, Renderer>::new(text)
        .size(size)
        .font(font.unwrap_or_default())
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .shaping(text::Shaping::Advanced)
        .width(Length::Shrink)
}

/// Builds a single tab's layout row (label content + optional close button).
///
/// Used by both `Tab::row_element` and `DragTabOverlay::layout`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_single_tab_row<'a, Message: 'a, Theme: 'a, Renderer: 'a>(
    tab_label: &'a TabLabel,
    icon_size: f32,
    text_size: f32,
    close_size: f32,
    close_spacing: f32,
    icon_spacing: f32,
    padding: Padding,
    tab_width: Option<f32>,
    height: Length,
    has_close: bool,
    position: Position,
    font: Option<Font>,
    text_font: Option<Font>,
) -> Row<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog + container::Catalog,
{
    let mut label_row = Row::new()
        .push(
            match tab_label {
                TabLabel::Icon(icon) => Container::new(layout_icon(
                    icon,
                    icon_size + LAYOUT_SIZE_OFFSET,
                    font,
                ))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
                TabLabel::Text(text) => Container::new(layout_text(
                    text.as_str(),
                    text_size + LAYOUT_SIZE_OFFSET,
                    text_font,
                ))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
                TabLabel::IconText(icon, text) => {
                    let icon_el = layout_icon(icon, icon_size + LAYOUT_SIZE_OFFSET, font);
                    let text_el = layout_text(text.as_str(), text_size + LAYOUT_SIZE_OFFSET, text_font);
                    let (first, second): (
                        Element<'_, Message, Theme, Renderer>,
                        Element<'_, Message, Theme, Renderer>,
                    ) = if position.is_icon_first() {
                        (icon_el.into(), text_el.into())
                    } else {
                        (text_el.into(), icon_el.into())
                    };
                    let inner: Element<'_, Message, Theme, Renderer> = if position.is_vertical() {
                        Column::new()
                            .align_x(Alignment::Center)
                            .spacing(icon_spacing)
                            .push(first)
                            .push(second)
                            .into()
                    } else {
                        Row::new()
                            .align_y(Alignment::Center)
                            .spacing(icon_spacing)
                            .push(first)
                            .push(second)
                            .into()
                    };
                    Container::new(inner)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                }
            }
            .width(tab_width.map_or(Length::Shrink, Length::Fixed))
            .height(height),
        )
        .align_y(Alignment::Center)
        .padding(padding)
        .spacing(close_spacing)
        .width(tab_width.map_or(Length::Shrink, Length::Fixed));

    if has_close {
        label_row = label_row.push(
            Row::new()
                .width(Length::Fixed(
                    close_size * CLOSE_HIT_AREA_MULTIPLIER + LAYOUT_SIZE_OFFSET,
                ))
                .height(Length::Fixed(
                    close_size * CLOSE_HIT_AREA_MULTIPLIER + LAYOUT_SIZE_OFFSET,
                ))
                .align_y(Alignment::Center)
                .push(
                    Space::new()
                        .width(close_size + LAYOUT_SIZE_OFFSET)
                        .height(close_size + LAYOUT_SIZE_OFFSET),
                ),
        );
    }

    label_row
}

impl<Message, TabId, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tab<'_, '_, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font> + svg::Renderer,
    Theme: Catalog + text::Catalog + container::Catalog,
    TabId: Eq + Clone,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let mut element = Element::new(self.row_element());
        let tab_tree = ensure_child_tree(&mut tree.children, &mut element);

        element
            .as_widget_mut()
            .layout(tab_tree, renderer, &limits.width(Length::Shrink).loose())
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let content_state = state.state.downcast_ref::<TabBarContentState>();
        let drag = content_state.drag.as_ref();
        let is_dragging = drag.is_some_and(|d| d.is_dragging);

        let tab_layouts: Vec<_> = layout.children().collect();

        let ctx = DrawCtx {
            position: self.position,
            theme,
            class: self.class,
            icon_data: (self.font.unwrap_or(CODICON_FONT), self.icon_size),
            text_data: (self.text_font.unwrap_or_default(), self.text_size),
            close_size: self.close_size,
            viewport,
        };

        if !is_dragging {
            // Normal (non-drag) drawing: each tab at its own layout position.
            for ((i, tab), tab_layout) in self.tab_labels.iter().enumerate().zip(&tab_layouts) {
                let tab_status = self.tab_statuses.get(i).expect("Should have a status.");
                draw_tab(renderer, tab, tab_status, *tab_layout, &ctx);
            }
        } else if let Some(drag) = drag {
            let dragged_idx = drag.tab_index;
            let target = compute_drop_index(&tab_layouts, drag.current_pos.x, dragged_idx);

            // Build visual order: simulate removing the dragged tab and
            // inserting it at the target position.
            let mut visual_order: Vec<usize> = (0..tab_layouts.len())
                .filter(|&i| i != dragged_idx)
                .collect();
            let insert_at = target.min(visual_order.len());
            visual_order.insert(insert_at, dragged_idx);

            // Draw each non-dragged tab at its new visual slot position.
            for (slot, &tab_idx) in visual_order.iter().enumerate() {
                if tab_idx == dragged_idx {
                    continue;
                }

                let tab = &self.tab_labels[tab_idx];
                let tab_status = self
                    .tab_statuses
                    .get(tab_idx)
                    .expect("Should have a status.");

                let original_bounds = tab_layouts[tab_idx].bounds();
                let slot_bounds = tab_layouts[slot].bounds();
                let offset_x = slot_bounds.x - original_bounds.x;

                if offset_x.abs() < 0.5 {
                    draw_tab(renderer, tab, tab_status, tab_layouts[tab_idx], &ctx);
                } else {
                    renderer.with_translation(iced::Vector::new(offset_x, 0.0), |renderer| {
                        draw_tab(renderer, tab, tab_status, tab_layouts[tab_idx], &ctx);
                    });
                }
            }

            // The dragged tab itself is rendered by DragTabOverlay (via
            // TabBar::overlay), so nothing more to draw here.
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarContentState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TabBarContentState {
            tab_statuses: self.tab_statuses.clone(),
            drag: None,
            tooltip: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(Element::new(self.row_element()))]
    }

    fn diff(&self, tree: &mut Tree) {
        let content = Element::new(self.row_element());
        tree.diff_children(std::slice::from_ref(&content));
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            if let Some(tab_tree) = tree.children.get_mut(0) {
                let row = self.row_element();
                let mut element = Element::new(row);
                tab_tree.diff(element.as_widget_mut());
                element
                    .as_widget_mut()
                    .operate(tab_tree, layout, renderer, operation);
            }
        });
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let content_state = state.state.downcast_mut::<TabBarContentState>();
        content_state.tab_statuses.clone_from(&self.tab_statuses);

        let mut element = Element::new(self.row_element());
        let tab_tree = ensure_child_tree(&mut state.children, &mut element);

        element.as_widget_mut().update(
            tab_tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );

        let tab_layouts: Vec<_> = layout.children().collect();

        let is_currently_dragging = content_state.drag.as_ref().is_some_and(|d| d.is_dragging);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(pos) = cursor.position()
                    && !shell.is_event_captured()
                    && layout.bounds().contains(pos)
                    && let Some(new_selected) =
                        tab_layouts.iter().position(|tl| tl.bounds().contains(pos))
                {
                    let tab_layout = &tab_layouts[new_selected];

                    let is_close_click = if let Some(on_close) = self.on_close.as_ref() {
                        let cross_layout = tab_layout
                            .children()
                            .nth(1)
                            .expect("TabBarContent: Layout should have a close layout");
                        if cross_layout.bounds().contains(pos) {
                            shell.publish(on_close(self.tab_indices[new_selected].clone()));
                            shell.capture_event();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !is_close_click {
                        shell.publish((self.on_select)(self.tab_indices[new_selected].clone()));
                        shell.capture_event();

                        if self.on_reorder.is_some() {
                            let tab_bounds = tab_layout.bounds();
                            content_state.drag = Some(DragState {
                                tab_index: new_selected,
                                press_origin: pos,
                                current_pos: pos,
                                is_dragging: false,
                                tab_offset_x: pos.x - tab_bounds.x,
                                tab_offset_y: pos.y - tab_bounds.y,
                                tab_size: Size::ZERO,
                                overlay_pos: Point::new(0.0, 0.0),
                            });
                        }
                    }
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(drag) = content_state.drag.as_mut()
                    && let Some(pos) = cursor.position()
                {
                    drag.current_pos = pos;
                    if !drag.is_dragging {
                        let dx = pos.x - drag.press_origin.x;
                        let dy = pos.y - drag.press_origin.y;
                        if dx * dx + dy * dy >= self.drag_threshold * self.drag_threshold {
                            drag.is_dragging = true;
                            if let Some(tl) = tab_layouts.get(drag.tab_index) {
                                let b = tl.bounds();
                                drag.tab_size = Size::new(b.width, b.height);
                            }
                        }
                    }
                    if drag.is_dragging {
                        shell.request_redraw();
                        shell.capture_event();
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if let Some(drag) = content_state.drag.take()
                    && drag.is_dragging
                {
                    if let Some(on_reorder) = self.on_reorder.as_ref() {
                        let target =
                            compute_drop_index(&tab_layouts, drag.current_pos.x, drag.tab_index);
                        if target != drag.tab_index {
                            shell.publish(on_reorder(drag.tab_index, target));
                        }
                    }
                    shell.request_redraw();
                    shell.capture_event();
                }
            }

            _ => {}
        }

        let mut request_redraw = false;
        let mut hovered_tab_with_tooltip: Option<(usize, Point)> = None;

        for ((i, _tab), tab_layout) in self.tab_labels.iter().enumerate().zip(&tab_layouts) {
            let active_idx = self.active_tab;
            let tab_status = content_state
                .tab_statuses
                .get_mut(i)
                .expect("Should have a status.");

            let current_status = if is_currently_dragging
                && content_state
                    .drag
                    .as_ref()
                    .is_some_and(|d| d.tab_index == i)
            {
                Status::Dragging
            } else if i == active_idx {
                Status::Active
            } else if cursor.is_over(tab_layout.bounds()) && !is_currently_dragging {
                Status::Hovered
            } else {
                Status::Inactive
            };

            // Track which tab with a tooltip is being hovered.
            if !is_currently_dragging
                && cursor.is_over(tab_layout.bounds())
                && self.tab_tooltips.get(i).is_some_and(|t| t.is_some())
            {
                if let Some(pos) = cursor.position() {
                    hovered_tab_with_tooltip = Some((i, pos));
                }
            }

            let mut is_cross_hovered = None;
            if self.has_close && !is_currently_dragging {
                let mut tab_children = tab_layout.children();
                if let Some(cross_layout) = tab_children.next_back() {
                    is_cross_hovered = Some(cursor.is_over(cross_layout.bounds()));
                }
            }

            if (tab_status.0 != Some(current_status)) || tab_status.1 != is_cross_hovered {
                *tab_status = (Some(current_status), is_cross_hovered);
                request_redraw = true;
            }
        }

        // Update tooltip hover tracking.
        match (&mut content_state.tooltip, hovered_tab_with_tooltip) {
            (Some(ts), Some((idx, pos))) if ts.tab_index == idx => {
                ts.cursor_pos = pos;
                if ts.hover_start.elapsed() < self.tooltip_delay {
                    request_redraw = true;
                }
            }
            (_, Some((idx, pos))) => {
                // Started hovering a new tab with a tooltip.
                content_state.tooltip = Some(TooltipState {
                    tab_index: idx,
                    hover_start: Instant::now(),
                    cursor_pos: pos,
                });
                request_redraw = true;
            }
            (Some(_), None) => {
                // Cursor left all tooltip-bearing tabs.
                content_state.tooltip = None;
                request_redraw = true;
            }
            (None, None) => {}
        }

        if request_redraw {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let content_state = state.state.downcast_ref::<TabBarContentState>();

        if content_state.drag.as_ref().is_some_and(|d| d.is_dragging) {
            return mouse::Interaction::Grabbing;
        }

        mouse::Interaction::default()
    }
}

/// Compute the target insertion index for a drag operation.
///
/// Compares the cursor's x position against each tab layout's center-x.
/// Returns the index where the dragged tab should be placed.
fn compute_drop_index(tab_layouts: &[Layout<'_>], cursor_x: f32, dragged_index: usize) -> usize {
    let count = tab_layouts.len();
    if count == 0 {
        return 0;
    }

    let mut target = count;
    for (i, tab_layout) in tab_layouts.iter().enumerate() {
        let center = tab_layout.bounds().center_x();
        if cursor_x < center {
            target = i;
            break;
        }
    }

    // When dragging right, the visual slot shifts because we remove from the left.
    // Adjust so "dropping in the same place" doesn't move.
    if target > dragged_index {
        target = target.saturating_sub(1);
    }

    target
}

/// Bundles the common parameters shared across all `draw_tab` calls within a
/// single `Tab::draw` invocation, avoiding repetitive argument lists.
struct DrawCtx<'a, 'b, Theme: Catalog> {
    position: Position,
    theme: &'a Theme,
    class: &'a <Theme as Catalog>::Class<'b>,
    icon_data: (Font, f32),
    text_data: (Font, f32),
    close_size: f32,
    viewport: &'a Rectangle,
}

#[allow(clippy::too_many_lines)]
fn draw_tab<Theme, Renderer>(
    renderer: &mut Renderer,
    tab: &TabLabel,
    tab_status: &(Option<Status>, Option<bool>),
    layout: Layout<'_>,
    ctx: &DrawCtx<'_, '_, Theme>,
) where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font> + svg::Renderer,
    Theme: Catalog + text::Catalog,
{
    use iced::advanced::widget::text::{LineHeight, Wrapping};
    use iced::{Background, Border, Color};

    fn child_bounds(item: Option<Layout<'_>>) -> Rectangle {
        item.expect("Graphics: Layout should have a child layout")
            .bounds()
    }

    let bounds = layout.bounds();

    let style = Catalog::style(
        ctx.theme,
        ctx.class,
        tab_status.0.unwrap_or(Status::Inactive),
    );

    let mut children = layout.children();
    let label_layout = children
        .next()
        .expect("Graphics: Layout should have a label layout");
    let mut label_layout_children = label_layout.children();

    if bounds.intersects(ctx.viewport) {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: style.tab.border_radius,
                    width: style.tab.border_width,
                    color: style.tab.border_color,
                },
                shadow: style.tab.shadow,
                ..renderer::Quad::default()
            },
            style.tab.background,
        );
    }

    match tab {
        TabLabel::Icon(icon) => {
            let icon_bounds = child_bounds(label_layout_children.next());

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: icon.to_string(),
                    bounds: Size::new(icon_bounds.width, icon_bounds.height),
                    size: Pixels(ctx.icon_data.1),
                    font: ctx.icon_data.0,
                    align_x: text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: LineHeight::Relative(1.3),
                    shaping: text::Shaping::Advanced,
                    wrapping: Wrapping::default(),
                },
                Point::new(icon_bounds.center_x(), icon_bounds.center_y()),
                style.tab.icon_color,
                icon_bounds,
            );
        }

        TabLabel::Text(text) => {
            let text_bounds = child_bounds(label_layout_children.next());

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: text.clone(),
                    bounds: Size::new(text_bounds.width, text_bounds.height),
                    size: Pixels(ctx.text_data.1),
                    font: ctx.text_data.0,
                    align_x: text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: LineHeight::Relative(1.3),
                    shaping: text::Shaping::Advanced,
                    wrapping: Wrapping::default(),
                },
                Point::new(text_bounds.center_x(), text_bounds.center_y()),
                style.tab.text_color,
                text_bounds,
            );
        }
        TabLabel::IconText(icon, text) => {
            let mut inner_children = label_layout_children
                .next()
                .expect("Graphics: Layout should have an inner layout for IconText")
                .children();
            let first = child_bounds(inner_children.next());
            let second = child_bounds(inner_children.next());
            let (icon_bounds, text_bounds) = if ctx.position.is_icon_first() {
                (first, second)
            } else {
                (second, first)
            };

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: icon.to_string(),
                    bounds: Size::new(icon_bounds.width, icon_bounds.height),
                    size: Pixels(ctx.icon_data.1),
                    font: ctx.icon_data.0,
                    align_x: text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: LineHeight::Relative(1.3),
                    shaping: text::Shaping::Advanced,
                    wrapping: Wrapping::default(),
                },
                Point::new(icon_bounds.center_x(), icon_bounds.center_y()),
                style.tab.icon_color,
                icon_bounds,
            );

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: text.clone(),
                    bounds: Size::new(text_bounds.width, text_bounds.height),
                    size: Pixels(ctx.text_data.1),
                    font: ctx.text_data.0,
                    align_x: text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: LineHeight::Relative(1.3),
                    shaping: text::Shaping::Advanced,
                    wrapping: Wrapping::default(),
                },
                Point::new(text_bounds.center_x(), text_bounds.center_y()),
                style.tab.text_color,
                text_bounds,
            );
        }
    }

    if let Some(cross_layout) = children.next() {
        let cross_bounds = cross_layout.bounds();
        let is_mouse_over_cross = tab_status.1.unwrap_or(false);

        let handle = CLOSE_SVG_HANDLE.clone();
        let svg_size = ctx.close_size + if is_mouse_over_cross { 1.0 } else { 0.0 };
        let svg_bounds = Rectangle {
            x: cross_bounds.center_x() - svg_size / 2.0,
            y: cross_bounds.center_y() - svg_size / 2.0,
            width: svg_size,
            height: svg_size,
        };
        renderer.draw_svg(
            svg::Svg::new(handle).color(style.tab.text_color),
            svg_bounds,
            cross_bounds,
        );

        if is_mouse_over_cross && cross_bounds.intersects(ctx.viewport) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: cross_bounds,
                    border: Border {
                        radius: style.tab.icon_border_radius,
                        width: style.bar.border_width,
                        color: style.bar.border_color.unwrap_or(Color::TRANSPARENT),
                    },
                    shadow: style.tab.shadow,
                    ..renderer::Quad::default()
                },
                style
                    .tab
                    .icon_background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }
    }
}

/// A floating tooltip overlay rendered above all other content.
pub(crate) struct TooltipOverlay<'a, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
{
    pub text: &'a str,
    pub position: Point,
    pub style: TooltipStyle,
    pub text_size: f32,
    pub font: Font,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Renderer> TooltipOverlay<'a, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
{
    pub fn new(
        text: &'a str,
        position: Point,
        style: TooltipStyle,
        text_size: f32,
        font: Font,
    ) -> Self {
        Self {
            text,
            position,
            style,
            text_size,
            font,
            _renderer: PhantomData,
        }
    }
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer> for TooltipOverlay<'_, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
{
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> Node {
        use iced::advanced::text::Paragraph;

        let padding = self.style.padding;

        // Measure the tooltip text to determine the node size.
        let paragraph = <Renderer as iced::advanced::text::Renderer>::Paragraph::with_text(
            iced::advanced::text::Text {
                content: self.text,
                bounds: Size::new(bounds.width * 0.5, f32::INFINITY),
                size: Pixels(self.text_size),
                font: self.font,
                align_x: text::Alignment::Left,
                align_y: Vertical::Top,
                line_height: iced::advanced::widget::text::LineHeight::Relative(1.3),
                shaping: text::Shaping::Advanced,
                wrapping: iced::advanced::widget::text::Wrapping::default(),
            },
        );

        let text_size = paragraph.min_bounds();
        let node_width = text_size.width + padding.left + padding.right;
        let node_height = text_size.height + padding.top + padding.bottom;

        let mut x = self.position.x;
        let mut y = self.position.y;

        // Clamp to stay within window bounds.
        if x + node_width > bounds.width {
            x = (bounds.width - node_width).max(0.0);
        }
        if y + node_height > bounds.height {
            // Show above cursor instead.
            y = (self.position.y - node_height - 4.0).max(0.0);
        }

        let mut node = Node::new(Size::new(node_width, node_height));
        node.move_to_mut(Point::new(x, y));
        node
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        use iced::advanced::widget::text::{LineHeight, Wrapping};

        let bounds = layout.bounds();
        let padding = self.style.padding;

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: self.style.border_radius,
                    width: self.style.border_width,
                    color: self.style.border_color,
                },
                ..renderer::Quad::default()
            },
            self.style.background,
        );

        // Draw text.
        let text_bounds = Rectangle {
            x: bounds.x + padding.left,
            y: bounds.y + padding.top,
            width: bounds.width - padding.left - padding.right,
            height: bounds.height - padding.top - padding.bottom,
        };

        renderer.fill_text(
            iced::advanced::text::Text {
                content: self.text.to_string(),
                bounds: Size::new(text_bounds.width, text_bounds.height),
                size: Pixels(self.text_size),
                font: self.font,
                align_x: text::Alignment::Left,
                align_y: Vertical::Center,
                line_height: LineHeight::Relative(1.3),
                shaping: text::Shaping::Advanced,
                wrapping: Wrapping::default(),
            },
            Point::new(text_bounds.x, text_bounds.center_y()),
            self.style.text_color,
            text_bounds,
        );
    }
}

/// A floating overlay that renders the dragged tab above all other content.
///
/// This overlay escapes the scrollable's clip region, ensuring the dragged tab
/// is never clipped. Its Y position is locked to the tab bar's row while X
/// follows the cursor.
pub(crate) struct DragTabOverlay<'a, 'b, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
{
    pub tab_label: TabLabel,
    pub position: Point,
    pub tab_size: Size,
    pub class: &'a <Theme as Catalog>::Class<'b>,
    pub icon_data: (Font, f32),
    pub text_data: (Font, f32),
    pub close_size: f32,
    pub close_spacing: f32,
    pub icon_spacing: f32,
    pub padding: Padding,
    pub tab_width: Option<f32>,
    pub height: Length,
    pub has_close: bool,
    pub icon_position: Position,
    _renderer: PhantomData<Renderer>,
}

impl<'a, 'b, Theme, Renderer> DragTabOverlay<'a, 'b, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tab_label: TabLabel,
        position: Point,
        tab_size: Size,
        class: &'a <Theme as Catalog>::Class<'b>,
        icon_data: (Font, f32),
        text_data: (Font, f32),
        close_size: f32,
        close_spacing: f32,
        icon_spacing: f32,
        padding: Padding,
        tab_width: Option<f32>,
        height: Length,
        has_close: bool,
        icon_position: Position,
    ) -> Self {
        Self {
            tab_label,
            position,
            tab_size,
            class,
            icon_data,
            text_data,
            close_size,
            close_spacing,
            icon_spacing,
            padding,
            tab_width,
            height,
            has_close,
            icon_position,
            _renderer: PhantomData,
        }
    }
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for DragTabOverlay<'_, '_, Theme, Renderer>
where
    Theme: Catalog + text::Catalog + container::Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font> + svg::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let label_row: Row<'_, Message, Theme, Renderer> =
            build_single_tab_row::<Message, Theme, Renderer>(
                &self.tab_label,
                self.icon_data.1,
                self.text_data.1,
                self.close_size,
                self.close_spacing,
                self.icon_spacing,
                self.padding,
                self.tab_width,
                self.height,
                self.has_close,
                self.icon_position,
                Some(self.icon_data.0),
                Some(self.text_data.0),
            );

        let mut element: Element<'_, Message, Theme, Renderer> = label_row.into();
        let mut tree = Tree::new(element.as_widget());

        let limits = Limits::new(Size::ZERO, self.tab_size);
        let mut node = element.as_widget_mut().layout(&mut tree, renderer, &limits);

        // Clamp X so the tab stays within window bounds.
        let x = self.position.x.clamp(0.0, (bounds.width - self.tab_size.width).max(0.0));
        let y = self.position.y;

        node.move_to_mut(Point::new(x, y));
        node
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();
        let ctx = DrawCtx {
            position: self.icon_position,
            theme,
            class: self.class,
            icon_data: self.icon_data,
            text_data: self.text_data,
            close_size: self.close_size,
            viewport: &viewport,
        };
        let dragged_status = (Some(Status::Dragging), None);
        draw_tab(renderer, &self.tab_label, &dragged_status, layout, &ctx);
    }
}
