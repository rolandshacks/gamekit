//
// Fragment Shader
//

#version 450

layout(std140, binding=0) uniform background_params {
    float offset_x;
    float offset_y;
} params;

layout (location = 0) in vec4 coord;
layout (location = 0) out vec4 col;

void main() {
    //col = vec4(0.42, 0.4, 0.89, 1.0); // C64 light blue

    float y = clamp(0.5 + coord.y / 2.0, 0.0, 1.0);
    float u = clamp(params.offset_y, 0.0, 1.0);

    float r = clamp((1.0 - y) * 0.1 + (1.0 - u) * 0.05, 0.0, 1.0);
    float g = clamp((1.0 - y) * 0.3, 0.0, 1.0);
    float b = clamp(1.0 - y * 0.6, 0.0, 1.0);

    col = vec4(r, g, b, 1.0);
}
