use bevy::{
    asset::Handle,
    ecs::Bundle,
    render::{
        draw::Draw,
        mesh::Mesh,
        pipeline::{DynamicBinding, PipelineSpecialization, RenderPipeline, RenderPipelines},
        render_graph::base::MainPass,
    },
    transform::prelude::{Rotation, Scale, Transform, Translation},
};

use crate::{
    collections::lod_tree::Voxel,
    render::{material::VoxelMaterial, render_graph::pipeline},
    world::{Chunk, Map},
};

#[derive(Debug, Clone)]
pub struct MeshPart {
    pub positions: Vec<[f32; 3]>,
    pub shades: Vec<f32>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
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
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
}

impl Default for ChunkRenderComponents {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
                pipeline::PIPELINE_HANDLE,
                PipelineSpecialization {
                    dynamic_bindings: vec![
                        // Transform
                        DynamicBinding {
                            bind_group: 2,
                            binding: 0,
                        },
                        // Voxel_material
                        DynamicBinding {
                            bind_group: 1,
                            binding: 0,
                        },
                    ],
                    ..Default::default()
                },
            )]),
            mesh: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            translation: Default::default(),
            rotation: Default::default(),
            scale: Default::default(),
        }
    }
}

pub fn generate_chunk_mesh<T: VoxelExt>(map: &Map<T>, chunk: &Chunk<T>) -> Option<Mesh> {
    let mut positions = Vec::new();
    let mut shades = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut n = 0;

    for elem in chunk.iter() {
        let mut mesh = elem.value.mesh(
            (elem.x, elem.y, elem.z),
            map,
            chunk,
            elem.width,
        );

        let count = mesh.positions.len();
        mesh.indices.iter_mut().for_each(|i| *i += n as u32);
        n += count;

        positions.extend(mesh.positions);
        shades.extend(mesh.shades);
        colors.extend(mesh.colors);
        indices.extend(mesh.indices);
    }

    if positions.is_empty() {
        return None;
    }

    Some(Mesh {
        primitive_topology: bevy::render::pipeline::PrimitiveTopology::TriangleList,
        attributes: vec![
            bevy::render::mesh::VertexAttribute {
                name: From::from("Voxel_Position"),
                values: bevy::render::mesh::VertexAttributeValues::Float3(positions),
            },
            bevy::render::mesh::VertexAttribute {
                name: From::from("Voxel_Shade"),
                values: bevy::render::mesh::VertexAttributeValues::Float(shades),
            },
            bevy::render::mesh::VertexAttribute {
                name: From::from("Voxel_Color"),
                values: bevy::render::mesh::VertexAttributeValues::Float4(colors),
            },
        ],
        indices: Some(indices),
    })
}
