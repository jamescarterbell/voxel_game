#version 140

in vec3 position;
in vec2 tex_coord;
in int lighting;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
}
