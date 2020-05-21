use v_transform::*;
use v_voxels::*;
use v_inputs::{Inputs, KeyCode, KeyState, ButtonCode};
use nalgebra as na;
use specs::prelude::*;
use specs::storage::BTreeStorage;
use nalgebra::Vector3;

#[derive(Default)]
pub struct Player{}

impl Component for Player{
    type Storage = NullStorage<Self>;
}

pub struct PlayerMovement{}

impl<'a> System<'a> for PlayerMovement{
    type SystemData = (
        Read<'a, Inputs>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Rotation>,
        Write<'a, ChunkStorage>
    );

    fn run(&mut self, (inputs, players, mut positions, mut rotations, mut chunks) : Self::SystemData){
        for (player, position, rotation) in (&players, &mut positions, &mut rotations).join(){
            let mut delta = (*inputs.get_mouse_delta()).clone();
            delta /= 1000.0;
            delta[0] = if delta[0].abs() > 0.0001 {delta[0]} else {0.0};
            delta[1] = if delta[1].abs() > 0.0001 {delta[1]} else {0.0};
            rotation.apply_axis_angle_rotation(-delta.x, Vector3::new(0.0, 1.0, 0.0));
            rotation.apply_axis_angle_rotation(-delta.y, rotation.right());

            let y: f32 =
                if *inputs.get_key(&KeyCode::W) == KeyState::Down {-0.025} else {0.0} +
                if *inputs.get_key(&KeyCode::S) == KeyState::Down {0.025} else {0.0};

            let x: f32 =
                if *inputs.get_key(&KeyCode::D) == KeyState::Down {0.025} else {0.0} +
                if *inputs.get_key(&KeyCode::A) == KeyState::Down {-0.025} else {0.0};

            let place_coord = *position.value() + -rotation.forward() * 3.0;
            if *inputs.get_button(&ButtonCode::MB0) == KeyState::Pressed{
                chunks.set_block(&BlockType::Dirt, &place_coord)
            }

            if *inputs.get_button(&ButtonCode::MB2) == KeyState::Pressed{
                chunks.set_block(&BlockType::Air, &place_coord)
            }

            *position.value() += y * rotation.forward() + x * rotation.right();
        }
    }
}