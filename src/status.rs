//! Status for widget events.
//!
/// The status of a widget (e.g. for styling).
///
/// Tab bar styling currently uses `Active`, `Hovered`, and `Disabled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Currently selected tab
    Active,
    /// Currently not selected tab
    Inactive,
    /// Can be pressed and it is being hovered.
    Hovered,

}

/// The style function of widget.
pub type StyleFn<'a, Theme, Style> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;
