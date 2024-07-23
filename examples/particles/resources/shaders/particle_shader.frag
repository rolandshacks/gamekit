//
// Fragment Shader
//

#version 450

layout (binding = 1) uniform sampler2D iTexture1;

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
    vec2 textureCoord = inputs.textureCoord;
    float intensity = 1.0;
    vec4 col = inputs.color;
    oColor = vec4(col.r*intensity, col.g*intensity, col.b*intensity, col.a) * texture(iTexture1, inputs.textureCoord);
}
