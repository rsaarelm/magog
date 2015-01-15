#version 120

uniform sampler2D texture;

varying vec2 v_tex_coord;
varying vec4 v_color;

void main() {
    vec4 tex_color = texture2D(texture, v_tex_coord);

    // Discard fully transparent pixels to keep them from
    // writing into the depth buffer.
    if (tex_color.a == 0.0) discard;

    gl_FragColor = v_color * tex_color;
}
