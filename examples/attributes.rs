use bevy::prelude::*;
use bevy_inspector_egui::{egui, Inspectable, InspectorPlugin};

#[derive(Resource, Inspectable)]
struct Data {
    #[inspectable(ignore)]
    _ignored: NotInspectable,
    #[inspectable(label = "Sheep")]
    wolf: String,
    #[inspectable(read_only)]
    change_my_view: String,
    #[inspectable(collapse)]
    oh_my: Collapsed,
    #[inspectable(wrapper = funky_ui)]
    funky: u8,
}

struct NotInspectable;

#[derive(Inspectable)]
struct Collapsed {
    o_o: (((), ()), ((), ())),
}

fn funky_ui(ui: &mut egui::Ui, mut content: impl FnMut(&mut egui::Ui)) {
    ui.scope(|ui| {
        let bg_color = egui::Color32::from_rgb(41, 80, 80);
        ui.style_mut().visuals.widgets.inactive.bg_fill = bg_color;
        ui.style_mut().visuals.widgets.active.bg_fill = bg_color;
        ui.style_mut().visuals.widgets.hovered.bg_fill = bg_color;
        ui.style_mut().visuals.widgets.noninteractive.bg_fill = bg_color;
        content(ui);
    });
}

impl Default for Data {
    fn default() -> Self {
        Self {
            _ignored: NotInspectable,
            wolf: "bah".to_string(),
            change_my_view: ":tf".to_string(),
            oh_my: Collapsed {
                o_o: (((), ()), ((), ())),
            },
            funky: 42,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .run();
}
