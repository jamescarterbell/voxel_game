use v_transform::*;
use v_renderer::*;
use nalgebra as na;
use na::Vector3;
use specs::prelude::*;
use specs::ParJoin;
use v_rle::*;
use std::iter::repeat;
use dashmap::*;
use dashmap::mapref::one::Ref;
use v_renderer::*;
use std::sync::Arc;
use nalgebra::{Matrix, U1};

const BLOCK_SIZE: f32 = 0.5;
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
        let rle : RLE<BlockType> = RLE::from(repeat(BlockType::Air).cycle().take(CHUNK_SIZE_3));
        Chunk {
            blocks : rle,
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
        self.blocks.get(index).unwrap()
    }

    pub fn set_block(&mut self, position: Vector3<usize>, block: &BlockType){
        let index = Self::vec_to_index(position).unwrap();
        self.blocks.set(index, block);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoxelError{
    ChunkCoordOutOfRange,
}

pub struct ChunkStorage{
    map: DashMap<Vector3<i32>, Chunk>,
}

impl ChunkStorage{
    pub fn new() -> Self{
        Self{
            map: DashMap::new(),
        }
    }

    pub fn get_chunk(&self, place: Vector3<i32>) -> Option<Ref<Vector3<i32>, Chunk>>{
        self.map.get(&place)
    }

    pub fn set_chunk(&mut self, place: Vector3<i32>, chunk: Chunk){
        self.map.insert(place, chunk);
    }

    pub fn set_block(&mut self, block: &BlockType, place: &Vector3<i32>){
        let chunk_coord = Vector3::new((place[0] as f32 / CHUNK_SIZE as f32).floor() as i32,
                                 (place[1] as f32 / CHUNK_SIZE as f32).floor() as i32,
                                 (place[2] as f32 / CHUNK_SIZE as f32).floor() as i32);
        let block_coord = place - chunk_coord * CHUNK_SIZE as i32;
        let mut chunk = match self.map.get_mut(&chunk_coord){
            Some(chunk) => chunk,
            None => {
                self.map.insert(chunk_coord,Chunk::new());
                self.map.get_mut(&chunk_coord).unwrap()
            }
        };
        let block_coord = Vector3::new(block_coord[0] as usize, block_coord[1] as usize, block_coord[2] as usize);
        chunk.set_block(block_coord, block);
    }

    pub fn get_block(&self, place: &Vector3<i32>) -> BlockType{
        let chunk_coord = Vector3::new((place[0] as f32 / CHUNK_SIZE as f32).floor() as i32,
                                       (place[1] as f32 / CHUNK_SIZE as f32).floor() as i32,
                                       (place[2] as f32 / CHUNK_SIZE as f32).floor() as i32);
        let block_coord = place - chunk_coord * CHUNK_SIZE as i32;
        let mut chunk = match self.map.get_mut(&chunk_coord){
            Some(chunk) => chunk,
            None => {
                self.map.insert(chunk_coord,Chunk::new());
                self.map.get_mut(&chunk_coord).unwrap()
            }
        };
        let block_coord = Vector3::new(block_coord[0] as usize, block_coord[1] as usize, block_coord[2] as usize);
        chunk.get_block(block_coord)
    }
}

impl Default for ChunkStorage{
    fn default() -> Self{
        Self::new()
    }
}

pub struct ChunkMarker{
    pub coords: Vector3<i32>,
    pub changed: bool,
}

impl Component for ChunkMarker{
    type Storage = VecStorage<Self>;
}

impl Default for ChunkMarker{
    fn default() -> Self{
        Self{
            coords: Vector3::new(0,0,0),
            changed: false,
        }
    }
}

pub struct ChunkRenderer{
    lods: Vec<(VertexBuffer<VoxelVertex>, IndexBuffer<u32>)>
}

impl Component for ChunkRenderer{
    type Storage = VecStorage<Self>;
}

unsafe impl Send for ChunkRenderer{}
unsafe impl Sync for ChunkRenderer{}

impl Default for ChunkRenderer{
    fn default() -> Self{
        Self{
            lods: vec![],
        }
    }
}

pub struct ChunkMesherSystem{}

impl<'a> System<'a> for ChunkMesherSystem{
    type SystemData = (
        Read<'a, ChunkStorage>,
        WriteStorage<'a, ChunkMarker>,
        WriteStorage<'a, ChunkRenderer>
    );

    fn run(&mut self, (chunks, mut markers, mut renderers): Self::SystemData){
        (&mut markers, &mut renderers).par_join().for_each(
            |(marker, renderer)|{
                marker.changed = false;

                let chunk = chunks.get_chunk(marker.coords);
                let chunk = chunk.unwrap().value();

                let mut verts = vec![];
                let mut tris = vec![];

                for block_x in 0..CHUNK_SIZE{
                    for block_y in 0..CHUNK_SIZE{
                        for block_z in 0..CHUNK_SIZE{
                            //TODO: CHUNK MESHING
                        }
                    }
                }

        });
    }
}