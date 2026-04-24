// src/vulkan_logic/shader_files/skybox.frag
#version 450

layout(location = 0) in vec3 fragRayDir;
layout(location = 1) out vec4 outColor;

struct SkyboxDiscData {
    vec4 direction; // xyz: Direction, w: Angular Size
    vec4 color;     // xyz: Color, w: Glow Intensity
};

struct SkyboxCrescentData {
    vec4 direction;     // xyz: Direction, w: Angular Size
    vec4 color;         // xyz: Color, w: Cutout Size
    vec4 cutout_offset; // xyz: Cutout Directional Offset, w: Padding
};

// PHASE 3 FIX: SSBO mapping
layout(std430, binding = 0) buffer SkyboxSSBO {
    vec4 zenith_color;
    vec4 horizon_color;
    vec4 ground_color;
    
    uint disc_count;
    uint crescent_count;
    uint _pad[2];
    
    SkyboxDiscData discs[5];        
    SkyboxCrescentData crescents[]; 
} sky;

void main() {
    vec3 dir = normalize(fragRayDir);
    
    // Gradient logic
    float y = dir.y;
    vec3 color;
    if (y >= 0.0) {
        // Sky
        float blend = pow(y, 0.5); // Smooth gradient
        color = mix(sky.horizon_color.rgb, sky.zenith_color.rgb, blend);
    } else {
        // Ground
        if (sky.ground_color.a > 0.5) {
            color = sky.ground_color.rgb;
        } else {
            color = sky.horizon_color.rgb; // Extend horizon down if ground is disabled
        }
    }
    
    // Draw Discs (e.g. Suns)
    for (uint i = 0; i < sky.disc_count; i++) {
        vec3 discDir = normalize(sky.discs[i].direction.xyz);
        float size = sky.discs[i].direction.w;
        float dist = distance(dir, discDir);
        
        if (dist < size) {
            color = sky.discs[i].color.rgb;
        } else {
            // Glow
            float glowDist = dist - size;
            float glowFactor = exp(-glowDist * sky.discs[i].color.w);
            color += sky.discs[i].color.rgb * glowFactor * 0.5;
        }
    }

    outColor = vec4(color, 1.0);
}