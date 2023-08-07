#version 460

layout( push_constant ) uniform constants
{
    mat4 transform;
    uint index;
} pc;

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 text_coords;

layout(location = 0) out vec2 text_coord_out;
layout(binding = 0) uniform sampler2D texSampler[1];
void main() {
    gl_Position = pc.transform * vec4(position, 1.0);
    text_coord_out = text_coords;
}