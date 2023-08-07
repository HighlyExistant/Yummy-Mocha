
#version 460

layout( push_constant ) uniform constants
{
    mat4 transform;
} pc;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 rgb;
layout(location = 0) out vec3 fragColor;
void main() {
    gl_Position = pc.transform * vec4(position, 1.0);
    fragColor = rgb;
}