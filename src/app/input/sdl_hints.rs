use sdl3::hint;

pub const SDL_HINTS: &[(&str, &str)] = &[
    (hint::names::JOYSTICK_ALLOW_BACKGROUND_EVENTS, "1"),
    // TODO: check
];
