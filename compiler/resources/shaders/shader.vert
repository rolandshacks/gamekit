//
// Vertex Shader
//

#version 450

layout(std140, set=0, binding=0) uniform shader_params {
    float resolution_x;
    float resolution_y;
    float x_min;
    float x_max;
    float y_min;
    float y_max;
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

    float screen_width = params.resolution_x > 0.0 ? params.resolution_x : 1.0;
    float screen_height = params.resolution_y > 0.0 ? params.resolution_y : 1.0;

    float width = params.x_max - params.y_min; if (width == 0.0) width = 1.0;
    float height = params.y_max - params.y_min; if (height == 0.0) height = 1.0;

    float x = -1.0 + 2.0 * (push.offset_x + iPosition.x - params.x_min) / width;
    float y = -1.0 + 2.0 * (push.offset_y + iPosition.y - params.y_min) / height;

    vec4 pos = vec4(x, y, iPosition.z, 1.0);

    outputs.position = pos;
    outputs.textureCoord = iTextureCoord;
    outputs.color = iColor;
    outputs.textureMask = iTextureMask;
    outputs.flags = iFlags;

    gl_Position = pos;
}
