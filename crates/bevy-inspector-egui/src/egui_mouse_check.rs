use bevy_ecs::prelude::*;
use bevy_egui::EguiContexts;
use bevy_log::error;
use bevy_window::PrimaryWindow;

#[derive(Resource)]
pub struct EguiMousePointerCheck {
    pointer_is_valid: bool,
    primary_window: Option<Entity>,
}

impl Default for EguiMousePointerCheck {
    fn default() -> EguiMousePointerCheck {
        EguiMousePointerCheck {
            pointer_is_valid: true,
            primary_window: None,
        }
    }
}

pub fn initialize_egui_mouse_check(
    mut egui_check: ResMut<EguiMousePointerCheck>,
    window_q: Query<Entity, With<PrimaryWindow>>,
) {
    if let Ok(window_id) = window_q.get_single() {
        egui_check.primary_window = Some(window_id);
    } else {
        error!("could not get Primary Window");
    }
}

pub fn update_egui_mouse_check(
    mut egui_checker: ResMut<EguiMousePointerCheck>,
    mut egui_ctxs: EguiContexts,
) {
    if let Some(window_id) = egui_checker.primary_window {
        egui_checker.pointer_is_valid = !egui_ctxs
            .ctx_for_window_mut(window_id)
            .wants_pointer_input();
    }
}

pub fn mouse_pointer_valid() -> impl FnMut(Res<EguiMousePointerCheck>) -> bool + Clone {
    move |egui_mouse_check: Res<EguiMousePointerCheck>| egui_mouse_check.pointer_is_valid
}
