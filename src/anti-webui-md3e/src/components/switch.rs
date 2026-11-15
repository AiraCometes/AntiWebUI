//! Switches. Sizes, colors and timing are those of Jetpack Compose. See
//! `docs/design/20-md3e-components.md`.

use super::icon::sized_icon;
use super::surface::{Ink, When, focus_ring, halo, when};
use crate::color::color_scheme;
use crate::icons::Symbol;
use crate::motion::{fast_effects, fast_spatial};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// The sizes of a switch of a density.
#[derive(Clone, Copy)]
struct Metrics {
    width: f32,
    height: f32,
    touch: f32,
    outline: f32,
    halo: f32,
    icon: f32,
    /// Of the handle: off without an icon, otherwise, and pressed.
    handles: (f32, f32, f32),
    /// Between the handle and the end of the track, at rest and pressed.
    ends: (f32, f32),
}

impl Metrics {
    fn now() -> Self {
        if crate::density::compact() {
            Metrics {
                width: 40.0,
                height: 24.0,
                touch: 28.0,
                outline: 1.5,
                halo: 32.0,
                icon: 12.0,
                handles: (12.0, 16.0, 20.0),
                ends: (4.0, 2.0),
            }
        } else {
            Metrics {
                width: 52.0,
                height: 32.0,
                touch: 48.0,
                outline: 2.0,
                halo: 40.0,
                icon: 16.0,
                handles: (16.0, 24.0, 28.0),
                ends: (4.0, 2.0),
            }
        }
    }
}

/// How a switch looks.
#[derive(Clone, Default)]
pub struct SwitchStyle {
    on_icon: Option<&'static Symbol>,
    off_icon: Option<&'static Symbol>,
    disabled: Option<When>,
}

impl SwitchStyle {
    pub fn new() -> Self {
        Self::default()
    }

    /// The icon on the handle while it is on.
    pub fn on_icon(mut self, symbol: &'static Symbol) -> Self {
        self.on_icon = Some(symbol);
        self
    }

    /// The icon on the handle while it is off, which makes the handle as big as when it is on.
    pub fn off_icon(mut self, symbol: &'static Symbol) -> Self {
        self.off_icon = Some(symbol);
        self
    }

    /// Doesn't react and looks that way while `f` returns true.
    pub fn disabled_when(mut self, f: impl Fn() -> bool + 'static) -> Self {
        self.disabled = Some(Rc::new(f));
        self
    }
}

/// How big the handle is and where its left edge is, measured from the outside of the track.
fn handle(m: &Metrics, checked: bool, pressed: bool, has_icon: bool) -> (f32, f32) {
    let (small, usual, held) = m.handles;
    let size = if pressed {
        held
    } else if has_icon || checked {
        usual
    } else {
        small
    };
    let (rest, pressed_end) = m.ends;
    let x = match (pressed, checked) {
        (true, true) => m.width - pressed_end - size,
        (true, false) => pressed_end,
        (false, true) => m.width - rest - size,
        (false, false) => (m.height - size) / 2.0,
    };
    (size, x)
}

/// A switch bound to `on`, which it flips when it is clicked. The behavior is the standard
/// switch's (`switch_with`).
pub fn switch(on: impl Into<Binding<bool>>) -> Div {
    styled_switch(on, SwitchStyle::new())
}

pub fn styled_switch(on: impl Into<Binding<bool>>, style: SwitchStyle) -> Div {
    switch_with(on, move |state| look(state, style))
}

