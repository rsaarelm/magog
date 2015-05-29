#version 150 core

uniform sampler2D tex;
in vec2 v_tex_coord;

out vec4 f_color;

void main() {
    vec4 tex_color = texture(tex, v_tex_coord);
    tex_color.a = 1.0;
    f_color = tex_color;
}
