//! The compact density. See `docs/design/20-md3e-components.md`. Each test runs on a thread of its
//! own, and the density belongs to the thread.

mod common;

use anti_webui::prelude::*;
use anti_webui_md3e::components::{
    ButtonStyle, TextFieldStyle, connected_button_group, dropdown_field, icon_button,
    outlined_button, outlined_text_field, slider, switch, tab, tabs,
};
use anti_webui_md3e::{Density, density, icons, set_density};
use common::*;

/// The sizes a text field, a dropdown, a button, a group of buttons, a switch, a slider, an icon
/// button and a row of tabs take.
fn sizes() -> Vec<(f32, f32)> {
    let sizes = signal(vec![(0.0f32, 0.0f32); 8]);
    let at = move |i: usize| move |w: f32, h: f32| sizes.update(|s| s[i] = (w, h));
    let mut ui = start_ui(move || {
        let text = signal("text".to_string());
        let chosen = signal("One".to_string());
        div()
            .flex()
            .flex_col()
            .items_start()
            .child(outlined_text_field(text, TextFieldStyle::new()).on_resized(at(0)))
            .child(
                dropdown_field(chosen, vec!["One".into()], TextFieldStyle::new(), true)
                    .on_resized(at(1)),
            )
            .child(outlined_button("Close").on_resized(at(2)))
            .child(
                connected_button_group(["A", "B"], ButtonStyle::tonal(), |i| i == 0, |_| {})
                    .on_resized(at(3)),
            )
            .child(switch(signal(true)).on_resized(at(4)))
            .child(slider(signal(0.5f32)).on_resized(at(5)))
            .child(icon_button(icons::ADD).on_resized(at(6)))
            .child(tabs(vec![tab("One"), tab("Two")], signal(0usize)).on_resized(at(7)))
    });
    settle(&mut ui);
    sizes.get()
}

#[test]
fn components_are_as_big_as_compose_has_them_by_default() {
    assert_eq!(density(), Density::Standard);
    let s = sizes();
    assert_eq!(s[0].1, 56.0);
    assert_eq!(s[1].1, 56.0);
    assert_eq!(s[2].1, 40.0);
    assert_eq!(s[3].1, 40.0);
    assert_eq!(s[4], (52.0, 48.0));
    assert_eq!(s[5].1, 48.0);
    assert_eq!(s[6], (40.0, 40.0));
    assert_eq!(s[7].1, 48.0);
}

#[test]
fn compact_components_are_smaller() {
    set_density(Density::Compact);
    let s = sizes();
    assert_eq!(s[0].1, 36.0);
    assert_eq!(s[1].1, 36.0);
    assert_eq!(s[2].1, 32.0);
    assert_eq!(s[3].1, 32.0);
    assert_eq!(s[4], (40.0, 28.0));
    assert_eq!(s[5].1, 28.0);
    assert_eq!(s[6], (32.0, 32.0));
    assert_eq!(s[7].1, 36.0);
}

#[test]
fn the_label_of_a_compact_field_floats_on_its_outline() {
    set_density(Density::Compact);
    let value = signal("text".to_string());
    let mut ui = start_ui(move || {
        div().p(20.0).child(outlined_text_field(
            value,
            TextFieldStyle::new().label("Name"),
        ))
    });
    let scene = settle(&mut ui);
    let outline = shaders(&scene).pop().expect("the outline");
    assert_eq!((outline.rect.y, outline.rect.height), (20, 36));
    let label = glyphs(&scene)
        .into_iter()
        .find(|g| g.size == 11.0)
        .expect("the label, small");
    // Its line, 14 high, is across the outline: so is the baseline.
    let baseline = label.glyphs[0].y;
    assert!((baseline - 20.0).abs() <= 7.0, "{baseline}");
    assert!(
        glyphs(&scene).iter().any(|g| g.size == 13.0),
        "what is typed, in the compact body"
    );
}
