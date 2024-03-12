use common::math::Vec3;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct KeyState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl KeyState {
    pub fn update(&mut self, code: KeyCode, state: bool) {
        match code {
            KeyCode::KeyW => self.forward = state,
            KeyCode::KeyS => self.backward = state,
            KeyCode::KeyA => self.left = state,
            KeyCode::KeyD => self.right = state,
            KeyCode::Space => self.up = state,
            KeyCode::ShiftLeft => self.down = state,
            _ => (),
        }
    }

    pub fn dir(&self) -> Vec3<f32> {
        Vec3::new(
            self.right as u8 as f32 - self.left as u8 as f32,
            self.up as u8 as f32 - self.down as u8 as f32,
            self.forward as u8 as f32 - self.backward as u8 as f32,
        )
    }
}
