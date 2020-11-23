#version 450

layout(location = 0) in vec3 Voxel_Position;
layout(location = 1) in float Voxel_Shade;
layout(location = 2) in vec4 Voxel_Color;

layout(location = 0) out flat vec3 v_position;
layout(location = 1) out flat float v_shade;
layout(location = 2) out flat vec4 v_color;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_position = (Model * vec4(Voxel_Position, 1.0)).xyz;
    v_shade = Voxel_Shade;
    v_color = Voxel_Color;
    gl_Position = ViewProj * vec4(v_position, 1.0);
}
