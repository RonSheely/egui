use crate::{
    Size,
    layout::{CellDirection, CellSize, StripLayout, StripLayoutFlags},
    sizing::Sizing,
};
use egui::{Response, Ui};

/// Builder for creating a new [`Strip`].
///
/// This can be used to do dynamic layouts.
///
/// In contrast to normal egui behavior, strip cells do *not* grow with its children!
///
/// First use [`Self::size`] and [`Self::sizes`] to allocate space for the rows or columns will follow.
/// Then build the strip with [`Self::horizontal`]/[`Self::vertical`], and add 'cells'
/// to it using [`Strip::cell`]. The number of cells MUST match the number of pre-allocated sizes.
///
/// ### Example
/// ```
/// # egui::__run_test_ui(|ui| {
/// use egui_extras::{StripBuilder, Size};
/// StripBuilder::new(ui)
///     .size(Size::remainder().at_least(100.0)) // top cell
///     .size(Size::exact(40.0)) // bottom cell
///     .vertical(|mut strip| {
///         // Add the top 'cell'
///         strip.cell(|ui| {
///             ui.label("Fixed");
///         });
///         // We add a nested strip in the bottom cell:
///         strip.strip(|builder| {
///             builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
///                 strip.cell(|ui| {
///                     ui.label("Top Left");
///                 });
///                 strip.cell(|ui| {
///                     ui.label("Top Right");
///                 });
///             });
///         });
///     });
/// # });
/// ```
pub struct StripBuilder<'a> {
    ui: &'a mut Ui,
    sizing: Sizing,
    clip: bool,
    cell_layout: egui::Layout,
    sense: egui::Sense,
}

impl<'a> StripBuilder<'a> {
    /// Create new strip builder.
    pub fn new(ui: &'a mut Ui) -> Self {
        let cell_layout = *ui.layout();
        Self {
            ui,
            sizing: Default::default(),
            clip: false,
            cell_layout,
            sense: egui::Sense::hover(),
        }
    }

    /// Should we clip the contents of each cell? Default: `false`.
    #[inline]
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// What layout should we use for the individual cells?
    #[inline]
    pub fn cell_layout(mut self, cell_layout: egui::Layout) -> Self {
        self.cell_layout = cell_layout;
        self
    }

    /// What should strip cells sense for? Default: [`egui::Sense::hover()`].
    #[inline]
    pub fn sense(mut self, sense: egui::Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Allocate space for one column/row.
    #[inline]
    pub fn size(mut self, size: Size) -> Self {
        self.sizing.add(size);
        self
    }

    /// Allocate space for several columns/rows at once.
    #[inline]
    pub fn sizes(mut self, size: Size, count: usize) -> Self {
        for _ in 0..count {
            self.sizing.add(size);
        }
        self
    }

    /// Build horizontal strip: Cells are positions from left to right.
    /// Takes the available horizontal width, so there can't be anything right of the strip or the container will grow slowly!
    ///
    /// Returns a [`egui::Response`] for hover events.
    pub fn horizontal<F>(self, strip: F) -> Response
    where
        F: for<'b> FnOnce(Strip<'a, 'b>),
    {
        let widths = self.sizing.to_lengths(
            self.ui.available_rect_before_wrap().width(),
            self.ui.spacing().item_spacing.x,
        );
        let mut layout = StripLayout::new(
            self.ui,
            CellDirection::Horizontal,
            self.cell_layout,
            self.sense,
        );
        strip(Strip {
            layout: &mut layout,
            direction: CellDirection::Horizontal,
            clip: self.clip,
            sizes: widths,
            size_index: 0,
        });
        layout.allocate_rect()
    }

    /// Build vertical strip: Cells are positions from top to bottom.
    /// Takes the full available vertical height, so there can't be anything below of the strip or the container will grow slowly!
    ///
    /// Returns a [`egui::Response`] for hover events.
    pub fn vertical<F>(self, strip: F) -> Response
    where
        F: for<'b> FnOnce(Strip<'a, 'b>),
    {
        let heights = self.sizing.to_lengths(
            self.ui.available_rect_before_wrap().height(),
            self.ui.spacing().item_spacing.y,
        );
        let mut layout = StripLayout::new(
            self.ui,
            CellDirection::Vertical,
            self.cell_layout,
            self.sense,
        );
        strip(Strip {
            layout: &mut layout,
            direction: CellDirection::Vertical,
            clip: self.clip,
            sizes: heights,
            size_index: 0,
        });
        layout.allocate_rect()
    }
}

/// A Strip of cells which go in one direction. Each cell has a fixed size.
/// In contrast to normal egui behavior, strip cells do *not* grow with its children!
pub struct Strip<'a, 'b> {
    layout: &'b mut StripLayout<'a>,
    direction: CellDirection,
    clip: bool,
    sizes: Vec<f32>,
    size_index: usize,
}

impl Strip<'_, '_> {
    #[cfg_attr(debug_assertions, track_caller)]
    fn next_cell_size(&mut self) -> (CellSize, CellSize) {
        let size = if let Some(size) = self.sizes.get(self.size_index) {
            self.size_index += 1;
            *size
        } else {
            crate::log_or_panic!(
                "Added more `Strip` cells than were pre-allocated ({} pre-allocated)",
                self.sizes.len()
            );
            8.0 // anything will look wrong, so pick something that is obviously wrong
        };

        match self.direction {
            CellDirection::Horizontal => (CellSize::Absolute(size), CellSize::Remainder),
            CellDirection::Vertical => (CellSize::Remainder, CellSize::Absolute(size)),
        }
    }

    /// Add cell contents.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn cell(&mut self, add_contents: impl FnOnce(&mut Ui)) {
        let (width, height) = self.next_cell_size();
        let flags = StripLayoutFlags {
            clip: self.clip,
            ..Default::default()
        };
        self.layout.add(
            flags,
            width,
            height,
            egui::Id::new(self.size_index),
            add_contents,
        );
    }

    /// Add an empty cell.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn empty(&mut self) {
        let (width, height) = self.next_cell_size();
        self.layout.empty(width, height);
    }

    /// Add a strip as cell.
    pub fn strip(&mut self, strip_builder: impl FnOnce(StripBuilder<'_>)) {
        let clip = self.clip;
        self.cell(|ui| {
            strip_builder(StripBuilder::new(ui).clip(clip));
        });
    }
}

impl Drop for Strip<'_, '_> {
    fn drop(&mut self) {
        while self.size_index < self.sizes.len() {
            self.empty();
        }
    }
}
