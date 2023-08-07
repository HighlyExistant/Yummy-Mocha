#version 460
layout(location = 0) in vec3 rgb;
layout(location = 0) out vec4 f_color;

layout( push_constant ) uniform constants
{
    mat4 transform;
} pc;

void main() {
    f_color = vec4(vec3(rgb), 1.0);
}