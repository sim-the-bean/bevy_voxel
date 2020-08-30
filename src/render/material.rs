use bevy::{
    prelude::*,
    render::{renderer::RenderResources, shader::ShaderDefs},
};

#[derive(RenderResources, ShaderDefs)]
pub struct VoxelMaterial {
    pub albedo: Color,
}
