// src/vulkan_logic/shader_files/postprocess.vert
#version 450

layout(location = 0) out vec2 fragUV;

void main() {
    vec2 uvs[3] = vec2[](
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0)
    );
    
    fragUV = uvs[gl_VertexIndex] * 0.5 + 0.5;
    gl_Position = vec4(uvs[gl_VertexIndex], 0.0, 1.0);
}