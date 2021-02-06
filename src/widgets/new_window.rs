use std::ops::{Deref, DerefMut};

use crate::{utils::short_name, Context, Inspectable};
use bevy_egui::egui::{self, Id};

/// The attributes for [`InNewWindow`]
#[allow(missing_docs)]
pub struct WindowAttributes<T: Inspectable> {
    pub title: Option<&'static str>,
    pub title_bar: bool,
    pub scrollable: bool,
    pub resizable: bool,
    pub collapsible: bool,
    pub inner_attributes: <T as Inspectable>::Attributes,
}
impl<T: Inspectable> Default for WindowAttributes<T> {
    fn default() -> Self {
        WindowAttributes {
            title: None,
            title_bar: true,
            scrollable: false,
            resizable: false,
            collapsible: true,
            inner_attributes: Default::default(),
        }
    }
}

#[derive(Default)]
/// Wrapper type which displays the inner value inside another egui window.
///
/// Can be configured using [`WindowAttributes`].
pub struct InNewWindow<T>(pub T);

impl<T> Deref for InNewWindow<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for InNewWindow<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Inspectable + 'static> Inspectable for InNewWindow<T> {
    type Attributes = WindowAttributes<T>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        ui.label("<shown in another window>");

        let window_title = options
            .title
            .map(|title| title.to_string())
            .unwrap_or_else(|| short_name(std::any::type_name::<T>()));

        let id = Id::new(std::any::TypeId::of::<T>()).with(context.id);
        egui::Window::new(window_title)
            .id(id)
            .resizable(options.resizable)
            .scroll(options.scrollable)
            .title_bar(options.title_bar)
            .collapsible(options.collapsible)
            .default_pos([400., 100.])
            .show(context.ui_ctx, |ui| {
                <T as Inspectable>::ui(&mut self.0, ui, options.inner_attributes, context)
            });
    }
}
