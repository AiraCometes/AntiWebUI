//! What core knows of a box that the program draws on the GPU, `gpu_canvas`: a number to find what
//! draws it by, and how many times what it shows has changed. What draws it is up to the renderer.
//! See `docs/design/41-gpu-canvas.md`.

use crate::drag::{DragData, DropEvent, GrabEvent};
use crate::event::{DragResult, KeyboardEvent, PinchEvent, PointerEvent, ScrollEvent};
use crate::geometry::PointF;
use crate::reactive::effect;
use crate::shader::{IntoImage, IntoPaint};
use crate::tree::{Node, NodeKind, with_doc};
use crate::view::{ElementConfig, IntoView, NodeRef, View, also, also_bool};
use anti_webui_style::{Sheet, Style, Styled};
use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

/// What a canvas is, when nothing says how big: as `<canvas>`.
const NATURAL: (f32, f32) = (300.0, 150.0);

#[doc(hidden)]
pub struct CanvasData {
    /// What the renderer finds what draws it by.
    pub id: u64,
    /// One more each time what it shows changes.
    pub version: u64,
    /// What draws it, for as long as the node is there.
    _keep: Rc<dyn Any>,
    /// What it is drawn from, when that goes with the scene.
    pub portable: Option<Portable>,
}

impl CanvasData {
    pub fn natural(&self) -> (f32, f32) {
        NATURAL
    }

    /// A canvas that is drawn from `paint` by whatever renderer draws the scene it is in, on
    /// whichever thread: what draws it isn't kept on the thread of the document. Make `paint` with
    /// `anti_webui_wgpu::canvas::portable`.
    pub fn portable(id: u64, paint: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            id,
            version: 0,
            _keep: Rc::new(()),
            portable: Some(Portable {
                life: Arc::new(()),
                paint,
            }),
        }
    }

    /// Has a portable canvas drawn from `paint` from now on. The node has to be marked with
    /// `mark_shader_of` for the scene to show it.
    pub fn set_paint(&mut self, paint: Arc<dyn Any + Send + Sync>) {
        if let Some(p) = &mut self.portable {
            p.paint = paint;
            self.version += 1;
        }
    }
}

/// What a portable canvas is drawn from (see [`CanvasData::portable`]), as the scene carries it.
#[derive(Clone)]
pub struct Portable {
    /// There for as long as the canvas is, in its node or in a scene: what a renderer keeps for the
    /// canvas goes when this does.
    pub life: Arc<()>,
    /// What draws it, and what from. Only the renderer knows what is in it.
    pub paint: Arc<dyn Any + Send + Sync>,
}

impl PartialEq for Portable {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.life, &other.life) && Arc::ptr_eq(&self.paint, &other.paint)
    }
}

impl std::fmt::Debug for Portable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Portable")
    }
}

/// A box whose pixels the program draws on the GPU. Make one with `anti_webui::gpu::gpu_canvas`.
pub struct GpuCanvas {
    cfg: ElementConfig,
    data: CanvasData,
    update: Box<dyn Fn() -> bool>,
    /// For a portable canvas, what it is drawn from after `update` said that changed.
    paint: Option<Box<dyn Fn() -> Arc<dyn Any + Send + Sync>>>,
    children: Vec<View>,
}

/// A canvas drawn by what the renderer finds by `id`, which `keep` keeps. `update` runs again when
/// the signals it reads change, and says whether what the canvas shows changed.
#[doc(hidden)]
pub fn gpu_canvas_from(
    id: u64,
    keep: Rc<dyn Any>,
    update: impl Fn() -> bool + 'static,
) -> GpuCanvas {
    GpuCanvas {
        cfg: ElementConfig::default(),
        data: CanvasData {
            id,
            version: 0,
            _keep: keep,
            portable: None,
        },
        update: Box::new(update),
        paint: None,
        children: Vec::new(),
    }
}

/// A portable canvas (see [`CanvasData::portable`]) with the id `id`. `update` runs again when the
/// signals it reads change, and says whether what the canvas shows changed; `paint` then gives what
/// it is drawn from.
#[doc(hidden)]
pub fn portable_canvas_from(
    id: u64,
    update: impl Fn() -> bool + 'static,
    paint: impl Fn() -> Arc<dyn Any + Send + Sync> + 'static,
) -> GpuCanvas {
    GpuCanvas {
        cfg: ElementConfig::default(),
        // Given what it is drawn from before it is first painted, by the effect below.
        data: CanvasData::portable(id, Arc::new(())),
        update: Box::new(update),
        paint: Some(Box::new(paint)),
        children: Vec::new(),
    }
}

impl Styled for GpuCanvas {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.cfg.style
    }
}

impl GpuCanvas {
    crate::view::element_methods!();
    crate::view::box_paint_methods!();

    /// Lays something over the canvas, in its content box. It has no say in the size of the box.
    pub fn child(mut self, child: impl IntoView) -> Self {
        self.children.push(child.into_view());
        self
    }

    pub fn children<V: IntoView>(mut self, children: impl IntoIterator<Item = V>) -> Self {
        self.children
            .extend(children.into_iter().map(IntoView::into_view));
        self
    }
}

impl IntoView for GpuCanvas {
    fn into_view(self) -> View {
        let id = self
            .cfg
            .build(Node::new(NodeKind::Canvas(Box::new(self.data))));
        let children: Vec<_> = self.children.into_iter().map(|v| v.0).collect();
        with_doc(|d| d.set_children(id, children));
        let update = self.update;
        let paint = self.paint;
        effect(move || {
            if !update() {
                return;
            }
            let paint = paint.as_ref().map(|p| p());
            with_doc(|d| {
                let Some(NodeKind::Canvas(c)) = d.get_mut(id).map(|n| &mut n.kind) else {
                    return;
                };
                match paint {
                    Some(p) => c.set_paint(p),
                    None => c.version += 1,
                }
                // Only the box changes, as when a shader's parameters do.
                d.mark_shader_of(id);
            });
        });
        View(id)
    }
}
