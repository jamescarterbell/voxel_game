use v_transform::*;
use v_rle::*;
use v_renderer::*;
use v_windowing::*;

use v_renderer::index::PrimitiveType;

use nalgebra as na;
use na::{Vector3, Vector2};
use specs::prelude::*;
use specs::ParJoin;
use dashmap::*;
use dashmap::mapref::one::Ref;
use std::sync::{Arc, Mutex};
use nalgebra::{Matrix, U1};
use std::collections::HashSet;
use std::ops::{Deref};
use std::sync::mpsc::{channel, TryRecvError, Sender};
use rayon::prelude::*;
use std::thread;
use specs::world::EntitiesRes;

use rand::*;

const BLOCK_SIZE: f32 = 0.5;
const CHUNK_SIZE: usize = 32;
const CHUNK_SIZE_2: usize = CHUNK_SIZE * CHUNK_SIZE;
const CHUNK_SIZE_3: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType{
    Dynamic = 0,
    Air,
    Dirt,
    Grass,
    Rock,
}

impl BlockType{
    pub fn is_transparent(&self) -> bool{
        match self{
            BlockType::Air => true,
            BlockType::Dynamic => true,
            _ => false,
        }
    }

    pub fn texture_index(&self, direction: Direction) -> Option<u32>{
        match self{
            BlockType::Dirt => Some(0),
            BlockType::Grass => match direction{
                Direction::Top => Some(1),
                _ => Some(0),
            },
            BlockType::Rock => Some(2),
            _ => None,
        }
    }
}

pub enum Direction{
    Top,
    Bottom,
    Right,
    Left,
    Front,
    Back
}

pub struct Chunk{
    blocks: RLE<BlockType>
}

impl Chunk{
    pub fn new() -> Self{
        let rle : RLE<BlockType> = RLE::from(std::iter::repeat(BlockType::Air).cycle().take(CHUNK_SIZE_3));
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
    needed_chunks: Arc<Mutex<Vec<Vector3<i32>>>>,
    changed_chunks: HashSet<Vector3<i32>>
}

impl ChunkStorage{
    pub fn new() -> Self{
        Self{
            map: DashMap::new(),
            needed_chunks: Arc::new(Mutex::new(vec![])),
            changed_chunks: HashSet::new(),
        }
    }

    pub fn get_chunk(&self, place: Vector3<f32>) -> Option<Ref<Vector3<i32>, Chunk>>{
        let chunk_coord = place.map(|x| (x as f32 / CHUNK_SIZE as f32 / BLOCK_SIZE).floor() as i32);
        self.map.get(&chunk_coord)
    }

    pub fn set_chunk(&mut self, place: Vector3<f32>, chunk: Chunk){
        let chunk_coord = place.map(|x| (x as f32 / CHUNK_SIZE as f32 / BLOCK_SIZE).floor() as i32);
        self.map.insert(chunk_coord, chunk);
    }

    fn world_point_to_chunk_block(place: &Vector3<f32>) -> (Vector3<i32>, Vector3<usize>){
        let chunk_coord = place.map(|x| (x as f32 / CHUNK_SIZE as f32 / BLOCK_SIZE).floor() as i32);
        let chunk_coord_f32 = chunk_coord.map(|x| x as f32);
        let block_coord = ((place   - chunk_coord_f32 * CHUNK_SIZE as f32 * BLOCK_SIZE) / BLOCK_SIZE).map(|x| x as usize);
        (chunk_coord, block_coord)
    }

    pub fn set_block(&mut self, block: &BlockType, place: &Vector3<f32>){
        let (chunk_coord, block_coord) = Self::world_point_to_chunk_block(place);
        let mut chunk = match self.map.get_mut(&chunk_coord){
            Some(chunk) => chunk,
            None => {
                self.map.insert(chunk_coord,Chunk::new());
                self.needed_chunks.lock().unwrap().push(chunk_coord);
                self.map.get_mut(&chunk_coord).unwrap()
            }
        };

        if block_coord[0] == 0{
            self.changed_chunks.insert(chunk_coord + Vector3::new(-1, 0, 0));
        } else if block_coord[0] == CHUNK_SIZE{
            self.changed_chunks.insert(chunk_coord + Vector3::new(1, 0, 0));
        }

        if block_coord[1] == 0{
            self.changed_chunks.insert(chunk_coord + Vector3::new(0, -1, 0));
        } else if block_coord[1] == CHUNK_SIZE{
            self.changed_chunks.insert(chunk_coord + Vector3::new(0, 1, 0));
        }

        if block_coord[2] == 0{
            self.changed_chunks.insert(chunk_coord + Vector3::new(0, 0, -1));
        } else if block_coord[2] == CHUNK_SIZE{
            self.changed_chunks.insert(chunk_coord + Vector3::new(0, 0, 1));
        }

        self.changed_chunks.insert(chunk_coord);
        chunk.set_block(block_coord, block);
    }

