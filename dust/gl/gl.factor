! Copyright (C) 2011 Risto Saarelma

USING: accessors arrays gpu gpu.state kernel math.vectors opengl
opengl.capabilities opengl.gl sequences.deep
specialized-arrays.instances.alien.c-types.float ;

IN: dust.gl

<PRIVATE

: rect-corners ( loc dim -- p00 p10 p11 p01 )
    dupd [ { 1 0 } v* v+ ] [ v+ ] [ { 0 1 } v* v+ ] 2tri ;

: rect-coord-array ( loc dim -- float-array )
    rect-corners 4array flatten >float-array ;

PRIVATE>

: init-gl ( -- )
    "2.0" { } require-gl-version-or-extensions
    init-gpu
    GL_DEPTH_TEST glEnable
    GL_STENCIL_TEST glEnable
    GL_SCISSOR_TEST glEnable
    GL_TEXTURE_2D glEnable
    GL_VERTEX_ARRAY glEnableClientState ;

: set-alpha-blending-state ( -- )
    f
    eq-add func-source-alpha func-one-minus-source-alpha <blend-mode>
    eq-add func-one func-zero <blend-mode>
    <blend-state> set-gpu-state ;
