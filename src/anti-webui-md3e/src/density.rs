//! How closely components are packed: the sizes of Jetpack Compose, which suit touch, or smaller
//! ones for a desktop tool with many controls in a window. See `docs/design/20-md3e-components.md`.

use std::cell::Cell;

/// How big the components are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    /// The sizes of Compose: a text field is 56 high and a touch target 48.
    #[default]
    Standard,
    /// For the pointer: a text field is 36 high, a button 32, and tracks and outlines are thinner.
    Compact,
}

thread_local! {
    static DENSITY: Cell<Density> = const { Cell::new(Density::Standard) };
}

/// The density components take when they are built.
pub fn density() -> Density {
    DENSITY.with(Cell::get)
}

/// Changes the density of what is built from now on on this thread; what is already shown stays
/// as it is. Usually set once, before the first view.
pub fn set_density(density: Density) {
    DENSITY.with(|d| d.set(density));
}

/// Whether components are to be compact.
pub(crate) fn compact() -> bool {
    density() == Density::Compact
}
