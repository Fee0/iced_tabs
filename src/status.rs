//! Status for widget events.
//!
/// The status of a widget (e.g. for styling).
///
/// Tab bar styling currently uses `Active`, `Hovered`, and `Disabled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Can be pressed (e.g. the active tab).
    Active,
    /// Can be pressed and it is being hovered.
    Hovered,
    /// Cannot be pressed (inactive tab).
    Disabled,
}

/// The style function of widget.
pub type StyleFn<'a, Theme, Style> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;
