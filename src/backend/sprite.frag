#version 150 core

uniform sampler2D tex;
in vec2 v_tex_coord;
in vec4 v_color;
in vec4 v_back_color;

out vec4 f_color;

void main() {
    vec4 tex_color = texture(tex, v_tex_coord);

    // Discard fully transparent pixels to keep them from
    // writing into the depth buffer.
    if (tex_color.a == 0.0) discard;

    f_color = v_color * tex_color + v_back_color * (vec4(1, 1, 1, 1) - tex_color);
}
