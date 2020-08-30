use bevy::{
    asset::Assets,
    ecs::Resources,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph, RenderResourcesNode},
        shader::Shader,
    },
    transform::prelude::Transform,
};

use super::material::VoxelMaterial;

pub mod pipeline;

pub mod node {
    pub const TRANSFORM: &str = "transform";
    pub const VOXEL_MATERIAL: &str = "voxel_material";
}

pub mod uniform {}

pub(crate) fn add_voxel_graph(graph: &mut RenderGraph, resources: &Resources) {
    graph.add_system_node(node::TRANSFORM, RenderResourcesNode::<Transform>::new(true));
    graph
        .add_node_edge(node::TRANSFORM, base::node::MAIN_PASS)
        .unwrap();

    graph.add_system_node(
        node::VOXEL_MATERIAL,
        AssetRenderResourcesNode::<VoxelMaterial>::new(true),
    );
    graph
        .add_node_edge(node::VOXEL_MATERIAL, base::node::MAIN_PASS)
        .unwrap();

    let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();
    let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
    pipelines.set(
        pipeline::PIPELINE_HANDLE,
        pipeline::build_pipeline(&mut shaders),
    );
}
