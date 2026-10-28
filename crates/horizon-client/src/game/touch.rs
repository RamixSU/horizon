//! Touch controls (Android; ANDROID.md), laid out like Bedrock Edition's.
//!
//! In menus a finger is the mouse (down = left click, moving drags). In the
//! world the controls are drawn over the HUD (`draw_touch_controls`):
//! a joystick bottom left (the controller's left stick, through the virtual
//! pad `horizon-ctl pad` uses), jump and sneak bottom right, pause, chat and
//! keyboard top right, inventory and drop beside the hotbar, and the hotbar
//! slots themselves select on tap. Anywhere else turns the view; a short tap
//! there uses (right click) and a finger held still breaks (left click), as
//! on Bedrock. Buttons go through the controller bindings.
use super::Game;
use horizon_client::{
    gamepad::Button,
    hud::{HudAtlas, HudBatch, rgba},
};
use std::{collections::HashMap, time::Instant};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, MouseButton, Touch, TouchPhase, WindowEvent},
    event_loop::ActiveEventLoop,
};

/// Held this long without moving, a look finger breaks blocks.
const HOLD_MS: u128 = 280;
/// Moved further than this (GUI pixels), a look finger is no tap or hold.
const SLOP: f64 = 4.0;

/// What a finger is doing, from where it went down.
#[derive(Clone, Copy, Debug)]
enum Finger {
    /// Acting as the mouse in a menu.
    Pointer,
    /// The joystick; the knob's position (GUI pixels).
    Stick,
    /// Turning the view; tap = use, hold = attack.
    Look {
        last: [f64; 2],
        start: [f64; 2],
        since: Instant,
        moved: bool,
        attacking: bool,
    },
    /// Holding a controller button.
    Press(Button),
    /// Took over a hotbar slot or the keyboard; nothing until it lifts.
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Control {
    Pad(Button),
    Keyboard,
}

#[derive(Default)]
pub(super) struct Touches {
    fingers: HashMap<u64, Finger>,
    /// The joystick knob's offset from the centre, -1..1.
    knob: [f32; 2],
    /// Frames until a tapped use is released.
    use_release: u8,
    /// Sneak toggled on (Bedrock's sneak button toggles).
    sneaking: bool,
}

/// The controls' places in GUI pixels for a `w` x `h` GUI.
struct Layout {
    w: f32,
    h: f32,
}
impl Layout {
    fn stick(&self) -> ([f32; 2], f32) {
        ([42.0, self.h - 46.0], 32.0)
    }
    fn hotbar_x(&self) -> f32 {
        (self.w / 2.0 - 91.0).floor()
    }
    /// Round buttons: centre, radius, control, label.
    fn buttons(&self) -> [([f32; 2], f32, Control, &'static str); 7] {
        let (w, h) = (self.w, self.h);
        let bar = self.hotbar_x() + 182.0;
        [
            ([w - 30.0, h - 42.0], 18.0, Control::Pad(Button::South), "^"),
            ([w - 70.0, h - 24.0], 13.0, Control::Pad(Button::RightStick), "v"),
            ([w - 12.0, 12.0], 9.0, Control::Pad(Button::Start), "II"),
            ([w - 34.0, 12.0], 9.0, Control::Pad(Button::DPadRight), "T"),
            ([w - 56.0, 12.0], 9.0, Control::Keyboard, "Kb"),
            ([bar + 13.0, h - 11.0], 10.0, Control::Pad(Button::North), "..."),
            ([bar + 13.0, h - 34.0], 10.0, Control::Pad(Button::East), "Q"),
        ]
    }
    /// The close (back) button over menus and containers, top right.
    fn close(&self) -> ([f32; 2], f32) {
        ([self.w - 12.0, 12.0], 9.0)
    }
    fn close_hit(&self, p: [f32; 2]) -> bool {
        let (c, r) = self.close();
        (p[0] - c[0]).hypot(p[1] - c[1]) <= r + 5.0
    }
    fn hit(&self, p: [f32; 2]) -> Option<Control> {
        self.buttons().into_iter().find_map(|(c, r, control, _)| {
            let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
            (dx * dx + dy * dy <= (r + 4.0) * (r + 4.0)).then_some(control)
        })
    }
    fn hotbar_slot(&self, p: [f32; 2]) -> Option<usize> {
        let x = p[0] - self.hotbar_x();
        (p[1] >= self.h - 23.0 && (0.0..182.0).contains(&x)).then(|| ((x / 20.0) as usize).min(8))
    }
    fn in_stick(&self, p: [f32; 2]) -> bool {
        p[0] < self.w * 0.35 && p[1] > self.h * 0.45
    }
}

fn disc(batch: &mut HudBatch, atlas: &HudAtlas, c: [f32; 2], r: f32, color: u32) {
    const N: usize = 24;
    let at = |i: usize| {
        let a = i as f32 / N as f32 * std::f32::consts::TAU;
        [c[0] + r * a.cos(), c[1] + r * a.sin()]
    };
    for i in 0..N {
        batch.fill_quad(atlas, [c, at(i), at(i + 1), c], color);
    }
}

fn ring(batch: &mut HudBatch, atlas: &HudAtlas, c: [f32; 2], r: f32, t: f32, color: u32) {
    const N: usize = 32;
    let at = |i: usize, r: f32| {
        let a = i as f32 / N as f32 * std::f32::consts::TAU;
        [c[0] + r * a.cos(), c[1] + r * a.sin()]
    };
    for i in 0..N {
        batch.fill_quad(atlas, [at(i, r - t), at(i, r), at(i + 1, r), at(i + 1, r - t)], color);
    }
}

impl Game<'_> {
    fn touch_layout(&self) -> Option<(Layout, f32)> {
        let batch = self.gui_scale()?;
        Some((Layout { w: batch.width, h: batch.height }, batch.scale))
    }

