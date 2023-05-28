use egui::FontId;

use crate::egui_utils::layout_job;

pub fn reflect_value_no_impl(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " is "),
        (FontId::monospace(12.0), "#[reflect_value]"),
        (FontId::proportional(13.0), ", but has no "),
        (FontId::monospace(12.0), "InspectorEguiImpl"),
        (FontId::proportional(13.0), " registered in the "),
        (FontId::monospace(12.0), "TypeRegistry"),
        (FontId::proportional(13.0), " .\n"),
        (FontId::proportional(13.0), "Try calling "),
        (
            FontId::monospace(12.0),
            &format!(
                ".register_type::<{}>",
                pretty_type_name::pretty_type_name_str(type_name)
            ),
        ),
        (FontId::proportional(13.0), " or add the "),
        (FontId::monospace(12.0), "DefaultInspectorConfigPlugin"),
        (FontId::proportional(13.0), " for builtin types."),
    ]);

    ui.label(job);
}
pub fn no_default_value(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " has no "),
        (FontId::monospace(12.0), "ReflectDefault"),
        (
            FontId::proportional(13.0),
            " type data, so no value of it can be constructed.",
        ),
    ]);

    ui.label(job);
}

pub fn unconstructable_variant(
    ui: &mut egui::Ui,
    type_name: &str,
    variant: &str,
    unconstructable_field_types: &[&str],
) {
    let mut vec = Vec::with_capacity(2 + unconstructable_field_types.len() * 2 + 4);

    let qualified_variant = format!(
        "{}::{}",
        pretty_type_name::pretty_type_name_str(type_name),
        variant
    );
    vec.extend([
        (FontId::monospace(12.0), qualified_variant.as_str()),
        (
            FontId::proportional(13.0),
            " has unconstructable fields.\nConsider adding ",
        ),
        (FontId::monospace(12.0), "#[reflect(Default)]"),
        (FontId::proportional(13.0), " to\n\n"),
    ]);
    vec.extend(unconstructable_field_types.iter().flat_map(|variant| {
        [
            (FontId::proportional(13.0), "- "),
            (FontId::monospace(12.0), *variant),
        ]
    }));

    let job = layout_job(&vec);

    ui.label(job);
}

pub fn not_in_type_registry(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " is not registered in the "),
        (FontId::monospace(12.0), "TypeRegistry"),
    ]);

    ui.label(job);
}

pub fn no_multiedit(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), type_name),
        (
            FontId::proportional(13.0),
            " doesn't support multi-editing.",
        ),
    ]);

    ui.label(job);
}
