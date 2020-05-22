#version 450

in vec3 position;
in vec2 tex_coord;
in int lighting;

uniform mat4 m;
uniform mat4 v;
uniform mat4 p;

flat out int vlighting;

void main() {
    gl_Position = v * p * m * vec4(position, 1.0);
    vlighting = lighting;
}
