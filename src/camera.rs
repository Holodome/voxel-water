use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::{CameraDTO, MapDTO};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Camera {
    aspect_ratio: f32,
    vfov: f32,
    znear: f32,
    zfar: f32,
    projection_matrix: Matrix4,
    inverse_projection_matrix: Matrix4,

    pitch: f32,
    yaw: f32,
    position: Vector3,
    view_matrix: Matrix4,
}

impl Camera {
    pub fn new(aspect_ratio: f32, vfov: f32, znear: f32, zfar: f32) -> Self {
        let projection_matrix =
            *nalgebra::Perspective3::new(aspect_ratio, vfov, znear, zfar).as_matrix();
        let inverse_projection_matrix = projection_matrix.try_inverse().unwrap();
        let view_matrix = Matrix4::identity();
        Self {
            aspect_ratio,
            vfov,
            znear,
            zfar,
            pitch: 0.0,
            yaw: 0.0,
            position: Vector3::new(0.0, 0.0, 0.0),
            view_matrix,
            projection_matrix,
            inverse_projection_matrix,
        }
    }

    pub fn rotate(&mut self, yaw_d: f32, pitch_d: f32) {
        self.pitch += pitch_d;
        self.yaw += yaw_d;
        self.update_view_matrix();
    }
    pub fn translate(&mut self, translation: Vector3) {
        let rotation = Quat::from_euler_angles(self.yaw, self.pitch, 0.0);
        let actual_translation = rotation.transform_vector(&translation);
        self.position += actual_translation;
        self.update_view_matrix();
    }

    fn update_view_matrix(&mut self) {
        let rotation = Matrix4::from_euler_angles(self.yaw, self.pitch, 0.0);
        self.view_matrix = Matrix4::new_translation(&self.position) * rotation;
    }

    pub fn as_dto(&self) -> CameraDTO {
        CameraDTO {
            view_matrix: self.view_matrix,
            projection_matrix: self.projection_matrix,
            inverse_projection_matrix: self.inverse_projection_matrix,
        }
    }
}
