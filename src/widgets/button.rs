use crate::{Context, Inspectable};
use bevy::app::Events;
use bevy::{ecs::component::Component, prelude::*};
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

impl<E: Component + Default> Inspectable for InspectableButton<E> {
    type Attributes = ButtonAttributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let world = expect_world!(ui, context, "InspectableButton");
        let mut events = world.get_resource_mut::<Events<E>>().unwrap();

        if ui.button(options.text).clicked() {
            events.send(E::default());
        }

        false
    }

    fn setup(app: &mut App) {
        app.add_event::<E>();
    }
}
