#version 460
layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 color_i;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding = 0) uniform sampler2D textures[];

layout( push_constant ) uniform constants
{
    mat4 transform;
    mat4 normal;
} pc;

void main() {
    vec4 text = texture(textures[0], uv);
    
    f_color = vec4(color_i, 1.0) + text;
}