    fn touch_controls_shown(&self) -> bool {
        cfg!(target_os = "android") && self.player.is_some() && !self.pad_in_menu()
    }

    /// Over a screen in the world (inventory, containers, pause, chat...):
    /// a close button, as Escape.
    fn touch_close_shown(&self) -> bool {
        cfg!(target_os = "android") && self.player.is_some() && self.title.is_none() && self.pad_in_menu()
    }

    /// Escape: closes a screen, or pauses (the controller's Start, tapped).
    pub(super) fn touch_back(&mut self) {
        self.pad.injected.set(Button::Start, true);
        self.pad.taps.push(Button::Start);
    }

    /// The controls over the HUD, while playing.
    pub(super) fn draw_touch_controls(&self, batch: &mut HudBatch) {
        if self.touch_close_shown() {
            let layout = Layout { w: batch.width, h: batch.height };
            let (c, r) = layout.close();
            let atlas = &self.gui.atlas;
            disc(batch, atlas, c, r, rgba(0, 0, 0, 150));
            ring(batch, atlas, c, r, 1.0, rgba(255, 255, 255, 160));
            let tw = HudBatch::text_width(atlas, "X");
            batch.text(atlas, "X", c[0] - tw / 2.0, c[1] - 4.0, 0xFFFF_FFFF, true);
        }
        if !self.touch_controls_shown() {
            return;
        }
        let atlas = &self.gui.atlas;
        let layout = Layout { w: batch.width, h: batch.height };
        let (c, r) = layout.stick();
        disc(batch, atlas, c, r, rgba(0, 0, 0, 70));
        ring(batch, atlas, c, r, 1.5, rgba(255, 255, 255, 110));
        let k = self.touch.knob;
        let knob = [c[0] + k[0] * r * 0.6, c[1] - k[1] * r * 0.6];
        let held = self.touch.fingers.values().any(|f| matches!(f, Finger::Stick));
        disc(batch, atlas, knob, 13.0, rgba(255, 255, 255, if held { 170 } else { 110 }));
        for (c, r, control, label) in layout.buttons() {
            let down = match control {
                Control::Pad(Button::RightStick) => self.touch.sneaking,
                Control::Pad(b) => self.pad.injected.held(b),
                Control::Keyboard => false,
            };
            disc(batch, atlas, c, r, rgba(0, 0, 0, if down { 140 } else { 80 }));
            ring(batch, atlas, c, r, 1.0, rgba(255, 255, 255, 120));
            let tw = HudBatch::text_width(atlas, label);
            batch.text(atlas, label, c[0] - tw / 2.0, c[1] - 4.0, 0xE0FF_FFFF, true);
        }
    }

    /// Once per loop iteration, before the pads are read: a tapped use
    /// lets go, a finger held still starts breaking.
    pub(super) fn touch_frame(&mut self) {
        if self.touch.use_release > 0 {
            self.touch.use_release -= 1;
            if self.touch.use_release == 0 {
                self.pad.injected.triggers[0] = 0.0;
            }
        }
        let mut attack = false;
        for finger in self.touch.fingers.values_mut() {
            if let Finger::Look { since, moved, attacking, .. } = finger
                && !*moved
                && since.elapsed().as_millis() >= HOLD_MS
            {
                *attacking = true;
            }
            if let Finger::Look { attacking: true, .. } = finger {
                attack = true;
            }
        }
        self.pad.injected.triggers[1] = if attack { 1.0 } else { 0.0 };
    }

