use v_transform::*;
use v_renderer::*;
use nalgebra as na;
use na::Vector3;
use specs::prelude::*;
use v_rle::*;

const CHUNK_SIZE: usize = 32;
const CHUNK_SIZE_2: usize = CHUNK_SIZE * CHUNK_SIZE;
const CHUNK_SIZE_3: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType{
    Dynamic = 0,
    Air,
    Dirt,
    Rock,
}

pub struct Chunk{
    blocks: RLE<BlockType>
}

impl Chunk{
    pub fn new() -> Self{
        Chunk {
            blocks : RLE::from(vec![BlockType::Air; CHUNK_SIZE_3 as usize].iter()),
        }
    }

    pub fn vec_to_index(position: Vector3<usize>) -> Result<usize, VoxelError>{
        if position[0] < CHUNK_SIZE && position[1] < CHUNK_SIZE && position[2] < CHUNK_SIZE {
            return Ok(position[0] + position[1] * CHUNK_SIZE + position[2] * CHUNK_SIZE_2)
        }
        Err(VoxelError::ChunkCoordOutOfRange)
    }

    pub fn get_block(&self, position: Vector3<usize>) -> BlockType{
        let index = Self::vec_to_index(position).unwrap();
        self.blocks.access(index).unwrap()
    }

    pub fn set_block(&mut self, position: Vector3<usize>, block: &BlockType){
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoxelError{
    ChunkCoordOutOfRange,
}