use bevy::{
    asset::{Assets, Handle},
    render::{
        pipeline::{
            BlendDescriptor, BlendFactor, BlendOperation, ColorStateDescriptor, ColorWrite,
            CompareFunction, CullMode, DepthStencilStateDescriptor, FrontFace, IndexFormat,
            PipelineDescriptor, RasterizationStateDescriptor, StencilStateFaceDescriptor,
        },
        shader::{Shader, ShaderStage, ShaderStages},
        texture::TextureFormat,
    },
};

pub const PIPELINE_HANDLE: Handle<PipelineDescriptor> = Handle::from_bytes(*b"voxelpipeline000");

pub(crate) fn build_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor {
        index_format: IndexFormat::Uint32,
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil_front: StencilStateFaceDescriptor::IGNORE,
            stencil_back: StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        }),
        color_states: vec![ColorStateDescriptor {
            format: TextureFormat::Bgra8UnormSrgb,
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        ..PipelineDescriptor::new(ShaderStages {
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
}
