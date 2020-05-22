#version 450

in vec2 o_tex_coord;
flat in uint o_tex_index;

uniform sampler2DArray tex;

out vec4 color;

void main() {
    color = texture(tex, vec3(o_tex_coord, float(o_tex_index)));
}
