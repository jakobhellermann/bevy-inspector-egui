use bevy_egui::egui::Color32;

use crate::{
    egui::{self, widgets},
    utils::ui::label_button,
};
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

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &mut Context) -> bool {
        let widget = match options.multiline {
            false => widgets::TextEdit::singleline(self),
            true => widgets::TextEdit::multiline(self),
        };

        // PERF: this is changed if text if highlighted
        ui.add(widget).changed()
    }
}
impl<'a> Inspectable for &'a str {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.label(*self);
        false
    }
}

impl Inspectable for bool {
    type Attributes = ();
    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.checkbox(self, "").changed()
    }
}

impl<T> Inspectable for RangeInclusive<T>
where
    T: Inspectable + Default,
{
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            let replacement = T::default()..=T::default();
            let (mut start, mut end) = std::mem::replace(self, replacement).into_inner();

            changed |= start.ui(ui, options.clone(), &mut context.with_id(0));
            ui.label("..=");
            changed |= end.ui(ui, options, &mut context.with_id(1));

            *self = start..=end;
        });
        changed
    }
}

impl<T> Inspectable for Range<T>
where
    T: Inspectable + Default,
{
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= self.start.ui(ui, options.clone(), &mut context.with_id(0));
            ui.label("..");
            changed |= self.end.ui(ui, options, &mut context.with_id(1));
        });
        changed
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

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        match self {
            Some(val) => {
                changed |= val.ui(ui, options.inner, context);
                if options.deletable {
                    if label_button(ui, "✖", Color32::RED) {
                        *self = None;
                        changed = true;
                    }
                }
            }
            None => {
                ui.label("None");
                if let Some(replacement) = options.replacement {
                    if label_button(ui, "+", Color32::GREEN) {
                        *self = Some(replacement());
                        changed = true;
                    }
                }
            }
        }
        changed
    }
}

impl Inspectable for Duration {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut seconds = self.as_secs_f64();
        let attributes = NumberAttributes {
            min: Some(0.0),
            suffix: "s".to_string(),
            ..Default::default()
        };
        let changed = seconds.ui(ui, attributes, context);
        if changed { // floating point conversion is lossy
            *self = Duration::from_secs_f64(seconds);
        }
        changed
    }
}
