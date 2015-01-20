#version 120

attribute vec3 pos;
attribute vec4 color;
attribute vec4 back_color;
attribute vec2 tex_coord;

varying vec2 v_tex_coord;
varying vec4 v_color;
varying vec4 v_back_color;

void main() {
    v_tex_coord = tex_coord;
    v_color = color;
    v_back_color = back_color;
    gl_Position = vec4(pos, 1.0);
}
