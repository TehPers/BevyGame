#version 450

// Inputs (from vertex buffer)
// layout(location = 0) in uint Vertex_Position_Index;
layout(location = 0) in vec3 Vertex_Expected_Position;
layout(location = 1) in vec2 Vertex_Uv;
layout(location = 2) in vec4 Vertex_Color;

// Outputs (to fragment shader)
layout(location = 0) out vec2 v_Uv;
layout(location = 1) out vec4 t_Color;

// Camera
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

// Transform
layout(set = 1, binding = 0) uniform Transform {
    mat4 WorldTransform;
};

// Tile vertices
/*
struct Vertex {
    vec2 position;
};
layout(set = 3, binding = 0) buffer TileWorldVertexData_positions {
    Vertex[] vertices;
};
*/

void main() {
    v_Uv = Vertex_Uv;
    t_Color = Vertex_Color;
    
    // vec3 vertex_position = vertices[Vertex_Position_Index];
    vec3 vertex_position = Vertex_Expected_Position;
    gl_Position = ViewProj * WorldTransform * vec4(vertex_position, 1.0);
}
