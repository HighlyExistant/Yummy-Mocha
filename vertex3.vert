
#version 460

layout( push_constant ) uniform constants
{
    mat4 transform;
} pc;

layout(location = 0) in vec3 position;
void main() {
    gl_Position = pc.transform * vec4(position, 1.0);
}