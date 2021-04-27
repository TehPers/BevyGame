#version 450

#define REGION_WIDTH 16
#define REGION_HEIGHT 16
#define REGION_TILES (REGION_WIDTH * REGION_HEIGHT)

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_Uv;
layout(location = 2) in int Vertex_Tile_Index;
// layout(location = 3) in vec4 Vertex_Color;

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
    if (Vertex_Tile_Index >= 256) {
        v_Color = vec4(0.0);
        gl_Position = vec4(10.0);
        return;
    }

    TileData tile_data = Tiles[Vertex_Tile_Index];
    vec4 tile_color = tile_data.tile_color;
    int atlas_index = tile_data.atlas_index;

    // Check if no tile is being rendered
    if (atlas_index < 0) {
        v_Color = vec4(0.0);
        gl_Position = vec4(10.0);
        return;
    }

    // Get UV
    Rect sprite = Textures[atlas_index];
    vec2 sprite_size = sprite.end - sprite.begin;
    
    v_Color = tile_color;
    v_Uv = floor(sprite.begin + sprite_size * Vertex_Uv + vec2(0.01, 0.01)) / AtlasSize;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