    pub fn get_block(&self, place: &Vector3<f32>) -> BlockType{
        let (chunk_coord, block_coord) = Self::world_point_to_chunk_block(place);
        let mut chunk = match self.map.get_mut(&chunk_coord){
            Some(chunk) => chunk,
            None => {
                self.map.insert(chunk_coord,Chunk::new());
                self.needed_chunks.lock().unwrap().push(chunk_coord);
                self.map.get_mut(&chunk_coord).unwrap()
            }
        };
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
    pub renderable: bool,
    pub changed: bool,
}

impl Component for ChunkMarker{
    type Storage = VecStorage<Self>;
}

impl Default for ChunkMarker{
    fn default() -> Self{
        Self{
            coords: Vector3::new(0,0,0),
            renderable: false,
            changed: false,
        }
    }
}


pub struct ChunkMesherSystem{}

impl ChunkMesherSystem{
    pub fn mesh_chunk(chunks: &ChunkStorage, marker: &mut ChunkMarker) -> (Vec<VoxelVertex>, Vec<u32>){

        let dimension = (0..CHUNK_SIZE).into_par_iter();

        let (sender, receiver):(Sender<Vec<VoxelVertex>>, std::sync::mpsc::Receiver<Vec<VoxelVertex>>) = channel();
        let (mesh_finished_sender, mesh_finished_receiver) = channel();

        let tri_thread = thread::spawn(move ||{
            let mut final_verts = vec![];
            let mut tris = vec![];
            while mesh_finished_receiver.try_recv() == Err(TryRecvError::Empty) {
                for mut verts in receiver.iter() {
                    let tri_start = final_verts.len() as u32;
                    let tri_count = verts.len() as u32 / 4;

                    final_verts.append(&mut verts);


                    for tri in 0..tri_count {
                        tris.push(tri_start + 4 * tri + 0);
                        tris.push(tri_start + 4 * tri + 1);
                        tris.push(tri_start + 4 * tri + 2);
                        tris.push(tri_start + 4 * tri + 2);
                        tris.push(tri_start + 4 * tri + 3);
                        tris.push(tri_start + 4 * tri + 0);
                    }
                }
            }
            (final_verts, tris)
        });

        dimension
            .for_each_with(sender,
                |sender, (block_x)| {

                    let mut verts = vec![];

                    for block_y in 0..CHUNK_SIZE{
                        for block_z in 0..CHUNK_SIZE{
                            let block_coord = Vector3::new(block_x as f32, block_y as f32, block_z as f32) * BLOCK_SIZE
                                            + marker.coords.map(|x| x as f32) * CHUNK_SIZE as f32 * BLOCK_SIZE;
                            let block = chunks.get_block(&block_coord);
                            if block == BlockType::Air { continue }

                            let top_block = chunks.get_block(&(block_coord + Vector3::new(0.0, 1.0, 0.0) * BLOCK_SIZE));
                            if top_block.is_transparent() {
                                let tex = block.texture_index(Direction::Top).unwrap();
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 1.0
                                    ),
                                    tex,
                                    0
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 0.0
                                    ),
                                    tex,
                                    0
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 0.0
                                    ),
                                    tex,
                                    0
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 1.0
                                    ),
                                    tex,
                                    0
                                ));
                            }


