# iced_tabs

A feature-rich tab bar widget for [iced](https://github.com/iced-rs/iced) GUI applications, based
on [iced_aw](https://github.com/iced-rs/iced_aw)'s tab bar with additional features.

[![Watch the video](https://raw.githubusercontent.com/Fee0/iced_tabs/main/assets/thumbnail.png)](https://raw.githubusercontent.com/Fee0/iced_tabs/main/assets/video.mp4)

## Features

- **Drag-and-drop reordering** -- rearrange tabs by dragging them (configurable drag threshold)
- **Three tab label types** -- `Text`, `Icon`, or `IconText` (icon + text combined)
- **Close buttons** -- optional per-tab close button with customizable size and spacing
- **Tooltips** -- hover tooltips with configurable delay
- **Scrolling** -- mouse wheel scrolling and an optional scrollbar (floating, below, or hidden)
- **Fully styleable** -- theme-aware styling via iced's `Catalog` pattern with status-based variants (active, inactive,
  hovered, dragging)

## Compatibility

| iced_tabs | iced |
|-----------|------|
| 0.1       | 0.14 |

## Installation

Add `iced_tabs` to your `Cargo.toml`:

```toml
[dependencies]
iced_tabs = { git = "https://github.com/Fee0/iced_tabs" }
```

If you want to use icon tabs, also add [iced_fonts](https://github.com/iced-rs/iced_fonts) (or supply your own icon
font):

```toml
iced_fonts = { version = "0.3", features = ["codicon"] }
```

## Quick start

```rust
use std::time::Duration;
use iced_tabs::{TabBar, TabLabel, Position, ScrollMode};

#[derive(Debug, Clone)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
    TabReordered(usize, usize),
}

fn view(active: &usize) -> TabBar<'_, Message, usize> {
    TabBar::new(Message::TabSelected)
        .push(0, TabLabel::Text("Home".into()))
        .push(1, TabLabel::Text("Settings".into()))
        .push_with_tooltip(2, TabLabel::Text("Help".into()), "Open help".into())
        .set_active_tab(active)
        .on_close(Message::TabClosed)
        .on_reorder(Message::TabReordered)
        .spacing(8.0)
        .padding(8.0)
        .text_size(16.0)
        .height(35.0)
        .scroll_mode(ScrollMode::Floating)
        .tooltip_delay(Duration::from_millis(500))
}
```

## API overview

### `TabBar`

The main widget. Created with `TabBar::new(on_select)` where `on_select` is a closure that produces a message when a tab
is clicked.

| Method                                   | Description                                                        |
|------------------------------------------|--------------------------------------------------------------------|
| `push(id, label)`                        | Add a tab                                                          |
| `push_with_tooltip(id, label, tooltip)`  | Add a tab with a hover tooltip                                     |
| `set_active_tab(&id)`                    | Mark a tab as active                                               |
| `on_close(f)`                            | Enable close buttons; `f` receives the closed tab's id             |
| `on_reorder(f)`                          | Enable drag-and-drop reordering; `f` receives `(from, to)` indices |
| `scroll_mode(mode)`                      | Set scroll behaviour (`Floating`, `Below`, `NoScrollbar`)          |
| `set_position(pos)`                      | Icon position relative to text (`Top`, `Right`, `Bottom`, `Left`)  |
| `width` / `height` / `max_height`        | Size constraints                                                   |
| `tab_width(f32)`                         | Fixed width for every tab                                          |
| `text_size` / `icon_size` / `close_size` | Font sizes                                                         |
| `icon_font` / `text_font`                | Custom fonts                                                       |
| `padding` / `spacing`                    | Outer padding and gap between tabs                                 |
| `close_spacing` / `icon_spacing`         | Spacing around close button / icon                                 |
| `drag_threshold(f32)`                    | Minimum pixels before a drag starts (default: 5)                   |
| `tooltip_delay(Duration)`                | Delay before showing tooltips (default: 500 ms)                    |
| `style(f)` / `class(c)`                  | Custom styling                                                     |

### `TabLabel`

Describes what a tab displays:

```rust
TabLabel::Text("Settings".into())
TabLabel::Icon('\u{eb51}')
TabLabel::IconText('\u{eb06}', "Home".into())
```

Convenient `From` implementations are provided:

```rust
let label: TabLabel = "Settings".into();          // Text
let label: TabLabel = '\u{eb51}'.into();           // Icon
let label: TabLabel = ('\u{eb06}', "Home").into(); // IconText
```

### `ScrollMode`

| Variant         | Description                                            |
|-----------------|--------------------------------------------------------|
| `Floating`      | Scrollbar overlays the tab bar when needed             |
| `Below(Pixels)` | Scrollbar sits in its own row below the tabs           |
| `NoScrollbar`   | No visible scrollbar; scroll with the mouse wheel only |

### `Position`

Controls where the icon sits relative to the text in `IconText` tabs:

`Top` | `Right` | `Bottom` | `Left` (default)

## Styling

`iced_tabs` uses iced's `Catalog` trait for theming. A default style (`primary`) is provided that adapts to the built-in
iced `Theme`.

You can supply a custom style function:

```rust
use iced_tabs::{Style, Status, BarStyle, TabStyle, TooltipStyle};

tab_bar.style( | theme, status| {
match status {
Status::Active => Style { /* ... */ },
Status::Inactive => Style { /* ... */ },
Status::Hovered => Style { /* ... */ },
Status::Dragging => Style { /* ... */ },
}
})
```

The `Style` struct is composed of three parts:

- **`BarStyle`** -- background, border, shadow of the outer bar
- **`TabStyle`** -- background, border, text/icon colours, shadow of each tab
- **`TooltipStyle`** -- background, border, text colour, padding of tooltips

## Running the example

```sh
cargo run --example tabs
```

The example app lets you interactively tweak every setting (spacing, sizes, scroll mode, label type, etc.) and see the
result in real time.

## License

MIT
