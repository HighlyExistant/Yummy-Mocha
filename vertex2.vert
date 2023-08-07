#version 460

layout( push_constant ) uniform constants
{
    mat2 linear_mat2;
	vec2 pos;
} pc;

layout(location = 0) in vec2 position;
void main() {
    gl_Position = vec4(pc.linear_mat2 * position + pc.pos , 0.0, 1.0);
}