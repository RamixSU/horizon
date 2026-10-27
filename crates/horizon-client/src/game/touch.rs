//! Touch controls (Android; ANDROID.md). In menus a finger is the mouse: down
//! is a left click at that point, moving drags. In the world the screen is
//! split: the left half is a floating move stick (the controller's left
//! stick, through the virtual pad `horizon-ctl pad` uses), the right half
//! turns the view like the mouse, and a column of zones on the right edge
//! presses controller buttons, so they follow the controller bindings.
//!
//! The zones are not drawn yet; their layout (fractions of the screen,
//! from the right edge):
//!
//! ```text
//!  top-right corner          Start (pause)        | North (inventory)
//!  right edge, upper middle  RightBumper (next hotbar slot)
//!  right edge, middle        LeftTrigger (use / place)
//!  right edge, lower middle  RightTrigger (attack / break)
//!  bottom-right corner       South (jump)         | RightStick (sneak)
//! ```
use super::Game;
use horizon_client::gamepad::Button;
use std::collections::HashMap;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, MouseButton, Touch, TouchPhase, WindowEvent},
    event_loop::ActiveEventLoop,
};

/// What a finger is doing, from where it went down.
#[derive(Clone, Copy, Debug)]
enum Finger {
    /// Acting as the mouse in a menu.
    Pointer,
    /// The move stick, centred where the finger went down.
    Stick { origin: [f64; 2] },
    /// Turning the view; the last position.
    Look { last: [f64; 2] },
    /// Holding a controller button.
    Press(Button),
}

#[derive(Default)]
pub(super) struct Touches {
    fingers: HashMap<u64, Finger>,
}

/// The button zone under `[x, y]` (fractions of the window), if any.
fn zone(x: f64, y: f64) -> Option<Button> {
    if y < 0.15 {
        return if x > 0.92 {
            Some(Button::Start)
        } else if x > 0.84 {
            Some(Button::North)
        } else {
            None
        };
    }
    if x < 0.86 {
        return None;
    }
    Some(match y {
        y if y < 0.35 => Button::RightBumper,
        y if y < 0.55 => Button::LeftTrigger,
        y if y < 0.75 => Button::RightTrigger,
        _ if x > 0.93 => Button::South,
        _ => Button::RightStick,
    })
}

impl Game<'_> {
    pub(super) fn touch_event(&mut self, event_loop: &ActiveEventLoop, touch: Touch) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let size = window.inner_size();
        let (w, h) = (f64::from(size.width.max(1)), f64::from(size.height.max(1)));
        let pos = [touch.location.x, touch.location.y];
        let id = touch.id;
        let device_id = touch.device_id;
        let window_id = window.id();
        let mut mouse = |game: &mut Self, event: WindowEvent| {
            game.window_event(event_loop, window_id, event);
        };
        match touch.phase {
            TouchPhase::Started => {
                let finger = if self.pad_in_menu() {
                    Finger::Pointer
                } else if let Some(button) = zone(pos[0] / w, pos[1] / h) {
                    Finger::Press(button)
                } else if pos[0] < w / 2.0 {
                    Finger::Stick { origin: pos }
                } else {
                    Finger::Look { last: pos }
                };
                match finger {
                    Finger::Pointer => {
                        cursor(&mut mouse, self, device_id, pos);
                        click(&mut mouse, self, device_id, ElementState::Pressed);
                    }
                    Finger::Press(button) => self.pad.injected.set(button, true),
                    _ => {}
                }
                self.touch.fingers.insert(id, finger);
            }
            TouchPhase::Moved => match self.touch.fingers.get_mut(&id).copied() {
                Some(Finger::Pointer) => cursor(&mut mouse, self, device_id, pos),
                Some(Finger::Stick { origin }) => {
                    // A full tilt at a tenth of the screen's height.
                    let reach = h * 0.1;
                    let x = ((pos[0] - origin[0]) / reach).clamp(-1.0, 1.0) as f32;
                    let y = ((origin[1] - pos[1]) / reach).clamp(-1.0, 1.0) as f32;
                    self.pad.injected.left = [x, y];
                }
                Some(Finger::Look { last }) => {
                    let (dx, dy) = (pos[0] - last[0], pos[1] - last[1]);
                    self.touch.fingers.insert(id, Finger::Look { last: pos });
                    self.mouse.on_move(0.0, 0.0, dx, dy, true, true);
                    self.last_input = std::time::Instant::now();
                }
                _ => {}
            },
            TouchPhase::Ended | TouchPhase::Cancelled => match self.touch.fingers.remove(&id) {
                Some(Finger::Pointer) => {
                    cursor(&mut mouse, self, device_id, pos);
                    click(&mut mouse, self, device_id, ElementState::Released);
                }
                Some(Finger::Stick { .. }) => self.pad.injected.left = [0.0, 0.0],
                Some(Finger::Press(button)) => self.pad.injected.set(button, false),
                _ => {}
            },
        }
    }
}

fn cursor<F: FnMut(&mut G, WindowEvent), G>(f: &mut F, g: &mut G, device_id: DeviceId, p: [f64; 2]) {
    f(
        g,
        WindowEvent::CursorMoved {
            device_id,
            position: PhysicalPosition::new(p[0], p[1]),
        },
    );
}

fn click<F: FnMut(&mut G, WindowEvent), G>(f: &mut F, g: &mut G, device_id: DeviceId, state: ElementState) {
    f(
        g,
        WindowEvent::MouseInput {
            device_id,
            state,
            button: MouseButton::Left,
        },
    );
}
