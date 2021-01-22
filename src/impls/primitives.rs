use std::ops::{Range, RangeInclusive};

use crate::egui::{self, widgets};
use crate::Inspectable;

#[derive(Clone, Debug, Default)]
pub struct StringAttributes {
    pub multiline: bool,
}

impl Inspectable for String {
    type Attributes = StringAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
        let widget = match options.multiline {
            false => widgets::TextEdit::singleline(self),
            true => widgets::TextEdit::multiline(self),
        };

        ui.add(widget);
    }
}

impl Inspectable for bool {
    type Attributes = ();
    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes) {
        ui.checkbox(self, "");
    }
}

impl<T> Inspectable for RangeInclusive<T>
where
    T: Inspectable + Default,
    T::Attributes: Clone,
{
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
        ui.horizontal(|ui| {
            let replacement = T::default()..=T::default();
            let (mut start, mut end) = std::mem::replace(self, replacement).into_inner();

            start.ui(ui, options.clone());
            ui.label("..=");
            end.ui(ui, options);

            *self = start..=end;
        });
    }
}

impl<T> Inspectable for Range<T>
where
    T: Inspectable + Default,
    T::Attributes: Clone,
{
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
        ui.horizontal(|ui| {
            self.start.ui(ui, options.clone());
            ui.label("..");
            self.end.ui(ui, options);
        });
    }
}
