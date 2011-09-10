! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays colors colors.constants dust.img gpu.render
gpu.shaders gpu.state gpu.textures gpu.util images kernel locals math.matrices
math.vectors opengl.gl sequences ;

IN: dust.gl.texture-rect

! Drawing texture rects with GPU shaders

TUPLE: texture-rect
    texture
    tex-subrect
    dim ;

<PRIVATE

GLSL-SHADER: rect-vertex-shader vertex-shader
#version 110

uniform mat4 p_matrix;
uniform mat4 mv_matrix;

uniform vec2 rect_dim;
uniform vec2 tex_loc;
uniform vec2 tex_dim;

attribute vec2 vertex;
varying vec2 texcoord;

void main() {
    texcoord = (vertex * vec2(1.0, -1.0)) * vec2(0.5) + vec2(0.5);
    texcoord = texcoord * tex_dim + tex_loc;
    gl_Position = p_matrix * (mv_matrix * vec4(vertex * rect_dim * 0.5, 0.0, 1.0));
}
;

GLSL-SHADER: rect-fragment-shader fragment-shader
#version 110

uniform sampler2D loading_texture;
uniform vec4 color;
varying vec2 texcoord;

void main() {
    gl_FragColor = texture2D(loading_texture, texcoord) * color;
}
;

GLSL-PROGRAM: rect-program rect-vertex-shader rect-fragment-shader window-vertex-format ;

UNIFORM-TUPLE: rect-uniforms
    { "p_matrix"        mat4-uniform    f }
    { "mv_matrix"       mat4-uniform    f }
    { "rect_dim"        vec2-uniform    f }
    { "tex_loc"         vec2-uniform    f }
    { "tex_dim"         vec2-uniform    f }
    { "color"           vec4-uniform    f }
    { "loading-texture" texture-uniform f } ;

: rect-vertex-array ( -- vertex-array )
    <window-vertex-buffer> rect-program <program-instance> <vertex-array> ;

: rect-mv-matrix ( loc dim -- mv-matrix )
    [ 0 suffix translation-matrix4 ] [ 1 suffix scale-matrix4 ]
    bi* m. ;

:: <rect-uniforms> ( texture-rect color -- rect-uniforms )
    GL_PROJECTION_MATRIX 16 get-gl-floats
    GL_MODELVIEW_MATRIX 16 get-gl-floats
    texture-rect dim>>
    texture-rect tex-subrect>> first2
    color >rgba-components 4array
    texture-rect texture>> rect-uniforms boa ;

PRIVATE>

: make-gpu-texture ( image -- tdt )
    [ component-order>> ubyte-components
      T{ texture-parameters
         { wrap clamp-texcoord-to-edge }
         { min-filter filter-linear }
         { mag-filter filter-nearest }
         { min-mipmap-filter f }
      } <texture-2d>
      dup 0 ] [ ] bi allocate-texture-image ;

: <texture-rect> ( texture tex-subrect size-dim -- texture-rect )
    texture-rect boa ;

:: image>texture-rect ( image -- texture-rect )
    image clone-pow2-dim-image :> tex-source
    tex-source make-gpu-texture { 0 0 }
    image dim>> tex-source dim>> v/ 2array
    image dim>> <texture-rect> ;

: render-colored-rect ( texture-rect color -- )
    <rect-uniforms> {
        { "primitive-mode" [ drop triangle-strip-mode ] }
        { "indexes" [ drop T{ index-range f 0 4 } ] }
        { "uniforms" [ ] }
        { "vertex-array" [ drop rect-vertex-array ] }
    } <render-set> render ;

: render-rect ( texture-rect -- ) COLOR: white render-colored-rect ;

:: render-with-framebuffer ( framebuffer texture-rect -- )
    texture-rect COLOR: white <rect-uniforms> framebuffer {
        { "primitive-mode" [ 2drop triangle-strip-mode ] }
        { "indexes" [ 2drop T{ index-range f 0 4 } ] }
        { "uniforms" [ drop ] }
        { "vertex-array" [ 2drop rect-vertex-array ] }
        { "framebuffer" [ nip ] }
    } 2<render-set> render ;
