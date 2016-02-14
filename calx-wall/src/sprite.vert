#version 150 core

uniform vec2 canvas_size;
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

    // Translate to canvas pixel size.
    vec3 my_pos = pos;
    my_pos.x = -1.0 + (2.0 * my_pos.x) / canvas_size.x;
    my_pos.y = 1.0 - (2.0 * my_pos.y) / canvas_size.y;

    gl_Position = vec4(my_pos, 1.0);
}
