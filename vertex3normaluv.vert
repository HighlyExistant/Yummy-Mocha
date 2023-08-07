#version 460

layout( push_constant ) uniform constants
{
    mat4 transform;
    mat4 normal;
} pc;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec2 uv_o;
layout(location = 1) out vec3 color_o;
layout(set = 0, binding = 0) uniform sampler2D textures[];

const vec3 DIRECTION = normalize(vec3(1.0, -3.0, -1.0));
const float AMBIENT = 0.02;

void main() {
    gl_Position = pc.transform * vec4(pos, 1.0);

    vec3 normal_ws = normalize(mat3(pc.normal) * normal);

    float intensity = AMBIENT + max(dot(normal_ws, DIRECTION), 0);

    uv_o = uv;
    color_o = intensity * vec3(0.5, 0.0, 0.5);
}