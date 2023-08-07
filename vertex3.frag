#version 460
layout(location = 0) out vec4 f_color;

layout( push_constant ) uniform constants
{
    mat4 transform;
} pc;

void main() {
    f_color = vec4(1.0, 1.0, 1.0, 1.0);
}