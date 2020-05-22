use specs::prelude::*;
use nalgebra as na;
use na::{Vector3, Matrix4, Vector4};
use std::ops::Index;
use nalgebra_glm as glm;
use glm::{translation, scaling, rotation};
use nalgebra_glm::{TVec3, RealField, look_at};
use nalgebra::{UnitQuaternion, Unit};

pub struct Position(Vector3<f32>, bool);

impl Component for Position{
    type Storage = DenseVecStorage<Self>;
}

impl Position{
    pub fn value(&mut self) -> &mut Vector3<f32>{
        &mut self.0
    }

    pub fn new(x: f32, y: f32, z:f32) -> Self{
        Position{
            0: Vector3::<f32>::new(x, y, z),
            1: true,
        }
    }
}

impl Index<usize> for Position{
    type Output = f32;

    fn index(&self, dex: usize) -> &Self::Output{
        &self.0[dex]
    }
}

pub struct Rotation(UnitQuaternion<f32>, bool);

impl Component for Rotation{
    type Storage = DenseVecStorage<Self>;
}

impl Rotation{
    pub fn new() ->Self{
        Rotation{
            0: UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), 0.0),
            1: true
        }
    }

    pub fn apply_axis_angle_rotation(&mut self, angle: f32, axis: TVec3<f32>){
        self.0 = UnitQuaternion::from_axis_angle(&Unit::new_normalize(axis), angle) * &self.0;
        self.1 = true;
    }

    pub fn forward(&self) -> Vector3<f32>{
        (&self.0 * Vector3::new(0.0, 0.0, 1.0)).xyz()
    }

    pub fn right(&self) -> Vector3<f32>{
        (&self.0 * Vector3::new(1.0, 0.0, 0.0)).xyz()
    }

    pub fn up(&self) -> Vector3<f32>{ (&self.0 * Vector3::new(0.0, 1.0, 0.0)).xyz() }
}

pub struct Scale(Vector3<f32>, bool);

impl Component for Scale{
    type Storage = DenseVecStorage<Self>;
}

impl Scale{
    pub fn value(&mut self) -> &mut Vector3<f32>{
        &mut self.0
    }
}

impl Index<usize> for Scale{
    type Output = f32;

    fn index(&self, dex: usize) -> &Self::Output{
        &self.0[dex]
    }
}

pub fn model_matrix_psr(position: &Position, scale: &Scale, rotation: &Rotation) -> Matrix4<f32>{
    let position = model_matrix_p(position);
    let scale = model_matrix_s(scale);
    let rotation = model_matrix_r(rotation);
    position * rotation * scale
}

pub fn model_matrix_pr(position: &Position, rotation: &Rotation) -> Matrix4<f32>{
    let position = model_matrix_p(position);
    let rotation = model_matrix_r(rotation);
    position * rotation
}

pub fn model_matrix_ps(position: &Position, scale: &Scale) -> Matrix4<f32>{
    let position = model_matrix_p(position);
    let scale = model_matrix_s(scale);
    position * scale
}

pub fn model_matrix_sr(rotation: &Rotation, scale: &Scale) -> Matrix4<f32>{
    let scale = scaling(&scale.0);
    let rotation = model_matrix_r(rotation);
    scale * rotation
}

pub fn model_matrix_p(position: &Position) -> Matrix4<f32>{ translation(&position.0) }

pub fn model_matrix_s(scale: &Scale) -> Matrix4<f32>{
    scaling(&scale.0)
}

pub fn model_matrix_r(rotation: &Rotation) -> Matrix4<f32>{
    let mut r = Matrix4::from(rotation.0.to_rotation_matrix());
    r[15] = 1.0;
    r
}

pub struct TransformMatrix(Matrix4<f32>);

impl Component for TransformMatrix{
    type Storage = DenseVecStorage<Self>;
}

impl Default for TransformMatrix{
    fn default() -> Self {
        TransformMatrix{
            0: Matrix4::identity()
        }
    }
}

impl TransformMatrix{
    pub fn view_matrix(&self) -> Matrix4<f32>{
        let mut v = self.0.try_inverse().unwrap();
        v[15] = 1.0;
        v
    }

    pub fn matrix(&self) -> &Matrix4<f32>{
        &self.0
    }
}

pub struct TransformSystem;

impl<'a> System<'a> for TransformSystem{
    type SystemData = (WriteStorage<'a, Position>,
                       WriteStorage<'a, Scale>,
                       WriteStorage<'a, Rotation>,
                       WriteStorage<'a, TransformMatrix>
    );

    fn run(&mut self, (mut positions, mut scales, mut rotations, mut transforms) : Self::SystemData){
        for (position, scale, rotation, transform) in (&mut positions, &mut scales, &mut rotations, &mut transforms).join(){
            if position.1 || scale.1 || rotation.1 {
                transform.0 = model_matrix_psr(position, scale, rotation);
                position.1 = false;
                scale.1 = false;
                rotation.1 = false;
            }
        }

        for (position, scale, transform, ()) in (&mut positions, &mut scales, &mut transforms, !&rotations).join(){
            if position.1 || scale.1 {
                transform.0 = model_matrix_ps(position, scale);
                position.1 = false;
                scale.1 = false;
            }
        }

        for (position, rotation, transform, ()) in (&mut positions, &mut rotations, &mut transforms, !&scales).join(){
            if position.1 || rotation.1 {
                transform.0 = model_matrix_pr(position, rotation);
                position.1 = false;
                rotation.1 = false;
            }
        }

        for (scale, rotation, transform, ()) in (&mut scales, &mut rotations, &mut transforms, !&positions).join(){
            if scale.1 || rotation.1 {
                transform.0 = model_matrix_sr(rotation, scale);
                scale.1 = false;
                rotation.1 = false;
            }
        }

        for (position, (), (), transform) in (&mut positions, !&scales, !&rotations, &mut transforms).join(){
            if position.1 {
                transform.0 = model_matrix_p(position);
                position.1 = false;
            }
        }

        for ((), scale, (), transform) in (!&positions, &mut scales, !&rotations, &mut transforms).join(){
            if scale.1 {
                transform.0 = model_matrix_s(scale);
                scale.1 = false;
            }
        }

        for ((), (), rotation, transform) in (!&positions, !&scales, &mut rotations, &mut transforms).join(){
            if rotation.1 {
                transform.0 = model_matrix_r(rotation);
                rotation.1 = false;
            }
        }
    }
}