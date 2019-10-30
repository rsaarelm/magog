// NB: Must be compiled manually into .spv file using `make shaders`

#version 450

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec4 v_back_color;
layout(location = 2) in vec2 v_tex_coord;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding = 1) uniform texture2D tex;
layout(set = 0, binding = 2) uniform sampler tex_sampler;

void main() {
    vec4 tex_color = texture(sampler2D(tex, tex_sampler), v_tex_coord);

    // Discard fully transparent pixels to keep them from
    // writing into the depth buffer.
    if (tex_color.a == 0.0) discard;

    f_color = v_color * tex_color + v_back_color * (vec4(1, 1, 1, 1) - tex_color);
}
