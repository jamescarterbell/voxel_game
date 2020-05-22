#version 450

flat in int vlighting;

out vec4 color;

void main() {
    vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
    switch (vlighting){
        case 0:
        col = vec4(1.0, 0.0, 0.0, 1.0);
        break;
        case 1:
        col = vec4(1.0, 1.0, 0.0, 1.0);
        break;
        case 2:
        col = vec4(0.0, 1.0, 0.0, 1.0);
        break;
        case 3:
        col = vec4(0.0, 1.0, 1.0, 1.0);
        break;
        case 4:
        col = vec4(0.0, 0.0, 1.0, 1.0);
        break;
        case 5:
        col = vec4(1.0, 0.0, 1.0, 1.0);
        break;
    }
    color = col;
}
