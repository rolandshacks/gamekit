//
// Builtin Tile Vertex Shader
//

#version 450
#extension GL_EXT_scalar_block_layout : require

const int INT32_MAX = 0x7FFFFFFF;
const int TRANSPARENT_TILE_ID = INT32_MAX;

// dynamic shader parameters
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

// dynamic lookup table for animated tiles
layout(std140, set=0, binding=2, scalar) uniform shader_tile_lookup_buffer {
    uint animation_frames[256];
} tile_lookup;

// 16 bit position index (hiword) + 16 bit tileset index (loword)
layout (location = 0) in uint map_index;
layout (location = 1) in int tileset_index;

layout (location = 0) out vertex_data {
    vec2 textureCoord;
    vec4 color;
} outputs;

void main() {

    // get quad element (corner)
    uint map_element = gl_VertexIndex;

    // negative tileset indices are references to the animated tiles lookup table

    int rel_index = -(tileset_index+1);

    uint resolved_tileset_index = (tileset_index >= 0) ? 
        uint(tileset_index) : tile_lookup.animation_frames[rel_index];

    // use grid size as scale factor
    float grid_size_f = float(params.grid_size);

    // calculate vertex coords
    uint map_row = map_index / params.cols;
    uint map_col = map_index % params.cols;
    vec2 map_coords = vec2(
        float(map_col) * grid_size_f,
        float(map_row) * grid_size_f
    );

    vec2 screen_coords = vec2(
        (map_coords.x + params.offset_left + grid_size_f / 2.0) * params.view_scaling + params.view_x,
        (map_coords.y + params.offset_top + grid_size_f / 2.0) * params.view_scaling + params.view_y
    );

    vec2 logical_coords = vec2(
        -1.0 + 2.0 * screen_coords.x / params.window_width,
        -1.0 + 2.0 * screen_coords.y / params.window_height
    );

    vec4 vertex_coords = vec4(logical_coords.x, logical_coords.y, 0.0, 1.0);

    // calculate texture coords
    uint tileset_cols = params.texture_width / params.grid_size;
    uint tileset_row = resolved_tileset_index / tileset_cols;
    uint tileset_col = resolved_tileset_index % tileset_cols;

    vec2 texture_coords = vec2(
        float(tileset_col) * grid_size_f / params.texture_width,
        float(tileset_row) * grid_size_f / params.texture_height
    );

    // set outputs for fragment shader
    //outputs.position = vertex_coords;
    outputs.textureCoord = texture_coords;
    outputs.color = resolved_tileset_index != TRANSPARENT_TILE_ID ? vec4(1.0, 1.0, 1.0, 1.0) : vec4(1.0, 1.0, 1.0, 0.0);

    gl_Position = vertex_coords;
    gl_PointSize = grid_size_f * params.view_scaling;
}
