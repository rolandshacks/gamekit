//
// Vertex Shader
//

#version 450

layout(std140, set=0, binding=0) uniform shader_params {
    float time;
    float time_delta;
    int frame;
    float width;
    float height;
    float offset_left;
    float offset_top;
    float window_width;
    float window_height;
    float view_left;
    float view_top;
    float view_width;
    float view_height;
} params;

layout (location = 0) in vec3 vertex_pos;
layout (location = 1) in vec4 vertex_color;
layout (location = 2) in vec2 vertex_texcoord;
layout (location = 3) in uint vertex_texmask;
layout (location = 4) in uint vertex_flags;

layout (location = 0) out vertex_data {
    vec4 position;
    vec4 color;
    vec2 textureCoord;
    flat uint textureMask;
    flat uint flags;
} outputs;

void main() {

    float scale_x = params.view_width / params.width;
    float scale_y = params.view_height / params.height;

    float xpos = (vertex_pos.x + params.offset_left) * scale_x + params.view_left;
    float ypos = (vertex_pos.y + params.offset_top) * scale_y + params.view_top;

    float log_x = -1.0 + 2.0 * xpos / params.window_width;
    float log_y = -1.0 + 2.0 * ypos / params.window_height;

    vec4 pos = vec4(log_x, log_y, vertex_pos.z, 1.0);

    outputs.position = pos;
    outputs.color = vertex_color;
    outputs.textureCoord = vertex_texcoord;
    outputs.textureMask = vertex_texmask;
    outputs.flags = vertex_flags;

    gl_Position = pos;
}