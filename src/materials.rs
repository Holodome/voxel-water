use crate::math::*;
use crate::renderer::MaterialDTO;

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Diffuse {
        albedo: Vector3,
    },
    Metal {
        albedo: Vector3,
        fuzz: f32,
    },
    Dielectric {
        albedo: Vector3,
        refractive_index: f32,
    },
}

impl Material {
    pub fn diffuse(albedo: Vector3) -> Self {
        Self::Diffuse { albedo }
    }
    pub fn metal(albedo: Vector3, fuzz: f32) -> Self {
        Self::Metal { albedo, fuzz }
    }
    pub fn dielectric(albedo: Vector3, refractive_index: f32) -> Self {
        Self::Dielectric {
            albedo,
            refractive_index,
        }
    }

    fn kind(&self) -> i32 {
        match self {
            Self::Diffuse { .. } => 0,
            Self::Metal { .. } => 1,
            Self::Dielectric { .. } => 2,
        }
    }

    pub fn as_dto(&self) -> MaterialDTO {
        match self {
            Material::Diffuse { albedo } => MaterialDTO {
                albedo: *albedo,
                fuzz: 0.0,
                refractive_index: 0.0,
                kind: self.kind(),
            },
            Material::Metal { albedo, fuzz } => MaterialDTO {
                albedo: *albedo,
                fuzz: *fuzz,
                refractive_index: 0.0,
                kind: self.kind(),
            },
            Material::Dielectric {
                albedo,
                refractive_index,
            } => MaterialDTO {
                albedo: *albedo,
                fuzz: 0.0,
                refractive_index: *refractive_index,
                kind: self.kind(),
            },
        }
    }
}
