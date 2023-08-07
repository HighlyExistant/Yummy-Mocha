#version 460
layout(location = 0) in vec2 texture_coords;
layout(location = 0) out vec4 f_color;
layout(binding = 0) uniform sampler2D texSampler[1];

layout( push_constant ) uniform constants
{
    mat4 transform;
    uint index;
} pc;

void main() {
    vec4 text = texture(texSampler[pc.index], texture_coords);
    
    f_color = text;
}