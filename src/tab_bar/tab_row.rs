//! A widget that displays a row of tabs for the [`TabBar`](super::TabBar).
//!
//! *This API requires the following crate features to be activated: `tab_bar`*

use iced::advanced::{
    layout::{Limits, Node},
    renderer,
    widget::{
        text::{LineHeight, Wrapping},
        Operation, Tree,
    },
    Layout, Widget,
};
use iced::widget::{text, Column, Row, Text};
use iced::{
    alignment::{Horizontal, Vertical},
    Alignment, Element, Font, Length, Padding, Pixels, Point, Rectangle, Size,
};

use super::tab_label::TabLabel;
use super::Position;
use crate::status::Status;
use crate::style::Catalog;
use iced_fonts::{codicon::advanced_text, CODICON_FONT};
use std::marker::PhantomData;

/// Offset added to icon/text size during layout to prevent clipping.
const LAYOUT_SIZE_OFFSET: f32 = 1.0;
/// Multiplier for close button hit area (larger than icon for easier clicking).
const CLOSE_HIT_AREA_MULTIPLIER: f32 = 1.3;

/// A row of tabs for the [`TabBar`](super::TabBar).
#[allow(missing_debug_implementations)]
pub struct TabRow<'a, 'b, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: Catalog,
{
    /// The tab labels.
    tab_labels: Vec<TabLabel>,
    /// The tab statuses.
    tab_statuses: Vec<(Option<Status>, Option<bool>)>,
    /// The icon size.
    icon_size: f32,
    /// The text size.
    text_size: f32,
    /// The close icon size.
    close_size: f32,
    /// The padding.
    padding: Padding,
    /// The spacing.
    spacing: Pixels,
    /// The icon font.
    font: Option<Font>,
    /// The text font.
    text_font: Option<Font>,
    /// The height.
    height: Length,
    /// The icon position.
    position: Position,
    /// Whether the close button is shown.
    has_close: bool,
    /// The style class for the tab bar.
    class: &'a <Theme as Catalog>::Class<'b>,
    #[allow(clippy::missing_docs_in_private_items)]
    _message: PhantomData<Message>,
    #[allow(clippy::missing_docs_in_private_items)]
    _renderer: PhantomData<Renderer>,
}

impl<'a, 'b, Message, Theme, Renderer> TabRow<'a, 'b, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog,
{
    /// Creates a new [`TabRow`] with the given tab data.
    #[must_use]
    pub fn new(
        tab_labels: Vec<TabLabel>,
        tab_statuses: Vec<(Option<Status>, Option<bool>)>,
        icon_size: f32,
        text_size: f32,
        close_size: f32,
        padding: Padding,
        spacing: Pixels,
        font: Option<Font>,
        text_font: Option<Font>,
        height: Length,
        position: Position,
        has_close: bool,
        class: &'a <Theme as Catalog>::Class<'b>,
    ) -> Self {
        Self {
            tab_labels,
            tab_statuses,
            icon_size,
            text_size,
            close_size,
            padding,
            spacing,
            font,
            text_font,
            height,
            position,
            has_close,
            class,
            _message: PhantomData,
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
                .shaping(text::Shaping::Advanced)
                .width(Length::Shrink)
        }

        self.tab_labels
            .iter()
            .fold(Row::<Message, Theme, Renderer>::new(), |row, tab_label| {
                let mut label_row = Row::new()
                    .push(
                        match tab_label {
                            TabLabel::Icon(icon) => Column::new().align_x(Alignment::Center).push(
                                layout_icon(icon, self.icon_size + LAYOUT_SIZE_OFFSET, self.font),
                            ),
                            TabLabel::Text(text) => Column::new()
                                .padding(5.0)
                                .align_x(Alignment::Center)
                                .push(layout_text(
                                    text.as_str(),
                                    self.text_size + LAYOUT_SIZE_OFFSET,
                                    self.text_font,
                                )),
                            TabLabel::IconText(icon, text) => {
                                let mut column = Column::new().align_x(Alignment::Center);
                                match self.position {
                                    Position::Top => {
                                        column = column
                                            .push(layout_icon(
                                                icon,
                                                self.icon_size + LAYOUT_SIZE_OFFSET,
                                                self.font,
                                            ))
                                            .push(layout_text(
                                                text.as_str(),
                                                self.text_size + LAYOUT_SIZE_OFFSET,
                                                self.text_font,
                                            ));
                                    }
                                    Position::Right => {
                                        column = column.push(
                                            Row::new()
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
                                                )),
                                        );
                                    }
                                    Position::Left => {
                                        column = column.push(
                                            Row::new()
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
                                                )),
                                        );
                                    }
                                    Position::Bottom => {
                                        column = column
                                            .height(Length::Fill)
                                            .push(layout_text(
                                                text.as_str(),
                                                self.text_size + LAYOUT_SIZE_OFFSET,
                                                self.text_font,
                                            ))
                                            .push(layout_icon(
                                                icon,
                                                self.icon_size + LAYOUT_SIZE_OFFSET,
                                                self.font,
                                            ));
                                    }
                                }
                                column
                            }
                        }
                        .width(Length::Shrink)
                        .height(self.height),
                    )
                    .align_y(Alignment::Center)
                    .padding(self.padding)
                    .width(Length::Shrink);

                if self.has_close {
                    let (close_content, close_font, close_shaping) = advanced_text::close();
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
                                Text::<Theme, Renderer>::new(close_content.to_string())
                                    .size(self.close_size + LAYOUT_SIZE_OFFSET)
                                    .font(close_font)
                                    .align_x(Horizontal::Center)
                                    .align_y(Vertical::Center)
                                    .shaping(close_shaping)
                                    .width(Length::Shrink),
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

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TabRow<'_, '_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog,
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
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((i, tab), tab_layout) in self.tab_labels.iter().enumerate().zip(layout.children()) {
            let tab_status = self.tab_statuses.get(i).expect("Should have a status.");

            draw_tab(
                renderer,
                tab,
                tab_status,
                tab_layout,
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
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(Element::new(self.row_element()))]
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
                // layout is the Row's layout (TabRow's layout() returns Row's layout)
                element
                    .as_widget_mut()
                    .operate(tab_tree, layout, renderer, operation);
            }
        });
    }
}

