use common::math::Vec3;

use crate::camera::{Camera, Matrices};

pub struct Scene {
    camera: Camera,
    movement_dir: Vec3<f32>,
}

// TODO: make this configurable
const FLY_CAMERA_SPEED: f32 = 3.0;

impl Scene {
    pub fn new(aspect: f32) -> Self {
        Self {
            movement_dir: Vec3::zero(),
            camera: Camera::new(aspect),
        }
    }

    pub fn look(&mut self, delta_pitch: f32, delta_yaw: f32) {
        self.camera.rotate_by(delta_pitch * 0.05, delta_yaw * 0.05);
    }

    pub fn set_movement_dir(&mut self, dir: Vec3<f32>) {
        self.movement_dir = dir;
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.camera.set_aspect_ratio(w / h);
    }

    pub fn tick(&mut self, dt: f32) {
        let dx = self.movement_dir.x * FLY_CAMERA_SPEED * dt;
        let dy = self.movement_dir.y * FLY_CAMERA_SPEED * dt;
        let dz = self.movement_dir.z * FLY_CAMERA_SPEED * dt;
        self.camera.move_by(dx, dy, dz);
    }

    pub fn camera_matrices(&mut self) -> Matrices {
        self.camera.compute_matrices()
    }
}
