#version 460
layout(location = 0) out vec4 f_color;

layout( push_constant ) uniform constants {
    mat2 linear_mat2;
	  vec2 pos;
} pc;

void main() {
    f_color = vec4(1.0, 1.0, 1.0, 1.0);
}