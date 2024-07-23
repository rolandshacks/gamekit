//
// Fragment Shader
//

#version 450

layout (binding = 1) uniform sampler2D iTexture;

layout (location = 0) in vertex_data {
    vec4 position;
    vec4 color;
    vec2 textureCoord;
    flat uint textureMask;
    flat uint flags;
} inputs;

layout (location = 0) out vec4 oColor;

void main() {
    vec2 fragCoord = inputs.position.xy;
    vec4 fragColor = inputs.color;

    fragColor *= texture(iTexture, inputs.textureCoord);

    oColor = fragColor;
}
