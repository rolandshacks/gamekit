//
// Vertex Shader
//

#version 450

layout(std140, set=0, binding=0) uniform shader_params {
    float window_width;
    float window_height;
    float view_width;
    float view_height;
    float view_x;
    float view_y;
    float view_scaling;
    float time;
    float time_delta;
    int frame;
} params;

layout (location = 0) in vec3 iPosition;
layout (location = 1) in vec4 iColor;
layout (location = 2) in vec2 iTextureCoord;
layout (location = 3) in uint iTextureMask;
layout (location = 4) in uint iFlags;

layout (location = 0) out vertex_data {
    vec4 position;
    vec4 color;
    vec2 textureCoord;
    flat uint textureMask;
    flat uint flags;
} outputs;

layout( push_constant ) uniform constants
{
	float offset_x;
    float offset_y;
} push;

void main() {

    float window_width = params.window_width > 0.0 ? params.window_width : 1.0;
    float window_height = params.window_height > 0.0 ? params.window_height : 1.0;

    float width = params.view_width; if (width <= 0.0) width = 1.0;
    float height = params.view_height; if (height <= 0.0) height = 1.0;

    float x = -1.0 + 2.0 * (push.offset_x + iPosition.x - params.view_x) / width;
    float y = -1.0 + 2.0 * (push.offset_y + iPosition.y - params.view_y) / height;

    vec4 pos = vec4(x, y, iPosition.z, 1.0);

    outputs.position = pos;
    outputs.textureCoord = iTextureCoord;
    outputs.color = iColor;
    outputs.textureMask = iTextureMask;
    outputs.flags = iFlags;

    gl_Position = pos;
}
