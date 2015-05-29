#version 150 core

in vec3 pos;
in vec4 color;
in vec4 back_color;
in vec2 tex_coord;

out vec2 v_tex_coord;
out vec4 v_color;
out vec4 v_back_color;

void main() {
    v_tex_coord = tex_coord;
    v_color = color;
    v_back_color = back_color;
    gl_Position = vec4(pos, 1.0);
}
