use crate::{Context, Inspectable};
use bevy::prelude::*;
use std::marker::PhantomData;

#[allow(missing_docs)]
#[derive(Clone)]
pub struct ButtonAttributes {
    pub text: String,
}
impl Default for ButtonAttributes {
    fn default() -> Self {
        ButtonAttributes {
            text: "Button".to_string(),
        }
    }
}

#[derive(Default)]
/// Button in the inspector. When clicking, the event `E::Default` will be sent.
///
/// Can be configured via `#[inspectable(text = "Button text")]`.
pub struct InspectableButton<E>(PhantomData<E>);
impl<E> InspectableButton<E> {
    /// Create a new `InspectableButton`
    pub fn new() -> Self {
        InspectableButton(PhantomData)
    }
}

impl<E> std::fmt::Debug for InspectableButton<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InspectableButton").finish()
    }
}

impl<E: Resource + Default> Inspectable for InspectableButton<E> {
    type Attributes = ButtonAttributes;

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, options: Self::Attributes, context: &Context) {
        let resources = expect_context!(ui, context.resources, "InspectableButton");
        let mut events = expect_resource!(ui, resources, get_mut Events<E>);

        if ui.button(options.text).clicked() {
            events.send(E::default());
        }
    }

    fn setup(app: &mut AppBuilder) {
        app.init_resource::<Events<E>>();
    }
}
