//
// Builtin Tilemap Fragment Shader
//

#version 450

layout(std140, set=0, binding=0) uniform shader_params {
    float offset_left;
    float offset_top;
    float window_width;
    float window_height;
    float view_width;
    float view_height;
    float view_x;
    float view_y;
    float view_scaling;
    uint texture_width;
    uint texture_height;
    uint grid_size;
    uint rows;
    uint cols;
} params;

layout (binding = 1) uniform sampler2D iTexture;

layout (location = 0) in vertex_data {
    vec2 textureCoord;
    vec4 color;
} inputs;

layout (location = 0) out vec4 oColor;

void main() {

    float grid_size_f = float(params.grid_size);

    vec2 scale = vec2(
        grid_size_f / params.texture_width,
        grid_size_f / params.texture_height
    );

    if (inputs.color.w == 0.0) {
        oColor = vec4(1.0, 0.0, 1.0, 0.0);
    } else {
        oColor = texture(iTexture, inputs.textureCoord + gl_PointCoord * scale) * inputs.color;
    }
}
