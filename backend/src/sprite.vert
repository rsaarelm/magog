#version 120

attribute vec3 pos;
attribute vec4 color;
attribute vec2 tex_coord;

varying vec2 v_tex_coord;
varying vec4 v_color;

void main() {
    v_tex_coord = tex_coord;
    v_color = color;
    gl_Position = vec4(pos, 1.0);
}
