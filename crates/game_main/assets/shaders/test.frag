#version 450

// Inputs (from vertex shader)
layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 t_Color;

// Outputs (to main pass)
layout(location = 0) out vec4 o_Target;

// Tile world material
layout(set = 2, binding = 0) uniform TileWorldMaterial_color {
    vec4 Color;
};
layout(set = 2, binding = 1) uniform texture2D TileWorldMaterial_tile_sheet;
layout(set = 2, binding = 2) uniform sampler TileWorldMaterial_tile_sheet_sampler;

void main() {
    o_Target = t_Color * texture(sampler2D(TileWorldMaterial_tile_sheet, TileWorldMaterial_tile_sheet_sampler), v_Uv);
}
