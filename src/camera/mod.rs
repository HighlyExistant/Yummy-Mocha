#![allow(unused)]
use drowsed_math::{FMat4, FVec3, Vector, EuclideanGeometry, SquareMatrix};

pub struct Camera {
    pub projection: FMat4,
    pub view: FMat4,
}
impl Default for Camera{
    fn default() -> Self {
        let projection = FMat4::identity();
        let view = FMat4::identity();
        Self { projection, view }
    }
}
impl Camera {
    pub fn new(projection: FMat4, view: FMat4) -> Self{
        Self { projection, view }
    }
    pub fn set_orthographic_projection(&mut self, left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) {
        self.projection = FMat4::identity();
        self.projection.x.x = 2.0 / (right - left);
        self.projection.y.y = 2.0 / (bottom - top);
        self.projection.z.z = 1.0 / (far - near);
        self.projection.w.x = -(right + left) / (right - left);
        self.projection.w.y = -(bottom + top) / (bottom - top);
        self.projection.w.z = -near / (far - near);
    }
    pub fn set_perspective_projection(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
        let tan_half_fovy = f32::tan(fovy / 2.0);
        self.projection = FMat4::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        self.projection.x.x = 1.0 / (aspect * tan_half_fovy);
        self.projection.y.y = 1.0 / (tan_half_fovy);
        self.projection.z.z = far / (far - near);
        self.projection.z.w = 1.0;
        self.projection.w.z = -(far * near) / (far - near);
    }
    pub fn set_direction(&mut self, position: FVec3, direction: FVec3, up: FVec3) {
        // std::assert!(direction.x != 0.0 || direction.y != 0.0 || direction.z != 0.0, "the direction provided should not be zero");
        let w = direction.normalize();
        let u = w.cross(up).normalize();
        let v = w.cross(u);

        self.view = FMat4::identity();
        self.view.x.x = u.x;
        self.view.y.x = u.y;
        self.view.z.x = u.z;
        self.view.x.y = v.x;
        self.view.y.y = v.y;
        self.view.z.y = v.z;
        self.view.x.z = w.x;
        self.view.y.z = w.y;
        self.view.z.z = w.z;
        self.view.w.x = -u.dot(&position);
        self.view.w.y = -v.dot(&position);
        self.view.w.z = -w.dot(&position);
    }
    pub fn set_target(&mut self, position: FVec3, target: FVec3, up: FVec3) {
        self.set_direction(position, target - position, up);
    }
    pub fn set_view_yxz(&mut self, position: FVec3, rotation: FVec3) {
        let c3 = rotation.z.cos();
        let s3 = rotation.z.sin();
        let c2 = rotation.x.cos();
        let s2 = rotation.x.sin();
        let c1 = rotation.y.cos();
        let s1 = rotation.y.sin();
        let u = FVec3::new(c1 * c3 + s1 * s2 * s3, c2 * s3, c1 * s2 * s3 - c3 * s1);
        let v = FVec3::new(c3 * s1 * s2 - c1 * s3, c2 * c3, c1 * c3 * s2 + s1 * s3);
        let w = FVec3::new(c2 * s1, -s2, c1 * c2);
        self.view = FMat4::identity();
        self.view.x.x = u.x;
        self.view.y.x = u.y;
        self.view.z.x = u.z;
        self.view.x.y = v.x;
        self.view.y.y = v.y;
        self.view.z.y = v.z;
        self.view.x.z = w.x;
        self.view.y.z = w.y;
        self.view.z.z = w.z;
        self.view.w.x = -u.dot(&position);
        self.view.w.y = -v.dot(&position);
        self.view.w.z = -w.dot(&position);
    }
}