use bevy::{input::gamepad::GamepadButtonType, prelude::GamepadAxisType};

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