fn look(state: SwitchState, style: SwitchStyle) -> Div {
    let m = Metrics::now();
    let Metrics {
        width,
        height,
        touch,
        outline,
        halo: halo_size,
        icon,
        ..
    } = m;
    let checked: When = Rc::new(move || state.on());
    let SwitchStyle {
        on_icon,
        off_icon,
        disabled,
    } = style;
    let disabled = when(disabled);
    let layer_checked = checked.clone();
    let ink = Ink::new_when(
        move |c| {
            if layer_checked() {
                c.primary
            } else {
                c.on_surface
            }
        },
        disabled.clone(),
    );
    let Ink {
        focused, pressed, ..
    } = ink;

    let shape_checked = checked.clone();
    let wanted = move || {
        let on = shape_checked();
        let has_icon = if on {
            on_icon.is_some()
        } else {
            off_icon.is_some()
        };
        handle(&m, on, pressed.get(), has_icon)
    };
    let (size_at_start, x_at_start) = untrack(&wanted);
    let size = animated(size_at_start);
    let x = animated(x_at_start);
    let last = Cell::new((size_at_start, x_at_start));
    effect(move || {
        let (to_size, to_x) = wanted();
        if last.replace((to_size, to_x)) != (to_size, to_x) {
            // Pressing is about as fast as Compose's snap; the rest may bounce.
            let spring = if pressed.get_untracked() {
                fast_effects()
            } else {
                fast_spatial()
            };
            untrack(|| {
                size.spring_to(to_size, spring);
                x.spring_to(to_x, spring);
            });
        }
    });

    let track_checked = checked.clone();
    let track_off = disabled.clone();
    let thumb_checked = checked.clone();
    let thumb_off = disabled.clone();
    let mark =
        move |symbol: Option<&'static Symbol>, shown_when_on: bool, checked: When, off: When| {
            symbol.map(move |symbol| {
                sized_icon(symbol, icon)
                    .absolute()
                    .transition_of([Prop::Opacity], Transition::ms(100).ease_out())
                    .dyn_style(move |s| {
                        let c = color_scheme();
                        let on = checked();
                        let color = match (off(), on) {
                            (false, true) => c.on_primary_container,
                            (false, false) => c.surface_container_highest,
                            (true, true) => c.on_surface.with_alpha(0.38),
                            (true, false) => c.surface_container_highest.with_alpha(0.38),
                        };
                        let inset = (size.get() - icon) / 2.0;
                        let s = s.text_color(color).left(inset).top(inset);
                        if on == shown_when_on {
                            s
                        } else {
                            s.opacity(0.0)
                        }
                    })
            })
        };
    let on_mark = mark(on_icon, true, checked.clone(), disabled.clone());
    let off_mark = mark(off_icon, false, checked.clone(), disabled.clone());

    let track = div()
        .relative()
        .w(width)
        .h(height)
        .rounded(height / 2.0)
        .border(outline)
        .transition_of(
            [Prop::Background, Prop::BorderColor],
            Transition::ms(100).ease_out(),
        )
        .dyn_style(move |s| {
            let c = color_scheme();
            let on = track_checked();
            let (background, border) = match (track_off(), on) {
                (false, true) => (c.primary, c.primary),
                (false, false) => (c.surface_container_highest, c.outline),
                (true, true) => (
                    mix_premultiplied(c.surface, c.on_surface, 0.12),
                    mix_premultiplied(c.surface, c.on_surface, 0.12),
                ),
                (true, false) => (
                    mix_premultiplied(c.surface, c.surface_container_highest, 0.12),
                    mix_premultiplied(c.surface, c.on_surface, 0.12),
                ),
            };
            focus_ring(s.bg(background).border_color(border), focused.get(), &c)
        })
        .child(halo(&ink, halo_size).dyn_style(move |s| {
            // The inside of the border is where this is measured from.
            s.left(x.get() + size.get() / 2.0 - halo_size / 2.0 - outline)
                .top(height / 2.0 - halo_size / 2.0 - outline)
        }))
        .child(
            div()
                .absolute()
                .flex()
                .items_center()
                .justify_center()
                .transition_of([Prop::Background], Transition::ms(100).ease_out())
                .dyn_style(move |s| {
                    let c = color_scheme();
                    let on = thumb_checked();
                    let background = match (thumb_off(), on) {
                        (false, true) => c.on_primary,
                        (false, false) => c.outline,
                        (true, true) => c.surface,
                        (true, false) => mix_premultiplied(c.surface, c.on_surface, 0.38),
                    };
                    let (size, x) = (size.get(), x.get());
                    s.left(x - outline)
                        .top((height - size) / 2.0 - outline)
                        .w(size)
                        .h(size)
                        .rounded(size / 2.0)
                        .bg(background)
                })
                .children(on_mark)
                .children(off_mark),
        );
    let root_off = disabled.clone();
    ink.listen_centered(
        div()
            .flex()
            .flex_none()
            .items_center()
            .w(width)
            .h(touch)
            .dyn_style(move |s| {
                s.cursor(if root_off() {
                    CursorIcon::Default
                } else {
                    CursorIcon::Pointer
                })
            }),
    )
    .disabled(move || disabled())
    .child(track)
}
