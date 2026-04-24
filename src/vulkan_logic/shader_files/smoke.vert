// src/vulkan_logic/shader_files/smoke.vert
#version 450

layout(location = 0) out vec2 fragUV;
layout(location = 1) out vec4 fragColor;

struct ParticleData {
    vec4 position; 
    vec4 color;    
};

layout(std430, binding = 0) buffer SmokeSSBO {
    mat4 view_proj;
    vec4 camera_right; 
    vec4 camera_up;
    uint particle_count;
    uint _pad[3]; 
    ParticleData particles[];
} smoke;

void main() {
    ParticleData p = smoke.particles[gl_InstanceIndex];
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    fragUV = uv;
    fragColor = p.color;

    vec2 offset = uv * 2.0 - 1.0;
    vec3 worldPos = p.position.xyz 
        + smoke.camera_right.xyz * offset.x * p.position.w 
        + smoke.camera_up.xyz * offset.y * p.position.w;

    gl_Position = smoke.view_proj * vec4(worldPos, 1.0);
}