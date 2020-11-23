use bevy::{
    asset::{Assets, Handle},
    render::{
        pipeline::PipelineDescriptor,
        shader::{Shader, ShaderStage, ShaderStages},
    },
};
use bevy::type_registry::TypeUuid;

pub const PIPELINE_HANDLE: Handle<PipelineDescriptor> =
    Handle::weak_from_u64(PipelineDescriptor::TYPE_UUID, 12585943984739023957);

pub(crate) fn build_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("voxel_vs.glsl"),
        )),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("voxel_fs.glsl"),
        ))),
    })
}
