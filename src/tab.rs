//! Content widget for [`TabBar`](super::TabBar) (handles selection/close in content-space for Scrollable).

use crate::status::Status;
use crate::style::Catalog;
use crate::tab_bar::Position;
use iced::advanced::{
    layout::{Limits, Node},
    renderer,
    widget::{tree, Operation, Tree},
    Clipboard, Layout, Shell, Widget,
};
use iced::widget::{container, text, Column, Container, Row, Space, Text};
use iced::{
    alignment::{Horizontal, Vertical},
    mouse, touch, Alignment, Element, Event, Font, Length, Padding, Pixels, Point, Rectangle, Size,
};
use iced::advanced::svg;
use iced_fonts::CODICON_FONT;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// Offset added to icon/text size during layout to prevent clipping.
const LAYOUT_SIZE_OFFSET: f32 = 1.0;
/// Multiplier for close button hit area (larger than icon for easier clicking).
const CLOSE_HIT_AREA_MULTIPLIER: f32 = 1.3;
/// SVG bytes for the close (X) icon.
const CLOSE_SVG: &[u8] = include_bytes!("../assets/close.svg");
/// Minimum mouse movement (in pixels) before a press is considered a drag.
const DRAG_THRESHOLD: f32 = 5.0;

/// A [`TabLabel`] showing an icon and/or a text on a tab
/// on a [`TabBar`](super::TabBar).
#[derive(Clone, Hash, Debug)]
pub enum TabLabel {
    /// A [`TabLabel`] showing only an icon on the tab.
    Icon(char),

    /// A [`TabLabel`] showing only a text on the tab.
    Text(String),

    /// A [`TabLabel`] showing an icon and a text on the tab.
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
}

/// State stored in `TabBarContent`'s tree for persisting `tab_statuses`.
#[derive(Debug, Clone, Default)]
pub struct TabBarContentState {
    pub tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    /// Active drag-and-drop state, if any.
    pub drag: Option<DragState>,
}

