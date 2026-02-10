//! Content widget for [`TabBar`](super::TabBar) that handles clicks with correct cursor coordinates.
//!
//! When used as the content of a [`Scrollable`], this widget receives the
//! cursor in content-space (transformed by scroll offset), ensuring correct
//! tab selection and close button hit testing.
//!
//! *This API requires the following crate features to be activated: `tab_bar`*

use iced::{
    Element, Event, Font, Length, Padding, Pixels, Rectangle, Size,
    mouse::{self, Cursor},
    touch,
};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget,
    layout::{Limits, Node},
    renderer,
    widget::{Operation, Tree, tree},
};
use std::marker::PhantomData;
use std::sync::Arc;
use crate::status::Status;
use crate::style::Catalog;
use super::Position;
use super::tab_label::TabLabel;
use super::tab_row::TabRow;

/// State stored in `TabBarContent`'s tree for persisting `tab_statuses`.
#[derive(Debug, Clone, Default)]
pub(crate) struct TabBarContentState {
    pub(crate) tab_statuses: Vec<(Option<Status>, Option<bool>)>,
}

/// Content widget for the tab bar that handles selection and close with correct
/// hit testing (works with scrollable content).
#[allow(missing_debug_implementations)]
pub struct TabBarContent<
    'a,
    'b,
    Message,
    TabId,
    Theme = iced::Theme,
    Renderer = iced::Renderer,
> where
    Renderer: iced::advanced::renderer::Renderer + iced::advanced::text::Renderer,
    Theme: Catalog,
    TabId: Eq + Clone,
{
    tab_labels: Vec<TabLabel>,
    tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    tab_indices: Vec<TabId>,
    icon_size: f32,
    text_size: f32,
    close_size: f32,
    padding: Padding,
    spacing: Pixels,
    font: Option<Font>,
    text_font: Option<Font>,
    height: Length,
    tab_width: Length,
    position: Position,
    has_close: bool,
    on_select: Arc<dyn Fn(TabId) -> Message>,
    on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
    active_tab: usize,
    class: &'a <Theme as Catalog>::Class<'b>,
    #[allow(clippy::missing_docs_in_private_items)]
    _renderer: PhantomData<Renderer>,
}

impl<'a, 'b, Message, TabId, Theme, Renderer> TabBarContent<'a, 'b, Message, TabId, Theme, Renderer>
where
    Renderer: iced::advanced::renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + iced::widget::text::Catalog,
    TabId: Eq + Clone,
{
    /// Creates a new [`TabBarContent`] with the given parameters.
    #[must_use]
    pub fn new(
        tab_labels: Vec<TabLabel>,
        tab_statuses: Vec<(Option<Status>, Option<bool>)>,
        tab_indices: Vec<TabId>,
        icon_size: f32,
        text_size: f32,
        close_size: f32,
        padding: Padding,
        spacing: Pixels,
        font: Option<Font>,
        text_font: Option<Font>,
        height: Length,
        tab_width: Length,
        position: Position,
        has_close: bool,
        active_tab: usize,
        on_select: Arc<dyn Fn(TabId) -> Message>,
        on_close: Option<Arc<dyn Fn(TabId) -> Message>>,
        class: &'a <Theme as Catalog>::Class<'b>,
    ) -> Self {
        Self {
            tab_labels,
            tab_statuses,
            tab_indices,
            icon_size,
            text_size,
            close_size,
            padding,
            spacing,
            font,
            text_font,
            height,
            tab_width,
            position,
            has_close,
            on_select,
            on_close,
            active_tab,
            class,
            _renderer: PhantomData,
        }
    }

    fn tab_row(&self) -> TabRow<'a, 'b, Message, Theme, Renderer> {
        TabRow::new(
            self.tab_labels.clone(),
            self.tab_statuses.clone(),
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
            self.has_close,
            self.class,
        )
    }
}

impl<Message, TabId, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TabBarContent<'_, '_, Message, TabId, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + iced::widget::text::Catalog,
    TabId: Eq + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarContentState>()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(Element::new(self.tab_row()))]
    }

    fn state(&self) -> tree::State {
        tree::State::new(TabBarContentState {
            tab_statuses: self.tab_statuses.clone(),
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let content = Element::new(self.tab_row());
        tree.diff_children(std::slice::from_ref(&content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.tab_width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let tab_row = self.tab_row();
        let mut element = Element::new(tab_row);
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
            .layout(tab_tree, renderer, &limits.width(self.tab_width).loose())
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
                let tab_row = self.tab_row();
                let mut element = Element::new(tab_row);
                tab_tree.diff(element.as_widget_mut());
                // layout is the Row's layout (TabBarContent's layout returns Row's layout)
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
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // Sync tab_statuses from TabBar when we have fresh data (e.g. active_tab changed)
        let content_state = state.state.downcast_mut::<TabBarContentState>();
        content_state.tab_statuses.clone_from(&self.tab_statuses);

        // Delegate to TabRow (for any child widget updates)
        let tab_row: TabRow<'_, '_, Message, Theme, Renderer> = TabRow::new(
            self.tab_labels.clone(),
            content_state.tab_statuses.clone(),
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
            self.has_close,
            self.class,
        );
        let mut element = Element::new(tab_row);
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

        // Tab layouts: layout is TabRow's Row layout; layout.children() = tab layouts (empty when no tabs)
        let tab_layouts: Vec<_> = layout.children().collect();

        // Handle clicks - cursor is already in content space when we're inside Scrollable
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
                        let message = if let Some(on_close) = self.on_close.as_ref() {
                            let cross_layout = tab_layout
                                .children()
                                .nth(1)
                                .expect("TabBarContent: Layout should have a close layout");
                            if cursor
                                .position()
                                .is_some_and(|pos| cross_layout.bounds().contains(pos))
                            {
                                on_close(self.tab_indices[new_selected].clone())
                            } else {
                                (self.on_select)(self.tab_indices[new_selected].clone())
                            }
                        } else {
                            (self.on_select)(self.tab_indices[new_selected].clone())
                        };
                        shell.publish(message);
                        shell.capture_event();
                    }
                }
            }
            _ => {}
        }

        // Update tab statuses for hover/active (stored in tree state for TabBar to read)
        let mut request_redraw = false;
        for ((i, _tab), tab_layout) in self.tab_labels.iter().enumerate().zip(&tab_layouts) {
            let active_idx = self.active_tab;
            let tab_status = content_state
                .tab_statuses
                .get_mut(i)
                .expect("Should have a status.");

            let current_status = if cursor.is_over(tab_layout.bounds()) {
                Status::Hovered
            } else if i == active_idx {
                Status::Active
            } else {
                Status::Disabled
            };

            let mut is_cross_hovered = None;
            if self.has_close {
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
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let tab_row = self.tab_row();
        let element = Element::new(tab_row);
        let tab_tree = state
            .children
            .first()
            .expect("TabBarContent: Should have TabRow tree");

        element
            .as_widget()
            .mouse_interaction(tab_tree, layout, cursor, viewport, renderer)
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
        let tab_row = self.tab_row();
        let element = Element::new(tab_row);
        let tab_tree = state
            .children
            .first()
            .expect("TabBarContent: Should have TabRow tree");

        element
            .as_widget()
            .draw(tab_tree, renderer, theme, style, layout, cursor, viewport);
    }
}
