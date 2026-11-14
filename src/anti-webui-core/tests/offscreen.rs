//! Nodes laid out but shown only when painted on their own (`Document::set_offscreen`,
//! `paint::paint_node`): what a scene in 3D shows on a panel.

use anti_webui_core::Ui;
use anti_webui_core::event::*;
use anti_webui_core::geometry::{Scale, SizeI};
use anti_webui_core::paint::paint_node;
use anti_webui_core::prelude::*;
use anti_webui_core::scene::{Clip, Primitive, Scene};
use anti_webui_core::tree::with_doc;
use std::time::Instant;

fn mount<V: IntoView>(f: impl FnOnce() -> V) -> Ui {
    let mut ui = Ui::new(f);
    ui.handle_event(WindowEvent::Resized {
        size: SizeI::new(400, 300),
        scale: Scale::ONE,
    });
    ui.update();
    ui
}

/// The colors of the quads of `scene`, and of those in its layers.
fn quads(scene: &Scene, out: &mut Vec<(Color, Clip)>) {
    for p in &scene.primitives {
        match p {
            Primitive::Quad(q) => out.push((q.background, q.clip)),
            Primitive::Layer(l) => quads(&l.content, out),
            _ => {}
        }
    }
}

#[test]
fn an_offscreen_node_is_laid_out_and_painted_only_on_its_own() {
    let (panel, inner) = (node_ref(), node_ref());
    let red = rgb(0xff0000);
    let blue = rgb(0x0000ff);
    let mut ui = mount(move || {
        // A box of no size that hides what overflows it, which would hide the panel too.
        div().w(10.0).h(10.0).overflow_clip().bg(Color::WHITE).child(
            div()
                .node_ref(panel)
                .w(120.0)
                .h(60.0)
                .bg(red)
                .child(div().node_ref(inner).w(30.0).h(30.0).bg(blue).absolute().top(10.0).left(20.0)),
        )
    });
    with_doc(|d| d.set_offscreen(panel.get().unwrap(), true));
    ui.update();

    // Not in the document's picture, nor what is in it (even what is stacked).
    let mut shown = Vec::new();
    quads(&ui.paint(Instant::now()), &mut shown);
    assert!(shown.iter().all(|(c, _)| *c != red && *c != blue), "{shown:?}");

    // Laid out where it is, as big as it says.
    let (scene, rect) = with_doc(|d| paint_node(d, panel.get().unwrap(), Instant::now())).unwrap();
    assert_eq!((rect.width, rect.height), (120, 60));
    let mut painted = Vec::new();
    quads(&scene, &mut painted);
    let colors: Vec<Color> = painted.iter().map(|(c, _)| *c).collect();
    assert_eq!(colors, [red, blue]);
    // Clipped by nothing around it.
    assert!(painted.iter().all(|(_, clip)| clip.rect.width > 1000), "{painted:?}");
    let r = with_doc(|d| d.get(inner.get().unwrap()).unwrap().abs);
    assert_eq!((r.x - rect.x, r.y - rect.y), (20, 10));

    // Shown again where it is once it isn't offscreen.
    with_doc(|d| d.set_offscreen(panel.get().unwrap(), false));
    ui.update();
    let mut shown = Vec::new();
    quads(&ui.paint(Instant::now()), &mut shown);
    assert!(shown.iter().any(|(c, _)| *c == red), "{shown:?}");
}
