#version 450

// Inputs (from vertex buffer)
layout(location = 0) in vec3 Vertex_Position;
// layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;
layout(location = 3) in vec4 Tile_Color;

// Outputs (to fragment shader)
layout(location = 0) out vec2 v_Uv;
layout(location = 1) out vec4 t_Color;

// Camera
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

// Transform
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_Uv = Vertex_Uv;
    t_Color = Tile_Color;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