    pub(super) fn touch_event(&mut self, event_loop: &ActiveEventLoop, touch: Touch) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some((layout, scale)) = self.touch_layout() else {
            return;
        };
        let pos = [touch.location.x, touch.location.y];
        let gui = [(pos[0] / f64::from(scale)) as f32, (pos[1] / f64::from(scale)) as f32];
        let id = touch.id;
        let device_id = touch.device_id;
        let window_id = window.id();
        let send = |game: &mut Self, event: WindowEvent| {
            game.window_event(event_loop, window_id, event);
        };
        match touch.phase {
            TouchPhase::Started => {
                let finger = if self.touch_close_shown() && layout.close_hit(gui) {
                    self.touch_back();
                    Finger::Done
                } else if !self.touch_controls_shown() {
                    send(self, cursor(device_id, pos));
                    send(self, click(device_id, ElementState::Pressed));
                    Finger::Pointer
                } else if let Some(control) = layout.hit(gui) {
                    match control {
                        Control::Keyboard => {
                            self.show_keyboard(true);
                            Finger::Done
                        }
                        Control::Pad(Button::RightStick) => {
                            self.touch.sneaking = !self.touch.sneaking;
                            self.pad.injected.set(Button::RightStick, self.touch.sneaking);
                            Finger::Done
                        }
                        Control::Pad(button) => {
                            self.pad.injected.set(button, true);
                            if button == Button::DPadRight {
                                // Chat: the soft keyboard comes with it.
                                self.show_keyboard(true);
                            }
                            Finger::Press(button)
                        }
                    }
                } else if let Some(slot) = layout.hotbar_slot(gui) {
                    self.selected = slot;
                    Finger::Done
                } else if layout.in_stick(gui) {
                    self.touch_stick(&layout, gui);
                    Finger::Stick
                } else {
                    Finger::Look {
                        last: pos,
                        start: pos,
                        since: Instant::now(),
                        moved: false,
                        attacking: false,
                    }
                };
                self.touch.fingers.insert(id, finger);
            }
            TouchPhase::Moved => match self.touch.fingers.get(&id).copied() {
                Some(Finger::Pointer) => send(self, cursor(device_id, pos)),
                Some(Finger::Stick) => self.touch_stick(&layout, gui),
                Some(Finger::Look { last, start, since, moved, attacking }) => {
                    let (dx, dy) = (pos[0] - last[0], pos[1] - last[1]);
                    let far = ((pos[0] - start[0]).hypot(pos[1] - start[1])) / f64::from(scale) > SLOP;
                    self.touch.fingers.insert(
                        id,
                        Finger::Look { last: pos, start, since, moved: moved || far, attacking },
                    );
                    self.mouse.on_move(0.0, 0.0, dx, dy, true, true);
                    self.last_input = Instant::now();
                }
                _ => {}
            },
            TouchPhase::Ended | TouchPhase::Cancelled => match self.touch.fingers.remove(&id) {
                Some(Finger::Pointer) => {
                    send(self, cursor(device_id, pos));
                    send(self, click(device_id, ElementState::Released));
                }
                Some(Finger::Stick) => {
                    self.touch.knob = [0.0, 0.0];
                    self.pad.injected.left = [0.0, 0.0];
                }
                Some(Finger::Press(button)) => self.pad.injected.set(button, false),
                Some(Finger::Look { moved: false, attacking: false, .. })
                    if touch.phase == TouchPhase::Ended =>
                {
                    // A tap: use, held for a couple of frames so a tick sees it.
                    self.pad.injected.triggers[0] = 1.0;
                    self.touch.use_release = 3;
                }
                _ => {}
            },
        }
    }

    fn touch_stick(&mut self, layout: &Layout, p: [f32; 2]) {
        let (c, r) = layout.stick();
        let mut v = [(p[0] - c[0]) / (r * 0.6), (c[1] - p[1]) / (r * 0.6)];
        let len = v[0].hypot(v[1]);
        if len > 1.0 {
            v = [v[0] / len, v[1] / len];
        }
        self.touch.knob = v;
        self.pad.injected.left = v;
    }

    /// The Android soft keyboard (its keys arrive as key events).
    fn show_keyboard(&self, show: bool) {
        #[cfg(target_os = "android")]
        if let Some(app) = super::ANDROID_APP.get() {
            if show {
                app.show_soft_input(true);
            } else {
                app.hide_soft_input(true);
            }
        }
        #[cfg(not(target_os = "android"))]
        let _ = show;
    }
}

fn cursor(device_id: DeviceId, p: [f64; 2]) -> WindowEvent {
    WindowEvent::CursorMoved {
        device_id,
        position: PhysicalPosition::new(p[0], p[1]),
    }
}

fn click(device_id: DeviceId, state: ElementState) -> WindowEvent {
    WindowEvent::MouseInput {
        device_id,
        state,
        button: MouseButton::Left,
    }
}
