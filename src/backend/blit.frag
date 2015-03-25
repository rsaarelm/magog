#version 120

uniform sampler2D texture;

varying vec2 v_tex_coord;

void main() {
    vec4 tex_color = texture2D(texture, v_tex_coord);
    tex_color.a = 1.0;
    gl_FragColor = tex_color;
}
