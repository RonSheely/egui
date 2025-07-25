//! Handles paint layers, i.e. how things
//! are sometimes painted behind or in front of other things.

use crate::{Id, IdMap, Rect, ahash, epaint};
use epaint::{ClippedShape, Shape, emath::TSTransform};

/// Different layer categories
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Order {
    /// Painted behind all floating windows
    Background,

    /// Normal moveable windows that you reorder by click
    Middle,

    /// Popups, menus etc that should always be painted on top of windows
    /// Foreground objects can also have tooltips
    Foreground,

    /// Things floating on top of everything else, like tooltips.
    /// You cannot interact with these.
    Tooltip,

    /// Debug layer, always painted last / on top
    Debug,
}

impl Order {
    const COUNT: usize = 5;
    const ALL: [Self; Self::COUNT] = [
        Self::Background,
        Self::Middle,
        Self::Foreground,
        Self::Tooltip,
        Self::Debug,
    ];
    pub const TOP: Self = Self::Debug;

    #[inline(always)]
    pub fn allow_interaction(&self) -> bool {
        match self {
            Self::Background | Self::Middle | Self::Foreground | Self::Tooltip | Self::Debug => {
                true
            }
        }
    }

    /// Short and readable summary
    pub fn short_debug_format(&self) -> &'static str {
        match self {
            Self::Background => "backg",
            Self::Middle => "middl",
            Self::Foreground => "foreg",
            Self::Tooltip => "toolt",
            Self::Debug => "debug",
        }
    }
}

/// An identifier for a paint layer.
/// Also acts as an identifier for [`crate::Area`]:s.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LayerId {
    pub order: Order,
    pub id: Id,
}

impl LayerId {
    pub fn new(order: Order, id: Id) -> Self {
        Self { order, id }
    }

    pub fn debug() -> Self {
        Self {
            order: Order::Debug,
            id: Id::new("debug"),
        }
    }

    pub fn background() -> Self {
        Self {
            order: Order::Background,
            id: Id::new("background"),
        }
    }

    #[inline(always)]
    #[deprecated = "Use `Memory::allows_interaction` instead"]
    pub fn allow_interaction(&self) -> bool {
        self.order.allow_interaction()
    }

    /// Short and readable summary
    pub fn short_debug_format(&self) -> String {
        format!(
            "{} {}",
            self.order.short_debug_format(),
            self.id.short_debug_format()
        )
    }
}

impl std::fmt::Debug for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { order, id } = self;
        write!(f, "LayerId {{ {order:?} {id:?} }}")
    }
}

/// A unique identifier of a specific [`Shape`] in a [`PaintList`].

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeIdx(pub usize);

/// A list of [`Shape`]s paired with a clip rectangle.
#[derive(Clone, Default)]
pub struct PaintList(Vec<ClippedShape>);

impl PaintList {
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn next_idx(&self) -> ShapeIdx {
        ShapeIdx(self.0.len())
    }

    /// Returns the index of the new [`Shape`] that can be used with `PaintList::set`.
    #[inline(always)]
    pub fn add(&mut self, clip_rect: Rect, shape: Shape) -> ShapeIdx {
        let idx = self.next_idx();
        self.0.push(ClippedShape { clip_rect, shape });
        idx
    }

    pub fn extend<I: IntoIterator<Item = Shape>>(&mut self, clip_rect: Rect, shapes: I) {
        self.0.extend(
            shapes
                .into_iter()
                .map(|shape| ClippedShape { clip_rect, shape }),
        );
    }

    /// Modify an existing [`Shape`].
    ///
    /// Sometimes you want to paint a frame behind some contents, but don't know how large the frame needs to be
    /// until the contents have been added, and therefor also painted to the [`PaintList`].
    ///
    /// The solution is to allocate a [`Shape`] using `let idx = paint_list.add(cr, Shape::Noop);`
    /// and then later setting it using `paint_list.set(idx, cr, frame);`.
    #[inline(always)]
    pub fn set(&mut self, idx: ShapeIdx, clip_rect: Rect, shape: Shape) {
        if self.0.len() <= idx.0 {
            #[cfg(feature = "log")]
            log::warn!("Index {} is out of bounds for PaintList", idx.0);
            return;
        }

        self.0[idx.0] = ClippedShape { clip_rect, shape };
    }

