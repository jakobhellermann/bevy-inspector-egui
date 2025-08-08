use egui::{
    Id, Popup, PopupCloseBehavior, Response, TextEdit, Ui, Widget, WidgetText,
    text::{CCursor, CCursorRange},
};
use std::hash::Hash;

/// Dropdown widget
pub struct DropDownBox<
    'a,
    F: FnMut(&mut Ui, &str) -> Response,
    V: AsRef<str>,
    I: Iterator<Item = V>,
> {
    buf: &'a mut String,
    popup_id: Id,
    display: F,
    it: I,
    hint_text: WidgetText,
    filter_by_input: bool,
    select_on_focus: bool,
    desired_width: Option<f32>,
}

impl<'a, F: FnMut(&mut Ui, &str) -> Response, V: AsRef<str>, I: Iterator<Item = V>>
    DropDownBox<'a, F, V, I>
{
    /// Creates new dropdown box.
    pub fn from_iter(
        it: impl IntoIterator<IntoIter = I>,
        id_source: impl Hash,
        buf: &'a mut String,
        display: F,
    ) -> Self {
        Self {
            popup_id: Id::new(id_source),
            it: it.into_iter(),
            display,
            buf,
            hint_text: WidgetText::default(),
            filter_by_input: true,
            select_on_focus: false,
            desired_width: None,
        }
    }

    /// Add a hint text to the Text Edit
    pub fn hint_text(mut self, hint_text: impl Into<WidgetText>) -> Self {
        self.hint_text = hint_text.into();
        self
    }

    /// Determine whether to filter box items based on what is in the Text Edit already
    pub fn filter_by_input(mut self, filter_by_input: bool) -> Self {
        self.filter_by_input = filter_by_input;
        self
    }

    /// Determine whether to select the text when the Text Edit gains focus
    pub fn select_on_focus(mut self, select_on_focus: bool) -> Self {
        self.select_on_focus = select_on_focus;
        self
    }

    /// Passes through the desired width value to the underlying Text Edit
    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = desired_width.into();
        self
    }
}

impl<'a, F: FnMut(&mut Ui, &str) -> Response, V: AsRef<str>, I: Iterator<Item = V>> Widget
    for DropDownBox<'a, F, V, I>
{
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            popup_id,
            buf,
            it,
            mut display,
            hint_text,
            filter_by_input,
            select_on_focus,
            desired_width,
        } = self;

        let mut edit = TextEdit::singleline(buf).hint_text(hint_text);
        if let Some(dw) = desired_width {
            edit = edit.desired_width(dw);
        }
        let mut edit_output = edit.show(ui);
        let mut r = edit_output.response;
        if r.gained_focus() {
            if select_on_focus {
                edit_output
                    .state
                    .cursor
                    .set_char_range(Some(CCursorRange::two(
                        CCursor::new(0),
                        CCursor::new(buf.len()),
                    )));
                edit_output.state.store(ui.ctx(), r.id);
            }
            Popup::open_id(ui.ctx(), popup_id);
        }

        let mut changed = false;
        Popup::menu(&r)
            .id(popup_id)
            .close_behavior(PopupCloseBehavior::CloseOnClick)
            .show(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut any_visible = false;
                    let mut counter = 0;
                    for var in it {
                        counter += 1;
                        let text = var.as_ref();
                        if filter_by_input
                            && !buf.is_empty()
                            && !text.to_lowercase().contains(&buf.to_lowercase())
                        {
                            continue;
                        }
                        any_visible = true;

                        if display(ui, text).clicked() {
                            *buf = text.to_owned();
                            changed = true;
                        }
                    }
                    if !any_visible {
                        if buf.is_empty() {
                            ui.label("No items found");
                        } else {
                            ui.label(format!("No items out of {} match the filter", counter));
                        }
                    }
                });
            });

        if changed {
            Popup::close_id(ui.ctx(), popup_id);
            r.mark_changed();
        }

        r
    }
}
