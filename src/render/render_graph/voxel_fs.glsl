#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in float v_shade;
layout(location = 2) in vec4 v_color;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform VoxelMaterial_albedo {
    vec4 Albedo;
};

void main() {
    o_Target = vec4(Albedo.rgb * v_color.rgb * v_shade, Albedo.a * v_color.a);
}
