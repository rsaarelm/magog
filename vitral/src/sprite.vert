// NB: Must be compiled manually into .spv file using `make shaders`

#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec4 color;
layout(location = 3) in vec4 back_color;
layout(location = 0) out vec4 v_color;
layout(location = 1) out vec4 v_back_color;
layout(location = 2) out vec2 v_tex_coord;

layout(set = 0, binding = 0) uniform Locals {
    mat4 matrix;
};

void main() {
    gl_Position = vec4(pos, 0.0, 1.0) * matrix;
    v_color = color;
    v_back_color = back_color;
    v_tex_coord = tex_coord;
}
