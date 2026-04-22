#version 450

struct ParticleData {
    vec4 position; // w = scale
    vec4 color;    // w = alpha
};

layout(set = 0, binding = 0) uniform SmokeUBO {
    mat4 view_proj;
    vec4 camera_right;
    vec4 camera_up;
    uint particle_count;
    uint pad1, pad2, pad3;
    ParticleData particles[500];
} ubo;

layout(location = 0) out vec2 fragUV;
layout(location = 1) out vec4 fragColor;

// Procedural Quad Generation (No Vertex Buffer Required!)
const vec2 quad[6] = vec2[](
    vec2(-0.5, -0.5), vec2(0.5, -0.5), vec2(0.5, 0.5),
    vec2(-0.5, -0.5), vec2(0.5, 0.5), vec2(-0.5, 0.5)
);

void main() {
    ParticleData p = ubo.particles[gl_InstanceIndex];
    vec2 v = quad[gl_VertexIndex];

    // AAA Billboarding: Uses camera vectors to force the quad to face the player
    vec3 worldPos = p.position.xyz
        + ubo.camera_right.xyz * v.x * p.position.w
        + ubo.camera_up.xyz * v.y * p.position.w;

    gl_Position = ubo.view_proj * vec4(worldPos, 1.0);
    
    fragUV = v + 0.5; // Map from -0.5..0.5 to 0.0..1.0
    fragColor = p.color;
}