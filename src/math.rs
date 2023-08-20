pub type Vector2 = nalgebra::Vector2<f32>;
pub type Point2 = nalgebra::Point2<f32>;
pub type Vector3 = nalgebra::Vector3<f32>;
pub type Point3 = nalgebra::Point3<f32>;
pub type Vector4 = nalgebra::Vector4<f32>;
pub type Point4 = nalgebra::Point4<f32>;
pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Quat = nalgebra::UnitQuaternion<f32>;

pub trait RandNalgebra: rand::Rng {
    fn usize_range(&mut self, low: usize, high: usize) -> usize {
        let base = self.gen::<usize>();
        low + base % (high - low)
    }

    fn bilateral(&mut self) -> f32 {
        self.gen_range(-1.0..1.0)
    }

    fn unit_vector<const L: usize>(&mut self) -> nalgebra::SVector<f32, L> {
        [(); L].map(|_| self.bilateral()).into()
    }

    fn unit_vector2(&mut self) -> Vector2 {
        self.unit_vector::<2>()
    }

    fn unit_vector3(&mut self) -> Vector3 {
        self.unit_vector::<3>()
    }

    fn unit_vector4(&mut self) -> Vector4 {
        self.unit_vector::<4>()
    }

    fn vector<const L: usize>(&mut self, low: f32, high: f32) -> nalgebra::SVector<f32, L> {
        [(); L].map(|_| self.gen_range(low..high)).into()
    }

    fn vector2(&mut self, low: f32, high: f32) -> Vector2 {
        self.vector(low, high)
    }

    fn vector3(&mut self, low: f32, high: f32) -> Vector3 {
        self.vector(low, high)
    }

    fn vector4(&mut self, low: f32, high: f32) -> Vector4 {
        self.vector(low, high)
    }

    fn unit_sphere(&mut self) -> Vector3 {
        loop {
            let result = self.unit_vector3();
            if result.dot(&result) >= 1.0 {
                break result;
            }
        }
    }

    fn unit_disk(&mut self) -> Vector2 {
        loop {
            let result = self.unit_vector2();
            if result.dot(&result) >= 1.0 {
                break result;
            }
        }
    }

    fn hemisphere(&mut self, normal: &Vector3) -> Vector3 {
        let result = self.unit_sphere();
        if result.dot(&normal) <= 0.0 {
            -result
        } else {
            result
        }
    }
}
