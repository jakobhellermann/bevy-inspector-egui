use bevy_egui::egui::Color32;

use crate::egui::{self, widgets};
use crate::{Context, Inspectable};
use std::{
    ops::{Range, RangeInclusive},
    time::Duration,
};

use super::NumberAttributes;

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
impl<'a> Inspectable for &'a str {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &Context) {
        ui.label(*self);
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

pub struct OptionAttributes<T: Inspectable> {
    pub replacement: Option<fn() -> T>,
    pub deletable: bool,
    pub inner: T::Attributes,
}
impl<T: Inspectable> Clone for OptionAttributes<T> {
    fn clone(&self) -> Self {
        OptionAttributes {
            replacement: self.replacement,
            deletable: self.deletable,
            inner: self.inner.clone(),
        }
    }
}
impl<T: Inspectable> Default for OptionAttributes<T> {
    fn default() -> Self {
        OptionAttributes {
            replacement: None,
            deletable: true,
            inner: T::Attributes::default(),
        }
    }
}

impl<T: Inspectable> Inspectable for Option<T> {
    type Attributes = OptionAttributes<T>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        match self {
            Some(val) => {
                val.ui(ui, options.inner, context);
                if options.deletable {
                    if ui.colored_label(Color32::RED, "✖").clicked() {
                        *self = None;
                    }
                }
            }
            None => {
                ui.label("None");
                if let Some(replacement) = options.replacement {
                    if ui.colored_label(Color32::GREEN, "+").clicked() {
                        *self = Some(replacement());
                    }
                }
            }
        }
    }
}

impl Inspectable for Duration {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        let mut seconds = self.as_secs_f32();
        let attributes = NumberAttributes {
            min: Some(0.0),
            suffix: "s".to_string(),
            ..Default::default()
        };
        seconds.ui(ui, attributes, context);
        *self = Duration::from_secs_f32(seconds);
    }
}
