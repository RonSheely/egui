use crate::{
    CursorIcon, Label, Response, Sense, Stroke, Ui, Widget, WidgetInfo, WidgetText, WidgetType,
    epaint, text_selection,
};

use self::text_selection::LabelSelectionState;

/// Clickable text, that looks like a hyperlink.
///
/// To link to a web page, use [`Hyperlink`], [`Ui::hyperlink`] or [`Ui::hyperlink_to`].
///
/// See also [`Ui::link`].
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// // These are equivalent:
/// if ui.link("Documentation").clicked() {
///     // …
/// }
///
/// if ui.add(egui::Link::new("Documentation")).clicked() {
///     // …
/// }
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Link {
    text: WidgetText,
}

impl Link {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for Link {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { text } = self;
        let label = Label::new(text).sense(Sense::click());

        let (galley_pos, galley, response) = label.layout_in_ui(ui);
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Link, ui.is_enabled(), galley.text()));

        if ui.is_rect_visible(response.rect) {
            let color = ui.visuals().hyperlink_color;
            let visuals = ui.style().interact(&response);

            let underline = if response.hovered() || response.has_focus() {
                Stroke::new(visuals.fg_stroke.width, color)
            } else {
                Stroke::NONE
            };

            let selectable = ui.style().interaction.selectable_labels;
            if selectable {
                LabelSelectionState::label_text_selection(
                    ui, &response, galley_pos, galley, color, underline,
                );
            } else {
                ui.painter().add(
                    epaint::TextShape::new(galley_pos, galley, color).with_underline(underline),
                );
            }

            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
        }

        response
    }
}

/// A clickable hyperlink, e.g. to `"https://github.com/emilk/egui"`.
///
/// See also [`Ui::hyperlink`] and [`Ui::hyperlink_to`].
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// // These are equivalent:
/// ui.hyperlink("https://github.com/emilk/egui");
/// ui.add(egui::Hyperlink::new("https://github.com/emilk/egui"));
///
/// // These are equivalent:
/// ui.hyperlink_to("My favorite repo", "https://github.com/emilk/egui");
/// ui.add(egui::Hyperlink::from_label_and_url("My favorite repo", "https://github.com/emilk/egui"));
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Hyperlink {
    url: String,
    text: WidgetText,
    new_tab: bool,
}

impl Hyperlink {
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(url: impl ToString) -> Self {
        let url = url.to_string();
        Self {
            url: url.clone(),
            text: url.into(),
            new_tab: false,
        }
    }

    #[expect(clippy::needless_pass_by_value)]
    pub fn from_label_and_url(text: impl Into<WidgetText>, url: impl ToString) -> Self {
        Self {
            url: url.to_string(),
            text: text.into(),
            new_tab: false,
        }
    }

    /// Always open this hyperlink in a new browser tab.
    #[inline]
    pub fn open_in_new_tab(mut self, new_tab: bool) -> Self {
        self.new_tab = new_tab;
        self
    }
}

impl Widget for Hyperlink {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { url, text, new_tab } = self;

        let response = ui.add(Link::new(text));

        if response.clicked_with_open_in_background() {
            ui.ctx().open_url(crate::OpenUrl {
                url: url.clone(),
                new_tab: true,
            });
        } else if response.clicked() {
            ui.ctx().open_url(crate::OpenUrl {
                url: url.clone(),
                new_tab,
            });
        }

        if ui.style().url_in_tooltip {
            response.on_hover_text(url)
        } else {
            response
        }
    }
}
