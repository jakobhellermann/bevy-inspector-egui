use egui::FontId;

use crate::egui_utils::layout_job;

pub fn error_message_reflect_value_no_impl(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " is "),
        (FontId::monospace(14.0), "#[reflect_value]"),
        (FontId::default(), ", but has no "),
        (FontId::monospace(14.0), "InspectorEguiImpl"),
        (FontId::default(), " registered in the "),
        (FontId::monospace(14.0), "TypeRegistry"),
        (FontId::default(), " ."),
    ]);

    ui.label(job);
}
pub fn error_message_no_default_value(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has no "),
        (FontId::monospace(14.0), "ReflectDefault"),
        (
            FontId::default(),
            " type data, so no value of it can be constructed.",
        ),
    ]);

    ui.label(job);
}

pub fn error_message_unconstructable_variants(
    ui: &mut egui::Ui,
    type_name: &str,
    unconstructable_variants: &[&str],
) {
    let mut vec = Vec::with_capacity(2 + unconstructable_variants.len() * 2 + 3);
    vec.extend([
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has unconstructable variants: "),
    ]);
    vec.extend(unconstructable_variants.iter().flat_map(|variant| {
        [
            (FontId::monospace(14.0), *variant),
            (FontId::default(), ", "),
        ]
    }));
    vec.extend([
        (FontId::default(), "\nyou should register "),
        (FontId::monospace(14.0), "ReflectDefault"),
        (FontId::default(), " for all fields."),
    ]);
    let job = layout_job(&vec);

    ui.label(job);
}

pub fn error_message_not_in_type_registry(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " is not registered in the "),
        (FontId::monospace(14.0), "TypeRegistry"),
    ]);

    ui.label(job);
}

pub fn error_message_no_multiedit(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " doesn't support multi-editing."),
    ]);

    ui.label(job);
}
