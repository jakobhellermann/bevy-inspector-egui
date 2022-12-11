use bevy::input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
    mouse::MouseButton,
};

use crate::{options::NumberAttributes, Context, Inspectable};

impl_for_simple_enum!(
    KeyCode: Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Snapshot,
    Scroll,
    Pause,
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    Back,
    Return,
    Space,
    Compose,
    Caret,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    AbntC1,
    AbntC2,
    NumpadAdd,
    Apostrophe,
    Apps,
    Asterisk,
    Plus,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    NumpadDecimal,
    NumpadDivide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    NumpadMultiply,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    Oem102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    NumpadSubtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut
);

impl Inspectable for MouseButton {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _: Self::Attributes,
        context: &mut Context,
    ) -> bool {
        use std::mem::discriminant;

        let mut changed = false;
        ui.vertical(|ui| {
            bevy_egui::egui::ComboBox::from_id_source(context.id())
                .selected_text(mouse_button_text(self))
                .show_ui(ui, |ui| {
                    for button in
                        [MouseButton::Left, MouseButton::Right, MouseButton::Middle].iter()
                    {
                        if ui
                            .selectable_label(*button == *self, mouse_button_text(button))
                            .clicked()
                        {
                            *self = *button;
                            changed = true;
                        }
                    }

                    if ui
                        .selectable_label(
                            discriminant(self) == discriminant(&MouseButton::Other(0)),
                            "Other",
                        )
                        .clicked()
                    {
                        *self = MouseButton::Other(0);
                        changed = true;
                    }
                });

            // Add support for other mouse buttons
            if discriminant(self) == discriminant(&MouseButton::Other(0)) {
                let mut value = match self {
                    MouseButton::Other(val) => *val,
                    _ => 0,
                };

                let attrs = NumberAttributes::default();
                if value.ui(ui, attrs, context) {
                    changed = true;
                    *self = MouseButton::Other(value);
                }
            }
        });

        changed
    }
}

fn mouse_button_text(value: &MouseButton) -> &str {
    match value {
        MouseButton::Left => "Left",
        MouseButton::Right => "Right",
        MouseButton::Middle => "Middle",
        MouseButton::Other(_) => "Other",
    }
}

impl_for_simple_enum!(
    GamepadButtonType: South,
    East,
    North,
    West,
    C,
    Z,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight
);

impl_for_simple_enum!(
    GamepadAxisType: LeftStickX,
    LeftStickY,
    LeftZ,
    RightStickX,
    RightStickY,
    RightZ
);
