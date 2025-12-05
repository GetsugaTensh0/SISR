use sdl3::hint;

pub const SDL_HINTS: &[(&str, &str)] = &[
    (hint::names::JOYSTICK_ALLOW_BACKGROUND_EVENTS, "1"),
    (hint::names::HIDAPI_IGNORE_DEVICES, ""),
    (hint::names::GAMECONTROLLER_IGNORE_DEVICES, ""),
    (hint::names::JOYSTICK_RAWINPUT, "1"),
    (hint::names::JOYSTICK_RAWINPUT_CORRELATE_XINPUT, "1"),
    // TODO: check
];
