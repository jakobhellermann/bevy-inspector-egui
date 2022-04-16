use crate::{
    utils::{self, error_label_needs_world},
    Context, Inspectable,
};
use bevy::{ecs::event::Events, prelude::*, text::Text2dSize};
use bevy_egui::egui::{self, Color32, RichText};

impl_for_simple_enum!(VerticalAlign: Top, Center, Bottom);
impl_for_simple_enum!(HorizontalAlign: Left, Center, Right);

impl_for_struct_delegate_fields!(TextAlignment: vertical, horizontal);
impl_for_struct_delegate_fields!(TextStyle: font, font_size, color);
impl_for_struct_delegate_fields!(TextSection: value, style);
impl_for_struct_delegate_fields!(Text: sections, alignment);
impl_for_struct_delegate_fields!(Text2dSize: size);

impl Inspectable for Handle<Font> {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let world = match context.world() {
            Some(world) => world,
            None => return error_label_needs_world(ui, "Handle<Font>"),
        };

        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let file_events = world.get_resource::<Events<FileDragAndDrop>>().unwrap();

        let fonts = world.get_resource::<Assets<Font>>().unwrap();

        let label = if fonts.contains(self.id) {
            egui::Label::new("<font>")
        } else {
            egui::Label::new(RichText::new("No font").color(Color32::RED))
        };

        if utils::ui::drag_and_drop_target_label(ui, label).hovered() {
            utils::ui::replace_handle_if_dropped(self, file_events, asset_server)
        } else {
            false
        }
    }
}
