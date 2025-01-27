use std::time::Instant;

use egui::{pos2, vec2, Key, PointerButton, Pos2, RawInput};
use winit::keyboard::{KeyCode, ModifiersState};
use winit::{event::MouseButton, keyboard::PhysicalKey};

/// Contains and manages everything related to the [`egui`] input
///
/// such as the location of the mouse or the pressed keys
pub struct Input {
    dt: Instant,
    pointer_pos: Pos2,
    pub(crate) raw: RawInput,
    pub(crate) scale_factor: f32,
}

impl Default for Input {
    /// scale_factor: 1.0
    fn default() -> Self {
        Self {
            dt: Instant::now(),
            pointer_pos: Default::default(),
            raw: Default::default(),
            scale_factor: 1.0,
        }
    }
}

impl Input {
    pub(crate) fn take(&mut self) -> RawInput {
        self.raw.predicted_dt = self.dt.elapsed().as_secs_f32();
        self.dt = Instant::now();
        self.raw.take()
    }

    /// It updates egui of what is happening in the input (keys pressed, mouse position, etc), but it doesn't updates
    /// the information of the pressed characters, to update that information you have to
    /// use the function [text_input_event](Input:: text_input_event)
    pub fn update(&mut self, ctx: &ggez::Context) {
        /*======================= Keyboard =======================*/
        for key in ctx.keyboard.pressed_physical_keys.iter() {
            if ctx.keyboard.is_physical_key_just_pressed(key) {
                if let Some(key) = translate_physical_key(*key) {
                    self.raw.events.push(egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        modifiers: translate_modifier(ctx.keyboard.active_modifiers),
                    })
                }
            }
        }

        /*======================= Mouse =======================*/
        let ggez::mint::Point2 { x, y } = ctx.mouse.position();
        self.pointer_pos = pos2(x / self.scale_factor, y / self.scale_factor);
        self.raw
            .events
            .push(egui::Event::PointerMoved(self.pointer_pos));

        for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
            if ctx.mouse.button_just_pressed(button) {
                self.raw.events.push(egui::Event::PointerButton {
                    button: match button {
                        MouseButton::Left => PointerButton::Primary,
                        MouseButton::Right => PointerButton::Secondary,
                        MouseButton::Middle => PointerButton::Middle,
                        _ => unreachable!(),
                    },
                    pos: self.pointer_pos,
                    pressed: true,
                    modifiers: translate_modifier(ctx.keyboard.active_modifiers),
                });
            } else if ctx.mouse.button_just_released(button) {
                self.raw.events.push(egui::Event::PointerButton {
                    button: match button {
                        MouseButton::Left => PointerButton::Primary,
                        MouseButton::Right => PointerButton::Secondary,
                        MouseButton::Middle => PointerButton::Middle,
                        _ => unreachable!(),
                    },
                    pos: self.pointer_pos,
                    pressed: false,
                    modifiers: translate_modifier(ctx.keyboard.active_modifiers),
                });
            }
        }
    }

    /// Set the scale_factor and update the screen_rect
    pub fn set_scale_factor(&mut self, scale_factor: f32, (w, h): (f32, f32)) {
        self.scale_factor = scale_factor;
        self.resize_event(w, h);
    }

    /// Update screen_rect data with window size
    pub fn resize_event(&mut self, w: f32, h: f32) {
        self.raw.screen_rect = Some(egui::Rect::from_min_size(Default::default(), vec2(w, h)));
    }

    /// lets you know the rotation of the mouse wheel
    pub fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.raw.events.push(egui::Event::Scroll(vec2(x, y)));
    }

    /// lets know what character is pressed on the keyboard
    pub fn text_input_event(&mut self, ch: char) {
        if is_printable(ch) {
            self.raw.events.push(egui::Event::Text(ch.to_string()));
        }
    }
}

#[inline]
fn translate_physical_key(key: PhysicalKey) -> Option<egui::Key> {
    let PhysicalKey::Code(key) = key else {
        return None;
    };
    Some(match key {
        KeyCode::Escape => Key::Escape,
        KeyCode::Insert => Key::Insert,
        KeyCode::Home => Key::Home,
        KeyCode::Delete => Key::Delete,
        KeyCode::End => Key::End,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowUp => Key::ArrowUp,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Space => Key::Space,

        KeyCode::KeyA => Key::A,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyZ => Key::Z,

        _ => {
            return None;
        }
    })
}

#[inline]
fn translate_modifier(keymods: ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: keymods.alt_key(),
        ctrl: keymods.control_key(),
        shift: keymods.shift_key(),
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        #[cfg(not(target_os = "macos"))]
        command: keymods.control_key(),
        #[cfg(target_os = "macos")]
        mac_cmd: keymods.super_key(),
        #[cfg(target_os = "macos")]
        command: keymods.super_key(),
    }
}

#[inline]
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}