/// Content widget for the tab bar (handles selection/close in content-space for Scrollable).
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
    label_spacing: f32,
    padding: Padding,
    spacing: Pixels,
    font: Option<Font>,
    text_font: Option<Font>,
    height: Length,
    position: Position,
    has_close: bool,
    on_select: Arc<dyn Fn(TabId) -> Message>,
    on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
    on_reorder: Option<Arc<dyn Fn(usize, usize) -> Message>>,
    active_tab: usize,
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
        label_spacing: f32,
        padding: Padding,
        spacing: Pixels,
        font: Option<Font>,
        text_font: Option<Font>,
        height: Length,
        position: Position,
        has_close: bool,
        active_tab: usize,
        on_select: Arc<dyn Fn(TabId) -> Message>,
        on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
        on_reorder: Option<Arc<dyn Fn(usize, usize) -> Message>>,
        class: &'a <Theme as Catalog>::Class<'b>,
    ) -> Self {
        Self {
            tab_labels,
            tab_statuses,
            tab_indices,
            icon_size,
            text_size,
            close_size,
            label_spacing,
            padding,
            spacing,
            font,
            text_font,
            height,
            position,
            has_close,
            on_select,
            on_close,
            on_reorder,
            active_tab,
            class,
            _renderer: PhantomData,
        }
    }

    fn row_element(&self) -> Row<'_, Message, Theme, Renderer> {
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

        self.tab_labels
            .iter()
            .fold(Row::<Message, Theme, Renderer>::new(), |row, tab_label| {
                let mut label_row = Row::new()
                    .push(
                        match tab_label {
                            TabLabel::Icon(icon) => Container::new(
                                layout_icon(icon, self.icon_size + LAYOUT_SIZE_OFFSET, self.font),
                            )
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center),
                            TabLabel::Text(text) => Container::new(layout_text(
                                text.as_str(),
                                self.text_size + LAYOUT_SIZE_OFFSET,
                                self.text_font,
                            ))
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center),
                            TabLabel::IconText(icon, text) => {
                                let inner: Element<'_, Message, Theme, Renderer> =
                                    match self.position {
                                        Position::Top => Column::new()
                                            .align_x(Alignment::Center)
                                            .push(layout_icon(
                                                icon,
                                                self.icon_size + LAYOUT_SIZE_OFFSET,
                                                self.font,
                                            ))
                                            .push(layout_text(
                                                text.as_str(),
                                                self.text_size + LAYOUT_SIZE_OFFSET,
                                                self.text_font,
                                            ))
                                            .into(),
                                        Position::Right => Row::new()
                                            .align_y(Alignment::Center)
                                            .push(layout_text(
                                                text.as_str(),
                                                self.text_size + LAYOUT_SIZE_OFFSET,
                                                self.text_font,
                                            ))
                                            .push(layout_icon(
                                                icon,
                                                self.icon_size + LAYOUT_SIZE_OFFSET,
                                                self.font,
                                            ))
                                            .into(),
                                        Position::Left => Row::new()
                                            .align_y(Alignment::Center)
                                            .push(layout_icon(
                                                icon,
                                                self.icon_size + LAYOUT_SIZE_OFFSET,
                                                self.font,
                                            ))
                                            .push(layout_text(
                                                text.as_str(),
                                                self.text_size + LAYOUT_SIZE_OFFSET,
                                                self.text_font,
                                            ))
                                            .into(),
                                        Position::Bottom => Column::new()
                                            .align_x(Alignment::Center)
                                            .push(layout_text(
                                                text.as_str(),
                                                self.text_size + LAYOUT_SIZE_OFFSET,
                                                self.text_font,
                                            ))
                                            .push(layout_icon(
                                                icon,
                                                self.icon_size + LAYOUT_SIZE_OFFSET,
                                                self.font,
                                            ))
                                            .into(),
                                    };
                                Container::new(inner)
                                    .align_x(Horizontal::Center)
                                    .align_y(Vertical::Center)
                            }
                        }
                        .width(Length::Shrink)
                        .height(self.height),
                    )
                    .align_y(Alignment::Center)
                    .padding(self.padding)
                    .spacing(self.label_spacing)
                    .width(Length::Shrink);

                if self.has_close {
                    label_row = label_row.push(
                        Row::new()
                            .width(Length::Fixed(
                                self.close_size * CLOSE_HIT_AREA_MULTIPLIER + LAYOUT_SIZE_OFFSET,
                            ))
                            .height(Length::Fixed(
                                self.close_size * CLOSE_HIT_AREA_MULTIPLIER + LAYOUT_SIZE_OFFSET,
                            ))
                            .align_y(Alignment::Center)
                            .push(
                                Space::new()
                                    .width(self.close_size + LAYOUT_SIZE_OFFSET)
                                    .height(self.close_size + LAYOUT_SIZE_OFFSET),
                            ),
                    );
                }

                row.push(label_row)
            })
            .width(Length::Shrink)
            .height(self.height)
            .spacing(self.spacing)
            .align_y(Alignment::Center)
    }
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
        let row = self.row_element();
        let mut element = Element::new(row);
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
            .layout(tab_tree, renderer, &limits.width(Length::Shrink).loose())
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let content_state = state.state.downcast_ref::<TabBarContentState>();
        let drag = content_state.drag.as_ref();
        let is_dragging = drag.is_some_and(|d| d.is_dragging);

        let tab_layouts: Vec<_> = layout.children().collect();

        if !is_dragging {
            // Normal (non-drag) drawing: each tab at its own layout position.
            for ((i, tab), tab_layout) in self.tab_labels.iter().enumerate().zip(&tab_layouts) {
                let tab_status = self.tab_statuses.get(i).expect("Should have a status.");

                draw_tab(
                    renderer,
                    tab,
                    tab_status,
                    *tab_layout,
                    self.position,
                    theme,
                    self.class,
                    cursor,
                    (self.font.unwrap_or(CODICON_FONT), self.icon_size),
                    (self.text_font.unwrap_or_default(), self.text_size),
                    self.close_size,
                    viewport,
                );
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
                let tab_status = self.tab_statuses.get(tab_idx).expect("Should have a status.");

                let original_bounds = tab_layouts[tab_idx].bounds();
                let slot_bounds = tab_layouts[slot].bounds();
                let offset_x = slot_bounds.x - original_bounds.x;

                if offset_x.abs() < 0.5 {
                    draw_tab(
                        renderer,
                        tab,
                        tab_status,
                        tab_layouts[tab_idx],
                        self.position,
                        theme,
                        self.class,
                        cursor,
                        (self.font.unwrap_or(CODICON_FONT), self.icon_size),
                        (self.text_font.unwrap_or_default(), self.text_size),
                        self.close_size,
                        viewport,
                    );
                } else {
                    renderer.with_translation(
                        iced::Vector::new(offset_x, 0.0),
                        |renderer| {
                            draw_tab(
                                renderer,
                                tab,
                                tab_status,
                                tab_layouts[tab_idx],
                                self.position,
                                theme,
                                self.class,
                                cursor,
                                (self.font.unwrap_or(CODICON_FONT), self.icon_size),
                                (self.text_font.unwrap_or_default(), self.text_size),
                                self.close_size,
                                viewport,
                            );
                        },
                    );
                }
            }

            // Draw the dragged tab floating at the cursor position.
            if let Some(dragged_layout) = tab_layouts.get(dragged_idx) {
                let original_bounds = dragged_layout.bounds();
                let offset_x = drag.current_pos.x - drag.tab_offset_x - original_bounds.x;
                let offset_y = drag.current_pos.y - original_bounds.center_y();

                renderer.with_translation(
                    iced::Vector::new(offset_x, offset_y),
                    |renderer| {
                        let dragged_tab = &self.tab_labels[dragged_idx];
                        let dragged_status = (Some(Status::Dragging), None);
                        draw_tab(
                            renderer,
                            dragged_tab,
                            &dragged_status,
                            *dragged_layout,
                            self.position,
                            theme,
                            self.class,
                            cursor,
                            (self.font.unwrap_or(CODICON_FONT), self.icon_size),
                            (self.text_font.unwrap_or_default(), self.text_size),
                            self.close_size,
                            viewport,
                        );
                    },
                );
            }
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarContentState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TabBarContentState {
            tab_statuses: self.tab_statuses.clone(),
            drag: None,
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

        let row = self.row_element();
        let mut element = Element::new(row);
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

        let tab_layouts: Vec<_> = layout.children().collect();

        let is_currently_dragging = content_state
            .drag
            .as_ref()
            .is_some_and(|d| d.is_dragging);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if !shell.is_event_captured()
                    && cursor
                        .position()
                        .is_some_and(|pos| layout.bounds().contains(pos))
                {
                    let tabs_map: Vec<bool> = tab_layouts
                        .iter()
                        .map(|tab_layout| {
                            cursor
                                .position()
                                .is_some_and(|pos| tab_layout.bounds().contains(pos))
                        })
                        .collect();

                    if let Some(new_selected) = tabs_map.iter().position(|b| *b) {
                        let tab_layout = tab_layouts.get(new_selected).expect(
                            "TabBarContent: Layout should have a tab layout at the selected index",
                        );

                        let is_close_click = if let Some(on_close) = self.on_close.as_ref() {
                            let cross_layout = tab_layout
                                .children()
                                .nth(1)
                                .expect("TabBarContent: Layout should have a close layout");
                            if cursor
                                .position()
                                .is_some_and(|pos| cross_layout.bounds().contains(pos))
                            {
                                shell.publish(on_close(
                                    self.tab_indices[new_selected].clone(),
                                ));
                                shell.capture_event();
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if !is_close_click {
                            shell.publish((self.on_select)(
                                self.tab_indices[new_selected].clone(),
                            ));
                            shell.capture_event();

                            if self.on_reorder.is_some() {
                                if let Some(pos) = cursor.position() {
                                    let tab_bounds = tab_layout.bounds();
                                    content_state.drag = Some(DragState {
                                        tab_index: new_selected,
                                        press_origin: pos,
                                        current_pos: pos,
                                        is_dragging: false,
                                        tab_offset_x: pos.x - tab_bounds.x,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(drag) = content_state.drag.as_mut() {
                    if let Some(pos) = cursor.position() {
                        drag.current_pos = pos;
                        if !drag.is_dragging {
                            let dx = pos.x - drag.press_origin.x;
                            let dy = pos.y - drag.press_origin.y;
                            if (dx * dx + dy * dy).sqrt() >= DRAG_THRESHOLD {
                                drag.is_dragging = true;
                            }
                        }
                        if drag.is_dragging {
                            shell.request_redraw();
                            shell.capture_event();
                        }
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if let Some(drag) = content_state.drag.take() {
                    if drag.is_dragging {
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
            }

            _ => {}
        }

        let mut request_redraw = false;
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

            let mut is_cross_hovered = None;
            if self.has_close && !is_currently_dragging {
                let mut tab_children = tab_layout.children();
                if let Some(cross_layout) = tab_children.next_back() {
                    is_cross_hovered = Some(cursor.is_over(cross_layout.bounds()));
                }
            }

            if tab_status.0.is_some_and(|status| status != current_status)
                || tab_status.1 != is_cross_hovered
            {
                *tab_status = (Some(current_status), is_cross_hovered);
                request_redraw = true;
            }
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

        if content_state
            .drag
            .as_ref()
            .is_some_and(|d| d.is_dragging)
        {
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

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn draw_tab<Theme, Renderer>(
    renderer: &mut Renderer,
    tab: &TabLabel,
    tab_status: &(Option<Status>, Option<bool>),
    layout: Layout<'_>,
    position: Position,
    theme: &Theme,
    class: &<Theme as Catalog>::Class<'_>,
    _cursor: mouse::Cursor,
    icon_data: (Font, f32),
    text_data: (Font, f32),
    close_size: f32,
    viewport: &Rectangle,
) where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font> + svg::Renderer,
    Theme: Catalog + text::Catalog,
{
    use iced::advanced::widget::text::{LineHeight, Wrapping};
    use iced::{Background, Border, Color};

    fn icon_bound_rectangle(item: Option<Layout<'_>>) -> Rectangle {
        item.expect("Graphics: Layout should have an icons layout for an IconText")
            .bounds()
    }

    fn text_bound_rectangle(item: Option<Layout<'_>>) -> Rectangle {
        item.expect("Graphics: Layout should have an texts layout for an IconText")
            .bounds()
    }

    let bounds = layout.bounds();

    let style = Catalog::style(theme, class, tab_status.0.unwrap_or(Status::Inactive));

    let mut children = layout.children();
    let label_layout = children
        .next()
        .expect("Graphics: Layout should have a label layout");
    let mut label_layout_children = label_layout.children();

    if bounds.intersects(viewport) {
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
            let icon_bounds = icon_bound_rectangle(label_layout_children.next());

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: icon.to_string(),
                    bounds: Size::new(icon_bounds.width, icon_bounds.height),
                    size: Pixels(icon_data.1),
                    font: icon_data.0,
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
            let text_bounds = text_bound_rectangle(label_layout_children.next());

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: text.clone(),
                    bounds: Size::new(text_bounds.width, text_bounds.height),
                    size: Pixels(text_data.1),
                    font: text_data.0,
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
            let icon_bounds: Rectangle;
            let text_bounds: Rectangle;

            match position {
                Position::Top => {
                    let mut inner_children = label_layout_children
                        .next()
                        .expect("Graphics: Top Layout should have an inner layout")
                        .children();
                    icon_bounds = icon_bound_rectangle(inner_children.next());
                    text_bounds = text_bound_rectangle(inner_children.next());
                }
                Position::Right => {
                    let mut row_children = label_layout_children
                        .next()
                        .expect("Graphics: Right Layout should have a row with one child")
                        .children();
                    text_bounds = text_bound_rectangle(row_children.next());
                    icon_bounds = icon_bound_rectangle(row_children.next());
                }
                Position::Left => {
                    let mut row_children = label_layout_children
                        .next()
                        .expect("Graphics: Left Layout should have a row with one child")
                        .children();
                    icon_bounds = icon_bound_rectangle(row_children.next());
                    text_bounds = text_bound_rectangle(row_children.next());
                }
                Position::Bottom => {
                    let mut inner_children = label_layout_children
                        .next()
                        .expect("Graphics: Bottom Layout should have an inner layout")
                        .children();
                    text_bounds = text_bound_rectangle(inner_children.next());
                    icon_bounds = icon_bound_rectangle(inner_children.next());
                }
            }

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: icon.to_string(),
                    bounds: Size::new(icon_bounds.width, icon_bounds.height),
                    size: Pixels(icon_data.1),
                    font: icon_data.0,
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
                    size: Pixels(text_data.1),
                    font: text_data.0,
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

        let handle = svg::Handle::from_memory(CLOSE_SVG);
        let svg_size = close_size + if is_mouse_over_cross { 1.0 } else { 0.0 };
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

        if is_mouse_over_cross && cross_bounds.intersects(viewport) {
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
