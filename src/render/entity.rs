use bevy::{
    prelude::*,
    asset::Handle,
    ecs::Bundle,
    render::{
        draw::Draw,
        mesh::*,
        pipeline::{PipelineSpecialization, PrimitiveTopology, RenderPipeline, RenderPipelines},
        render_graph::base::MainPass,
    },
    transform::prelude::{Transform},
};

use crate::{
    collections::lod_tree::Voxel,
    render::{material::VoxelMaterial, render_graph::pipeline},
    world::{Chunk, Map},
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transparent {
    No,
    Yes,
}

impl From<bool> for Transparent {
    fn from(p: bool) -> Self {
        if p {
            Self::Yes
        } else {
            Self::No
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshPart {
    pub positions: Vec<[f32; 3]>,
    pub shades: Vec<f32>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub transparent: Transparent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
}

pub trait VoxelExt: Voxel {
    fn mesh(
        &self,
        coords: (i32, i32, i32),
        map: &Map<Self>,
        chunk: &Chunk<Self>,
        width: usize,
    ) -> MeshPart;

    fn set_shade(&mut self, _face: Face, _light: f32) {}

    fn shade(&mut self, _face: Face) -> Option<f32> {
        None
    }
}

#[derive(Bundle)]
pub struct ChunkRenderComponents {
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ChunkRenderComponents {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(pipeline::PIPELINE_HANDLE)]),
            mesh: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub fn generate_chunk_mesh<T: VoxelExt>(map: &Map<T>, chunk: &Chunk<T>) -> (Option<Mesh>, Option<Mesh>) {
    let mut positions = Vec::new();
    let mut shades = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut n = 0;
    
    let mut t_positions = Vec::new();
    let mut t_shades = Vec::new();
    let mut t_colors = Vec::new();
    let mut t_indices = Vec::new();
    let mut t_n = 0;

    for elem in chunk.iter() {
        let mut mesh = elem
            .value
            .mesh((elem.x, elem.y, elem.z), map, chunk, elem.width);

        if mesh.transparent == Transparent::Yes {
            let count = mesh.positions.len();
            mesh.indices.iter_mut().for_each(|i| *i += t_n as u32);
            t_n += count;

            t_positions.extend(mesh.positions);
            t_shades.extend(mesh.shades);
            t_colors.extend(mesh.colors);
            t_indices.extend(mesh.indices);
        } else {
            let count = mesh.positions.len();
            mesh.indices.iter_mut().for_each(|i| *i += n as u32);
            n += count;

            positions.extend(mesh.positions);
            shades.extend(mesh.shades);
            colors.extend(mesh.colors);
            indices.extend(mesh.indices);
        }
    }

    let mesh = if positions.is_empty() {
        None
    } else {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute("Voxel_Position", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Voxel_Shade", VertexAttributeValues::Float(shades));
        mesh.set_attribute("Voxel_Color", VertexAttributeValues::Float4(colors));
        mesh.set_indices(Some(Indices::U32(indices)));

        Some(mesh)
    };
    
    let t_mesh = if t_positions.is_empty() {
        None
    } else {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute("Voxel_Position", VertexAttributeValues::Float3(t_positions));
        mesh.set_attribute("Voxel_Shade", VertexAttributeValues::Float(t_shades));
        mesh.set_attribute("Voxel_Color", VertexAttributeValues::Float4(t_colors));
        mesh.set_indices(Some(Indices::U32(t_indices)));

        Some(mesh)
    };

    (mesh, t_mesh)
}
