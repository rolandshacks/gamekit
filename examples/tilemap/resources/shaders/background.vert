//
// Vertex Shader
//

#version 450

layout (location = 0) in vec3 pos;
layout (location = 0) out vec4 coord;

void main() {
    coord = vec4(pos, 1.0);
    gl_Position = coord;
}