/// Draws a single tab.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn draw_tab<Theme, Renderer>(
    renderer: &mut Renderer,
    tab: &TabLabel,
    tab_status: &(Option<Status>, Option<bool>),
    layout: Layout<'_>,
    position: Position,
    theme: &Theme,
    class: &<Theme as Catalog>::Class<'_>,
    _cursor: iced::mouse::Cursor,
    icon_data: (Font, f32),
    text_data: (Font, f32),
    close_size: f32,
    viewport: &Rectangle,
) where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + text::Catalog,
{
    use iced::widget::text;
    use iced::{Background, Border, Color, Shadow};

    fn icon_bound_rectangle(item: Option<Layout<'_>>) -> Rectangle {
        item.expect("Graphics: Layout should have an icons layout for an IconText")
            .bounds()
    }

    fn text_bound_rectangle(item: Option<Layout<'_>>) -> Rectangle {
        item.expect("Graphics: Layout should have an texts layout for an IconText")
            .bounds()
    }

    let bounds = layout.bounds();

    let style = Catalog::style(theme, class, tab_status.0.unwrap_or(Status::Disabled));

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
                    radius: style.tab_border_radius,
                    width: style.tab_label_border_width,
                    color: style.tab_label_border_color,
                },
                shadow: Shadow::default(),
                ..renderer::Quad::default()
            },
            style.tab_label_background,
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
                style.icon_color,
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
                style.text_color,
                text_bounds,
            );
        }
        TabLabel::IconText(icon, text) => {
            let icon_bounds: Rectangle;
            let text_bounds: Rectangle;

            match position {
                Position::Top => {
                    icon_bounds = icon_bound_rectangle(label_layout_children.next());
                    text_bounds = text_bound_rectangle(label_layout_children.next());
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
                    text_bounds = text_bound_rectangle(label_layout_children.next());
                    icon_bounds = icon_bound_rectangle(label_layout_children.next());
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
                style.icon_color,
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
                style.text_color,
                text_bounds,
            );
        }
    }

    if let Some(cross_layout) = children.next() {
        let cross_bounds = cross_layout.bounds();
        let is_mouse_over_cross = tab_status.1.unwrap_or(false);

        let (content, font, shaping) = advanced_text::close();

        renderer.fill_text(
            iced::advanced::text::Text {
                content,
                bounds: Size::new(cross_bounds.width, cross_bounds.height),
                size: Pixels(close_size + if is_mouse_over_cross { 1.0 } else { 0.0 }),
                font,
                align_x: text::Alignment::Center,
                align_y: Vertical::Center,
                line_height: LineHeight::Relative(1.3),
                shaping,
                wrapping: Wrapping::default(),
            },
            Point::new(cross_bounds.center_x(), cross_bounds.center_y()),
            style.text_color,
            cross_bounds,
        );

        if is_mouse_over_cross && cross_bounds.intersects(viewport) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: cross_bounds,
                    border: Border {
                        radius: style.icon_border_radius,
                        width: style.border_width,
                        color: style.border_color.unwrap_or(Color::TRANSPARENT),
                    },
                    shadow: Shadow::default(),
                    ..renderer::Quad::default()
                },
                style
                    .icon_background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }
    }
}
