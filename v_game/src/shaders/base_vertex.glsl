#version 450

in vec3 position;
in vec2 tex_coord;
in uint tex_index;
in int lighting;

uniform mat4 mvp;

flat out uint o_tex_index;
out vec2 o_tex_coord;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    o_tex_index = tex_index;
    o_tex_coord = tex_coord;
}
