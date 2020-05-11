use specs::prelude::*;
use glium::*;
use nalgebra as na;
use std::sync::{Arc, Mutex};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use nalgebra::Matrix4;
use nalgebra_glm::perspective;

#[derive(Copy, Clone)]
pub struct VoxelVertex{
    pub position: [f32; 3],
    pub texcoords: [f32; 2],
    pub lighting: u32
}

implement_vertex!(VoxelVertex, position, texcoords, lighting);

pub struct MeshBuffer<V>
    where V: Vertex{
    pub vertex_buffer: VertexBuffer<V>,
    pub index_buffer: IndexBuffer<u32>
}

impl<V> MeshBuffer<V>
    where V: Vertex{
    pub fn new<F>(display: &F, verts: Vec<V>, indicies: Vec<u32>) -> Self
    where F: Facade{
        Self{
            vertex_buffer: VertexBuffer::new(display, &verts).unwrap(),
            index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &indicies).unwrap(),
        }
    }
}

pub struct MeshRenderer<V>
    where V: Vertex + Send + Sync{
    pub mesh: Arc<Mutex<MeshBuffer<V>>>,
}

unsafe impl<V> Sync for MeshRenderer<V>
    where V: Vertex + Send + Sync{}
unsafe impl<V> Send for MeshRenderer<V>
    where V: Vertex + Send + Sync{}

impl<V> Component for MeshRenderer<V>
    where V: Vertex + Send + Sync + 'static{
    type Storage = VecStorage<Self>;
}

pub fn VoxelProgram(display: &Display) -> Program{
    glium::Program::from_source(display, include_str!("../../src/shaders/base_vertex.glsl"), include_str!("../../src/shaders/base_frag.glsl"), None).unwrap()
}


pub struct Camera{
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera{

    pub fn perspective_matrix<F: Surface>(&self, target: &F) -> Matrix4<f32> {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        perspective(aspect_ratio, self.fov, self.znear, self.zfar)
    }
}

impl Component for Camera{
    type Storage = HashMapStorage<Self>;
}