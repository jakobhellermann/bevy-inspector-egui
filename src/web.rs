use bevy::prelude::*;

use crossbeam_channel::Receiver;
use winit::dpi::LogicalSize;

use bevy::winit::WinitWindows;
#[cfg(target_arch = "wasm32")]
pub fn web_resize_system(winit_windows: Res<WinitWindows>, web_resizing: Local<WinitWebResizing>) {
    use bevy::window::WindowId;
    let winit_window = winit_windows.get_window(WindowId::primary()).unwrap();
    for size in web_resizing.rx.clone().try_iter().last() {
        winit_window.set_inner_size(size);
    }
}

impl Default for WinitWebResizing {
    fn default() -> Self {
        WinitWebResizing::new()
    }
}

pub struct WinitWebResizing {
    pub rx: Receiver<LogicalSize<f32>>,
}

impl WinitWebResizing {
    pub fn new() -> Self {
        use bevy::log;
        use wasm_bindgen::JsCast;
        let (tx, rx) = crossbeam_channel::unbounded();

        let get_full_size = || {
            let win = web_sys::window().unwrap();
            // `inner_width` corresponds to the browser's `self.innerWidth` function, which are in
            // Logical, not Physical, pixels
            winit::dpi::LogicalSize::new(
                win.inner_width().unwrap().as_f64().unwrap() as f32,
                win.inner_height().unwrap().as_f64().unwrap() as f32,
            )
        };

        tx.send(get_full_size()).unwrap();

        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
            log::debug!("handling resize event: {:?}", e);
            tx.send(get_full_size()).unwrap();
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();

        return Self { rx };
    }
}
