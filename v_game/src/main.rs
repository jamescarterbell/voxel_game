use v_inputs::*;
use v_renderer::*;
use v_transform::*;
use v_windowing::*;
use v_agents::*;

use std::sync::{Arc, Mutex};

use nalgebra as na;
use na::Matrix4;

use specs::prelude::*;

use glium::buffer::BufferType::TransformFeedbackBuffer;
use glium::uniforms::EmptyUniforms;
use glium::*;

fn main() {
    let glium_state = GliumState::new();
    let mut world = World::new();

    let verts = vec![
        VoxelVertex{position: [1.0, 0.0, 0.0], texcoords: [1.0, 1.0], lighting: 0},
        VoxelVertex{position: [0.0, 0.0, 0.0], texcoords: [1.0, 1.0], lighting: 0},
        VoxelVertex{position: [1.0, 1.0, 0.0], texcoords: [1.0, 1.0], lighting: 0},
        VoxelVertex{position: [0.0, 1.0, 0.0], texcoords: [1.0, 1.0], lighting: 0}];

    let tris = vec![0, 1, 2, 1, 3, 2];

    let base_mesh = MeshBuffer::new(&glium_state.display, verts, tris);

    let mesh_renderer = MeshRenderer{mesh: Arc::new(Mutex::new(base_mesh))};

    world.register::<MeshRenderer<VoxelVertex>>();
    world.register::<Position>();
    world.register::<Scale>();
    world.register::<Rotation>();
    world.register::<TransformMatrix>();
    world.register::<Camera>();
    world.register::<Player>();

    world.insert(Inputs::default());

    world.create_entity().with(Camera{fov: 1.57, znear: 0.001, zfar: 4096.0}).with(Position::new(0.0, 0.0, 0.0)).with(Rotation::new()).with(Player{}).with(TransformMatrix::default()).build();
    world.create_entity().with(mesh_renderer).with(Position::new(1.0, 0.0, 20.0)).with(TransformMatrix::default()).build();

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(InputSystem::new(glium_state.input_queue().clone()))
        .with(PlayerMovement{}, "player_movement", &[])
        .with(TransformSystem, "transform_system", &["player_movement"])
        .build();

    let game = Game{world, dispatcher, program: VoxelProgram(&glium_state.display)};
    glium_state.run_event_loop(game);
}

struct Game<'a, 'b>{
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    program: Program,
}

impl GameState for Game<'_, '_>{
    fn game_loop(&mut self) {
        self.dispatcher.dispatch(&self.world);
    }

    fn render_loop(&mut self, display: &mut Display) -> Result<(), GliumError> {
        //Get the frame
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

        //Get the storage for the meshes that will be drawn
        let (voxel_meshes, transforms, cameras): (ReadStorage<MeshRenderer<VoxelVertex>>, ReadStorage<TransformMatrix>, ReadStorage<Camera>) = self.world.system_data();

        //Draw the meshes
        for(camera, cam_transform) in (&cameras, &transforms).join() {
            let vp = camera.perspective_matrix(&frame) * cam_transform.view_matrix();
            for (voxel_mesh, transform) in (&voxel_meshes, &transforms).join() {
                let mvp = (vp * transform.matrix());
                let mvp = [ [mvp.m11, mvp.m12, mvp.m13, mvp.m14],
                                    [mvp.m21, mvp.m22, mvp.m23, mvp.m24],
                                    [mvp.m31, mvp.m32, mvp.m33, mvp.m34],
                                    [mvp.m41, mvp.m42, mvp.m43, mvp.m44]];
                let mesh_buffer = voxel_mesh.mesh.lock().unwrap();
                frame.draw(&mesh_buffer.vertex_buffer, &mesh_buffer.index_buffer, &self.program, &uniform!(mvp: mvp), &params);
            }
        }

        frame.finish().map_err(|e|{GliumError::SwapBuffersError(e)})?;
        Ok(())
    }
}