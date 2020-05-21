use v_inputs::*;
use v_renderer::*;
use v_transform::*;
use v_windowing::*;
use v_agents::*;
use v_voxels::*;

use std::sync::{Arc, Mutex};

use nalgebra as na;
use na::{Matrix4, Vector3};

use specs::prelude::*;

use glium::buffer::BufferType::TransformFeedbackBuffer;
use glium::uniforms::EmptyUniforms;
use glium::*;
use glutin::CreationError::Window;
use std::ops::Deref;

fn main() {
    let glium_state = GliumState::new();
    let mut world = World::new();

    let verts = vec![
        VoxelVertex{position: [1.0, 0.0, 0.0], texcoords: [1.0, 1.0], lighting: 0},
        VoxelVertex{position: [0.0, 0.0, 0.0], texcoords: [1.0, 1.0], lighting: 0},
        VoxelVertex{position: [1.0, 1.0, 0.0], texcoords: [1.0, 1.0], lighting: 0},
        VoxelVertex{position: [0.0, 1.0, 0.0], texcoords: [1.0, 1.0], lighting: 0}];

    let tris = vec![0, 1, 2, 1, 3, 2];

    let base_mesh = MeshBuffer::new(glium_state.display.as_ref().unwrap().lock().unwrap().deref(), verts, tris);

    let mesh_renderer = MeshRenderer{mesh: Arc::new(Mutex::new(base_mesh))};

    world.register::<MeshRenderer<VoxelVertex>>();
    world.register::<Position>();
    world.register::<Scale>();
    world.register::<Rotation>();
    world.register::<TransformMatrix>();
    world.register::<Camera>();
    world.register::<Player>();
    world.register::<ChunkMarker>();

    world.insert(Inputs::default());
    world.insert(glium_state.display.clone());
    world.insert(CursorState::default());
    world.insert(ChunkStorage::new());

    world.create_entity().with(Camera{fov: 1.57, znear: 0.001, zfar: 4096.0}).with(Position::new(0.0, 0.0, 0.0)).with(Rotation::new()).with(Player{}).with(TransformMatrix::default()).build();
    world.create_entity().with(mesh_renderer).with(Position::new(0.0, 0.0, 10.0)).with(TransformMatrix::default()).build();


    let (window_inputs, hardware_inputs) = glium_state.input_queues();

    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(InputSystem::new(window_inputs.clone(), hardware_inputs.clone()))
        .with_thread_local(CursorLockSystem{})
        .with(PlayerMovement{}, "player_movement", &[])
        .with(NewChunkPlacementSystem{}, "chunk_placer", &[])
        .with_thread_local(ChunkMesherSystem{})
        .with(TransformSystem, "transform_system", &["player_movement"])
        .with_thread_local(VoxelRenderingSystem::new(glium_state.display.as_ref().unwrap().lock().unwrap().deref()))
        .build();

    let game = Game{world, dispatcher};
    glium_state.run_event_loop(game);
}

struct Game<'a, 'b>{
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl GameState for Game<'_, '_>{
    fn game_loop(&mut self) {
        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
    }
}