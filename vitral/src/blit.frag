// NB: Must be compiled manually into .spv file using `make shaders`

#version 450

layout(location = 0) in vec2 v_tex_coord;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding = 1) uniform texture2D tex;
layout(set = 0, binding = 2) uniform sampler tex_sampler;

void main() {
    vec4 tex_color = texture(sampler2D(tex, tex_sampler), v_tex_coord);
    tex_color.a = 1.0;
    f_color = tex_color;
}
