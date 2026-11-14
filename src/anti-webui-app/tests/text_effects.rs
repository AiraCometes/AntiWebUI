//! Shadows and strokes of text through the real renderer (see `docs/design/04-text.md`). Skipped
//! when no GPU adapter is available.

use anti_webui_core::prelude::*;

const W: u32 = 240;
const H: u32 = 160;
/// The row the tests look along: through the middle of the stem of the `l`.
const ROW: u32 = 80;

const RED: Color = Color::from_rgb(0xff0000);
const BLUE: Color = Color::from_rgb(0x0000ff);

/// A black `l` on white, its stem at about x = 60, styled by `style`.
fn letter(style: impl FnOnce(Div) -> Div) -> Option<Vec<u8>> {
    let view = move || {
        div().size_full().bg(Color::WHITE).child(style(
            div()
                .absolute()
                .left(50.0)
                .top(0.0)
                .text_size(140.0)
                .text_color(Color::BLACK)
                .child(text("l")),
        ))
    };
    match anti_webui_app::headless::render_rgba(W, H, 1.0, view) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("skipping: {e}");
            None
        }
    }
}

fn px(p: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [p[i], p[i + 1], p[i + 2], p[i + 3]]
}

/// Which of three colors a pixel is nearest to, if it is near one: black, red or blue.
fn kind(p: [u8; 4]) -> Option<char> {
    let near = |c: [u8; 3]| (0..3).all(|i| (p[i] as i32 - c[i] as i32).abs() < 60);
    if near([0, 0, 0]) {
        Some('k')
    } else if near([255, 0, 0]) {
        Some('r')
    } else if near([0, 0, 255]) {
        Some('b')
    } else if near([255, 255, 255]) {
        Some('w')
    } else {
        None
    }
}

/// The colors along the row, as runs: (color, how many pixels), leaving out what is between.
fn runs(p: &[u8]) -> Vec<(char, u32)> {
    let mut out: Vec<(char, u32)> = Vec::new();
    for x in 0..W {
        let Some(k) = kind(px(p, x, ROW)) else {
            continue;
        };
        match out.last_mut() {
            Some((last, n)) if *last == k => *n += 1,
            _ => out.push((k, 1)),
        }
    }
    out
}

fn width_of(runs: &[(char, u32)], k: char) -> u32 {
    runs.iter().filter(|(c, _)| *c == k).map(|(_, n)| n).sum()
}

#[test]
fn a_stroke_under_the_letters_edges_them_and_over_them_eats_into_them() {
    let Some(plain) = letter(|d| d) else { return };
    let stem = width_of(&runs(&plain), 'k');
    assert!(stem >= 10, "the stem is {stem} px wide: {:?}", runs(&plain));

    let under = letter(|d| d.text_stroke(12.0, RED).paint_order(PaintOrder::Stroke)).unwrap();
    let r = runs(&under);
    assert_eq!(
        r.iter().map(|(c, _)| *c).collect::<String>(),
        "wrkrw",
        "{r:?}"
    );
    // Half of the stroke is outside the letter, on each side.
    assert!(
        width_of(&r, 'k').abs_diff(stem) <= 2,
        "{r:?} for a stem of {stem}"
    );
    assert!((10..=14).contains(&width_of(&r, 'r')), "{r:?}");

    let over = letter(|d| d.text_stroke(12.0, RED)).unwrap();
    let r = runs(&over);
    // The half inside covers the letter too.
    assert!(width_of(&r, 'k') + 10 <= stem, "{r:?} for a stem of {stem}");
    assert!(width_of(&r, 'r') >= 22, "{r:?}");
}

#[test]
fn the_first_stroke_is_on_top_and_a_wider_one_after_it_edges_it() {
    let Some(p) = letter(|d| {
        d.text_strokes(vec![
            TextStroke {
                width: 8.0,
                color: RED,
            },
            TextStroke {
                width: 24.0,
                color: BLUE,
            },
        ])
        .paint_order(PaintOrder::Stroke)
    }) else {
        return;
    };
    let r = runs(&p);
    assert_eq!(
        r.iter().map(|(c, _)| *c).collect::<String>(),
        "wbrkrbw",
        "{r:?}"
    );
    // Each side: 4 px of red, then 12 - 4 of blue.
    assert!((6..=10).contains(&width_of(&r, 'r')), "{r:?}");
    assert!((14..=18).contains(&width_of(&r, 'b')), "{r:?}");
}

#[test]
fn a_shadow_is_the_letters_moved_and_under_them() {
    let Some(plain) = letter(|d| d) else { return };
    let shadowed = letter(|d| {
        d.text_shadow(TextShadow {
            offset_x: 40.0,
            offset_y: 0.0,
            blur: 0.0,
            color: BLUE,
        })
    })
    .unwrap();
    let (a, b) = (runs(&plain), runs(&shadowed));
    assert_eq!(
        b.iter().map(|(c, _)| *c).collect::<String>(),
        "wkwbw",
        "{b:?}"
    );
    assert!(
        width_of(&a, 'k').abs_diff(width_of(&b, 'b')) <= 1,
        "{a:?} {b:?}"
    );
    let first = |p: &[u8], k| (0..W).find(|&x| kind(px(p, x, ROW)) == Some(k)).unwrap();
    assert_eq!(first(&shadowed, 'b') - first(&shadowed, 'k'), 40);
}

#[test]
fn a_blurred_shadow_is_soft_and_strokes_are_in_it() {
    let Some(sharp) = letter(|d| {
        d.text_stroke(16.0, RED)
            .paint_order(PaintOrder::Stroke)
            .text_shadow(TextShadow {
                offset_x: 60.0,
                offset_y: 0.0,
                blur: 0.0,
                color: BLUE,
            })
    }) else {
        return;
    };
    let soft = letter(|d| {
        d.text_stroke(16.0, RED)
            .paint_order(PaintOrder::Stroke)
            .text_shadow(TextShadow {
                offset_x: 60.0,
                offset_y: 0.0,
                blur: 6.0,
                color: BLUE,
            })
    })
    .unwrap();
    // The shadow of the stroked letter is as wide as the letter and its stroke.
    let r = runs(&sharp);
    let letter = width_of(&r, 'k') + width_of(&r, 'r');
    assert!(width_of(&r, 'b').abs_diff(letter) <= 2, "{r:?}");
    // Blurred, it fades out: there are pixels of blue all along between full and none.
    let blues: Vec<u8> = (0..W).map(|x| 255 - px(&soft, x, ROW)[0]).collect();
    let between = blues.iter().filter(|&&b| (40..215).contains(&b)).count();
    assert!(between >= 12, "{blues:?}");
    // The letter itself is as sharp as before.
    let r = runs(&soft);
    assert!(
        r.iter()
            .any(|(c, n)| *c == 'k' && *n + 2 >= width_of(&runs(&sharp), 'k')),
        "{r:?}"
    );
}
