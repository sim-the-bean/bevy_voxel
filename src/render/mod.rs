use bevy::{
    prelude::*,
    render::{render_graph::RenderGraph, shader},
};

use self::material::VoxelMaterial;

pub mod entity;
pub mod light;
pub mod lod;
pub mod material;
pub mod render_graph;

pub mod prelude {
    pub use super::{entity::ChunkRenderComponents, material::VoxelMaterial, VoxelRenderPlugin};
}

#[derive(Debug, Default)]
pub struct VoxelRenderPlugin;

impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<VoxelMaterial>().add_system_to_stage(
            stage::POST_UPDATE,
            shader::asset_shader_defs_system::<VoxelMaterial>.system(),
        );
        let resources = app.resources();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        render_graph::add_voxel_graph(&mut render_graph, resources);
    }
}
