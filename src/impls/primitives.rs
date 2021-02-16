use crate::egui::{self, widgets};
use crate::{Context, Inspectable};
use std::ops::{Range, RangeInclusive};

#[derive(Clone, Debug, Default)]
pub struct StringAttributes {
    pub multiline: bool,
}

impl Inspectable for String {
    type Attributes = StringAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) {
        let widget = match options.multiline {
            false => widgets::TextEdit::singleline(self),
            true => widgets::TextEdit::multiline(self),
        };

        ui.add(widget);
    }
}

impl Inspectable for bool {
    type Attributes = ();
    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &Context) {
        ui.checkbox(self, "");
    }
}

impl<T> Inspectable for RangeInclusive<T>
where
    T: Inspectable + Default,
{
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        ui.horizontal(|ui| {
            let replacement = T::default()..=T::default();
            let (mut start, mut end) = std::mem::replace(self, replacement).into_inner();

            start.ui(ui, options.clone(), &context.with_id(0));
            ui.label("..=");
            end.ui(ui, options, &context.with_id(1));

            *self = start..=end;
        });
    }
}

impl<T> Inspectable for Range<T>
where
    T: Inspectable + Default,
{
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        ui.horizontal(|ui| {
            self.start.ui(ui, options.clone(), &context.with_id(0));
            ui.label("..");
            self.end.ui(ui, options, &context.with_id(1));
        });
    }
}

impl<T: Inspectable> Inspectable for Option<T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        match self {
            Some(val) => {
                val.ui(ui, options, context);
            }
            None => {
                ui.label("None");
            }
        }
    }
}