    /// Set the given shape to be empty (a `Shape::Noop`).
    #[inline(always)]
    pub fn reset_shape(&mut self, idx: ShapeIdx) {
        self.0[idx.0].shape = Shape::Noop;
    }

    /// Mutate the shape at the given index, if any.
    pub fn mutate_shape(&mut self, idx: ShapeIdx, f: impl FnOnce(&mut ClippedShape)) {
        self.0.get_mut(idx.0).map(f);
    }

    /// Transform each [`Shape`] and clip rectangle by this much, in-place
    pub fn transform(&mut self, transform: TSTransform) {
        for ClippedShape { clip_rect, shape } in &mut self.0 {
            *clip_rect = transform.mul_rect(*clip_rect);
            shape.transform(transform);
        }
    }

    /// Transform each [`Shape`] and clip rectangle in range by this much, in-place
    pub fn transform_range(&mut self, start: ShapeIdx, end: ShapeIdx, transform: TSTransform) {
        for ClippedShape { clip_rect, shape } in &mut self.0[start.0..end.0] {
            *clip_rect = transform.mul_rect(*clip_rect);
            shape.transform(transform);
        }
    }

    /// Read-only access to all held shapes.
    pub fn all_entries(&self) -> impl ExactSizeIterator<Item = &ClippedShape> {
        self.0.iter()
    }
}

/// This is where painted [`Shape`]s end up during a frame.
#[derive(Clone, Default)]
pub struct GraphicLayers([IdMap<PaintList>; Order::COUNT]);

impl GraphicLayers {
    /// Get or insert the [`PaintList`] for the given [`LayerId`].
    pub fn entry(&mut self, layer_id: LayerId) -> &mut PaintList {
        self.0[layer_id.order as usize]
            .entry(layer_id.id)
            .or_default()
    }

    /// Get the [`PaintList`] for the given [`LayerId`].
    pub fn get(&self, layer_id: LayerId) -> Option<&PaintList> {
        self.0[layer_id.order as usize].get(&layer_id.id)
    }

    /// Get the [`PaintList`] for the given [`LayerId`].
    pub fn get_mut(&mut self, layer_id: LayerId) -> Option<&mut PaintList> {
        self.0[layer_id.order as usize].get_mut(&layer_id.id)
    }

    pub fn drain(
        &mut self,
        area_order: &[LayerId],
        to_global: &ahash::HashMap<LayerId, TSTransform>,
    ) -> Vec<ClippedShape> {
        profiling::function_scope!();

        let mut all_shapes: Vec<_> = Default::default();

        for &order in &Order::ALL {
            let order_map = &mut self.0[order as usize];

            // If a layer is empty at the start of the frame
            // then nobody has added to it, and it is old and defunct.
            // Free it to save memory:
            order_map.retain(|_, list| !list.is_empty());

            // First do the layers part of area_order:
            for layer_id in area_order {
                if layer_id.order == order {
                    if let Some(list) = order_map.get_mut(&layer_id.id) {
                        if let Some(to_global) = to_global.get(layer_id) {
                            for clipped_shape in &mut list.0 {
                                clipped_shape.clip_rect = *to_global * clipped_shape.clip_rect;
                                clipped_shape.shape.transform(*to_global);
                            }
                        }
                        all_shapes.append(&mut list.0);
                    }
                }
            }

            // Also draw areas that are missing in `area_order`:
            for (id, list) in order_map {
                let layer_id = LayerId::new(order, *id);

                if let Some(to_global) = to_global.get(&layer_id) {
                    for clipped_shape in &mut list.0 {
                        clipped_shape.clip_rect = *to_global * clipped_shape.clip_rect;
                        clipped_shape.shape.transform(*to_global);
                    }
                }

                all_shapes.append(&mut list.0);
            }
        }

        all_shapes
    }
}
