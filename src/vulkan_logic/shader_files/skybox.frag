#version 450

layout(location = 0) in vec3 fragRayDir;
layout(location = 0) out vec4 outColor;

struct SkyboxDisc {
    vec4 direction; 
    vec4 color;     
};

struct SkyboxCrescent {
    vec4 direction;
    vec4 color;
    vec4 cutout_offset;
};

layout(set = 0, binding = 0) uniform SkyboxUBO {
    vec4 zenith_color;
    vec4 horizon_color;
    vec4 ground_color; // w: 1.0 = Terrestrial Ground, 0.0 = 360 Space/Orbit
    uint disc_count;
    uint crescent_count;
    uint pad1;
    uint pad2;
    SkyboxDisc discs[5];
    SkyboxCrescent crescents[5];
} ubo;

void main() {
    vec3 dir = normalize(fragRayDir);

    // 1. Calculate Atmospheric Background Gradient
    float t = clamp(dir.y, 0.0, 1.0);
    vec3 sky_color = mix(ubo.horizon_color.xyz, ubo.zenith_color.xyz, pow(t, 0.5));
    
    // Check if the ray is pointing towards the bottom hemisphere
    if (dir.y < 0.0) {
        if (ubo.ground_color.w > 0.5) {
            // Terrestrial Mode: Transition smoothly into the solid ground color
            sky_color = mix(ubo.ground_color.xyz, ubo.horizon_color.xyz, clamp(dir.y + 0.2, 0.0, 1.0));
        } else {
            // Space/Orbit Mode: Mirror the top hemisphere logic perfectly downwards
            float t_bottom = clamp(-dir.y, 0.0, 1.0);
            sky_color = mix(ubo.horizon_color.xyz, ubo.zenith_color.xyz, pow(t_bottom, 0.5));
        }
    }

    // 2. Render Discs (Suns, Large Planets)
    for (uint i = 0; i < ubo.disc_count; i++) {
        vec3 disc_dir = normalize(ubo.discs[i].direction.xyz);
        float angular_size = ubo.discs[i].direction.w;
        float glow = ubo.discs[i].color.w;
        
        float cos_theta = dot(dir, disc_dir);
        
        if (cos_theta > 1.0 - angular_size) {
            // Core of the sun
            sky_color += ubo.discs[i].color.xyz;
        } else if (cos_theta > 1.0 - angular_size * glow) {
            // Soft glow aura
            float factor = (cos_theta - (1.0 - angular_size * glow)) / (angular_size * glow - angular_size);
            sky_color += ubo.discs[i].color.xyz * factor * 0.5;
        }
    }

    // 3. Render Crescents (Moons)
    for (uint i = 0; i < ubo.crescent_count; i++) {
        vec3 main_dir = normalize(ubo.crescents[i].direction.xyz);
        vec3 cutout_dir = normalize(main_dir + ubo.crescents[i].cutout_offset.xyz);
        
        float cos_main = dot(dir, main_dir);
        float cos_cutout = dot(dir, cutout_dir);
        
        float main_size = ubo.crescents[i].direction.w;
        float cutout_size = ubo.crescents[i].color.w;
        
        if (cos_main > 1.0 - main_size && cos_cutout <= 1.0 - cutout_size) {
            sky_color = mix(sky_color, ubo.crescents[i].color.xyz, 0.9);
        }
    }

    outColor = vec4(sky_color, 1.0);
}