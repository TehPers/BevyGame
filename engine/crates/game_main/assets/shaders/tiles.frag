#version 450

// Inputs (from vertex shader)
layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 t_Color;

// Outputs (to main pass)
layout(location = 0) out vec4 o_Target;

// Tile world material
layout(set = 2, binding = 1) uniform texture2D TileWorldMaterial_texture;
layout(set = 2, binding = 2) uniform sampler TileWorldMaterial_texture_sampler;

void main() {
    o_Target = t_Color * texture(sampler2D(TileWorldMaterial_texture, TileWorldMaterial_texture_sampler), v_Uv);
}
