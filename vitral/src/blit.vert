// NB: Must be compiled manually into .spv file using `make shaders`

#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) out vec2 v_tex_coord;

layout(set = 0, binding = 0) uniform Locals {
    // Upper left position of the texture canvas in normalized device
    // coordinates. Componets are expected to be between -1.0 and 0.0.
    vec2 canvas_position;
};

// Geometry for triangle strip rectangle.

const vec2 pos[4] = vec2[4](
    vec2(-1.0, -1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2( 1.0,  1.0)
);

const vec2 tex_coord[4] = vec2[4](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0)
);

void main() {
     gl_Position = vec4(-canvas_position * pos[gl_VertexIndex], 0.0, 1.0);
     v_tex_coord = tex_coord[gl_VertexIndex];
}
