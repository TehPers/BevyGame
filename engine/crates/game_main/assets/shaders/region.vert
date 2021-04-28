#version 450

#define REGION_WIDTH 16
#define REGION_HEIGHT 16
#define REGION_TILES (REGION_WIDTH * REGION_HEIGHT)

layout(location = 0) out vec2 v_Uv;
layout(location = 1) out vec4 v_Color;

struct Rect {
    vec2 begin; // top left
    vec2 end; // bottom right
};
struct TileData {
    vec4 tile_color;
    int atlas_index;
    int padding0; // explicit padding (also implied if not here)
    int padding1; // explicit padding (also implied if not here)
    int padding2; // explicit padding (also implied if not here)
};

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
layout(set = 2, binding = 0) uniform TextureAtlas_size {
    vec2 AtlasSize;
};
layout(set = 2, binding = 1) buffer TextureAtlas_textures {
    Rect[] Textures;
};
layout(std140, set = 3, binding = 0) uniform RegionData_tile_data {
    TileData Tiles[REGION_TILES];
};

void main() {
    uint vertex_index = gl_VertexIndex;

    // 4 vertices per tile, divide by 4 to get tile index
    uint tile_index = vertex_index >> 2;
    if (tile_index >= 256) {
        v_Color = vec4(0.0);
        gl_Position = vec4(10.0);
        return;
    }

    // Tile position can be determined from the index
    vec2 tile_position = vec2(tile_index % 16, tile_index / 16);

    // Offset can be determined from last two bits
    vec2 pos_offset = vec2((vertex_index >> 1) & 1, vertex_index & 1);

    // UV offset can also be determined from last two bits
    vec2 uv_offset = vec2(pos_offset.x, 1 - pos_offset.y);

    TileData tile_data = Tiles[tile_index];
    vec4 tile_color = tile_data.tile_color;
    int atlas_index = tile_data.atlas_index;

    // Check if no tile is being rendered
    if (atlas_index < 0) {
        v_Color = vec4(0.0);
        gl_Position = vec4(10.0);
        return;
    }

    // Get actual UV
    Rect sprite = Textures[atlas_index];
    vec2 sprite_size = sprite.end - sprite.begin;
    
    v_Color = tile_color;
    v_Uv = floor(sprite.begin + sprite_size * uv_offset + vec2(0.01, 0.01)) / AtlasSize;
    gl_Position = ViewProj * Model * vec4(tile_position + pos_offset, 0.0, 1.0);
}