                            let bottom_block = chunks.get_block(&(block_coord + Vector3::new(0.0, -1.0, 0.0) * BLOCK_SIZE));
                            if bottom_block.is_transparent() {
                                let tex = block.texture_index(Direction::Bottom).unwrap();
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 1.0
                                    ),
                                    tex,
                                    1
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 0.0
                                    ),
                                    tex,
                                    1
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 0.0
                                    ),
                                    tex,
                                    1
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 1.0
                                    ),
                                    tex,
                                    1
                                ));
                            }


                            let right_block = chunks.get_block(&(block_coord + Vector3::new(1.0, 0.0, 0.0) * BLOCK_SIZE));
                            if right_block.is_transparent() {
                                let tex = block.texture_index(Direction::Right).unwrap();
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 1.0
                                    ),
                                    tex,
                                    2
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 0.0
                                    ),
                                    tex,
                                    2
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 0.0
                                    ),
                                    tex,
                                    2
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 1.0
                                    ),
                                    tex,
                                    2
                                ));
                            }

                            let left_block = chunks.get_block(&(block_coord + Vector3::new(-1.0, 0.0, 0.0) * BLOCK_SIZE));
                            if left_block.is_transparent() {
                                let tex = block.texture_index(Direction::Left).unwrap();
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 1.0
                                    ),
                                    tex,
                                    3
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 0.0
                                    ),
                                    tex,
                                    3
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 0.0
                                    ),
                                    tex,
                                    3
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 1.0
                                    ),
                                    tex,
                                    3
                                ));
                            }


                            let front_block = chunks.get_block(&(block_coord + Vector3::new(0.0, 0.0, 1.0) * BLOCK_SIZE));
                            if front_block.is_transparent() {
                                let tex = block.texture_index(Direction::Front).unwrap();
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 1.0
                                    ),
                                    tex,
                                    4
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 0.0
                                    ),
                                    tex,
                                    4
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 0.0
                                    ),
                                    tex,
                                    4
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 1.0
                                    ),
                                    tex,
                                    4
                                ));
                            }

                            let back_block = chunks.get_block(&(block_coord + Vector3::new(0.0, 0.0, -1.0) * BLOCK_SIZE));
                            if back_block.is_transparent() {
                                let tex = block.texture_index(Direction::Back).unwrap();
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 1.0
                                    ),
                                    tex,
                                    5
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        1.0, 0.0
                                    ),
                                    tex,
                                    5
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 0.0
                                    ),
                                    tex,
                                    5
                                ));
                                verts.push(VoxelVertex::new(
                                    Vector3::new(
                                        block_x as f32 * BLOCK_SIZE + BLOCK_SIZE / 2.0,
                                        block_y as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0,
                                        block_z as f32 * BLOCK_SIZE - BLOCK_SIZE / 2.0),
                                    Vector2::new(
                                        0.0, 1.0
                                    ),
                                    tex,
                                    5
                                ));
                            }
                        }
                    }

                    if !verts.is_empty() {
                        sender.send(verts);
                    }
                });

        mesh_finished_sender.send(true);
        tri_thread.join().unwrap()
    }
}

impl<'a> System<'a> for ChunkMesherSystem{
    type SystemData = (
        Entities<'a>,
        Read<'a, ChunkStorage>,
        Read<'a, WindowDisplay>,
        WriteStorage<'a, ChunkMarker>,
        WriteStorage<'a, MeshRenderer<VoxelVertex>>
    );

    fn run(&mut self, (entities, chunks, display, mut markers, mut renderers): Self::SystemData){
        let (send, recieve) = channel();
        (&mut markers, &entities).par_join().for_each_with(send, |sender, (marker, entity)|{
                if !marker.changed || !marker.renderable{return;}
                marker.changed = false;

                let (verts, tris) = Self::mesh_chunk(chunks.deref(), marker);

                sender.send((entity, verts, tris)).unwrap();
        });

        let display = display.as_ref().unwrap().lock().unwrap();
        for (entity, verts, tris) in recieve.iter() {
            let buffer = MeshBuffer::new(display.deref(), verts, tris);
            renderers.insert(entity, MeshRenderer { mesh: Arc::new(Mutex::new(buffer)) });
        }
    }
}

pub struct NewChunkPlacementSystem{}

impl<'a> System<'a> for NewChunkPlacementSystem{
    type SystemData = (
        Write<'a, ChunkStorage>,
        Read<'a, EntitiesRes>,
        WriteStorage<'a, ChunkMarker>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, TransformMatrix>,
        Read<'a, LazyUpdate>
    );

    fn run(&mut self, (mut chunks, entities, mut chunk_markers, mut positions, mut transforms, lazy): Self::SystemData){
        if let Some(new_chunk_coord)  = chunks.needed_chunks.lock().unwrap().pop(){
            let chunk_pos = new_chunk_coord.map(|x| x as f32 * BLOCK_SIZE * CHUNK_SIZE as f32);
            lazy.create_entity(&entities)
                .with(Position::new(chunk_pos[0], chunk_pos[1], chunk_pos[2]))
                .with(TransformMatrix::default())
                .with(ChunkMarker{coords:new_chunk_coord, changed:true, renderable:true})
                .build();
        }

        if !chunks.changed_chunks.is_empty() {
            for (chunk_marker) in (&mut chunk_markers).join() {
                if chunks.changed_chunks.contains(&chunk_marker.coords) {
                    chunk_marker.changed = true;
                }
                chunks.changed_chunks.remove(&chunk_marker.coords);
            }
        }
    }
}
