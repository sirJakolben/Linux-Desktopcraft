use glam::{Mat4, Vec3};

/// Free-flying first-person camera (Minecraft-style: horizontal movement
/// ignores pitch, Space/Shift move straight up and down).
pub struct Camera {
    pub position: Vec3,
    /// Rotation around +Y in radians; 0 looks along -Z.
    pub yaw: f32,
    /// Rotation up/down in radians, clamped to just below ±90°.
    pub pitch: f32,
    pub fov_y: f32,
}

const MOUSE_SENSITIVITY: f32 = 0.0025;
const PITCH_LIMIT: f32 = 1.55;

impl Camera {
    pub fn new(position: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
            fov_y: 70f32.to_radians(),
        }
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }

    pub fn rotate(&mut self, dx: f64, dy: f64) {
        self.yaw += dx as f32 * MOUSE_SENSITIVITY;
        self.pitch = (self.pitch - dy as f32 * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Moves the camera. `local` is (right, up, forward) in units of `distance`.
    pub fn translate(&mut self, local: Vec3, distance: f32) {
        if local == Vec3::ZERO {
            return;
        }
        let forward = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = forward.cross(Vec3::Y);
        let dir = right * local.x + Vec3::Y * local.y + forward * local.z;
        self.position += dir.normalize() * distance;
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        let view = Mat4::look_to_rh(self.position, self.forward(), Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov_y, aspect, 0.05, 500.0);
        proj * view
    }
}
