use specs::prelude::*;
use glium::*;
use nalgebra as na;
use std::sync::{Arc, Mutex};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use nalgebra::{Matrix4, Vector3, Vector2};
use nalgebra_glm::perspective;
use v_windowing::WindowDisplay;
use v_transform::*;
pub use glium::*;
use glium::texture::RawImage2d;
use glium::texture::Texture2dArray;
use glium::uniforms::{MinifySamplerFilter, MagnifySamplerFilter};


#[derive(Copy, Clone)]
pub struct VoxelVertex{
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub tex_index: u32,
    pub lighting: u32
}

impl VoxelVertex{
    pub fn new(position: Vector3::<f32>, texcoord: Vector2::<f32>, tex: u32, lighting: u32) -> Self{
        VoxelVertex{
            position: [position[0], position[1], position[2]],
            tex_coord: [texcoord[0], texcoord[1]],
            tex_index: tex,
            lighting
        }
    }
}

implement_vertex!(VoxelVertex, position, tex_coord, tex_index, lighting);

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

pub struct Camera{
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera{

    pub fn perspective_matrix<F: Surface>(&self, target: &F) -> Matrix4<f32> {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        perspective(1.0/aspect_ratio, self.fov, self.znear, self.zfar)
    }
}

impl Component for Camera{
    type Storage = HashMapStorage<Self>;
}

pub struct VoxelRenderingSystem{
    program: Program,
    textures: Texture2dArray
}

impl VoxelRenderingSystem{
    pub fn new(display: &Display) -> Self{
        VoxelRenderingSystem{
            program:  glium::Program::from_source(display, include_str!("../../src/shaders/base_vertex.glsl"), include_str!("../../src/shaders/base_frag.glsl"), None).unwrap(),
            textures: Texture2dArray::new(display, vec!["dirt.png", "grass.png", "rock.png"].iter().map(|x| Self::get_raw_image(*x)).collect()).unwrap(),
        }
    }

    fn get_raw_image(file_name: &str) -> RawImage2d<u8>{
        let tex = image::open(format!("{}{}", "./assets/", file_name)).unwrap().to_rgba();
        let tex_dimensions = tex.dimensions();
        glium::texture::RawImage2d::from_raw_rgba_reversed(&tex.into_raw(), tex_dimensions)
    }
}

impl<'a> System<'a> for VoxelRenderingSystem{
    type SystemData = (
    Write<'a, WindowDisplay>,
    ReadStorage<'a, MeshRenderer<VoxelVertex>>,
    ReadStorage<'a, TransformMatrix>,
    ReadStorage<'a, Camera>);

    fn run(&mut self, (mut window, voxel_meshes, transforms, cameras): Self::SystemData){
        let display = window.as_ref().unwrap().lock().unwrap();
        let mut frame = display.draw();


        //Clear the frame color and depth
        frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        //Create the draw params
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };

        //Draw the meshes
        for(camera, cam_transform) in (&cameras, &transforms).join() {
            let vp = camera.perspective_matrix(&frame) * cam_transform.view_matrix();
            let sampler = self.textures.sampled().minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest);
            for (voxel_mesh, transform) in (&voxel_meshes, &transforms).join() {
                let mvp = (vp * transform.matrix());
                let mesh_buffer = voxel_mesh.mesh.lock().unwrap();
                frame.draw(&mesh_buffer.vertex_buffer, &mesh_buffer.index_buffer, &self.program, &uniform!(mvp: mvp.as_ref().clone(), tex: sampler), &params);
            }
        }

        frame.finish();
    }
}