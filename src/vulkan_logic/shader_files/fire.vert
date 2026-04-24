// src/vulkan_logic/shader_files/fire.vert
#version 450

// Constants pulled in from centralized definitions instead of hardcoded magic numbers
#define MAX_FIRE_PARTICLES 500

struct FireParticleData {
    vec4 position; // w = scale
    vec4 color;    // w = alpha
};

layout(set = 0, binding = 0) uniform FireUBO {
    mat4 view_proj;
    vec4 camera_right;
    vec4 camera_up;
    uint particle_count;
    uint pad1, pad2, pad3;
    FireParticleData particles[MAX_FIRE_PARTICLES];
} ubo;

layout(location = 0) out vec2 fragUV;
layout(location = 1) out vec4 fragColor;

const vec2 quad[6] = vec2[](
    vec2(-0.5, -0.5), vec2(0.5, -0.5), vec2(0.5, 0.5),
    vec2(-0.5, -0.5), vec2(0.5, 0.5), vec2(-0.5, 0.5)
);

void main() {
    FireParticleData p = ubo.particles[gl_InstanceIndex];
    vec2 v = quad[gl_VertexIndex];

    vec3 worldPos = p.position.xyz
        + ubo.camera_right.xyz * v.x * p.position.w
        + ubo.camera_up.xyz * v.y * p.position.w;

    gl_Position = ubo.view_proj * vec4(worldPos, 1.0);
    
    fragUV = v + 0.5;
    fragColor = p.color;
}