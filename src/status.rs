//! Status for widget events.
//!
/// The status of a widget (e.g. for styling).
///
/// Tab bar styling currently uses `Active`, `Hovered`, and `Disabled`.
/// `Pressed`, `Focused`, and `Selected` are reserved for future use (e.g. keyboard focus, selection styling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Can be pressed (e.g. the active tab).
    Active,
    /// Can be pressed and it is being hovered.
    Hovered,
    /// Is being pressed. Reserved for future use.
    Pressed,
    /// Cannot be pressed (inactive tab).
    Disabled,
    /// Is focused. Reserved for future use.
    Focused,
    /// Is selected. Reserved for future use.
    Selected,
}

/// The style function of widget.
pub type StyleFn<'a, Theme, Style> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;
