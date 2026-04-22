#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 1) in vec4 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    // Distance from the center of the billboard
    vec2 center = fragUV - 0.5;
    float dist = length(center);
    
    // Create a soft glowing sphere shape
    float shape = smoothstep(0.5, 0.05, dist);
    
    // Make the very center intensely white-hot
    vec3 core_glow = mix(fragColor.rgb, vec3(1.0, 1.0, 1.0), smoothstep(0.25, 0.0, dist));
    
    outColor = vec4(core_glow, fragColor.a * shape);
}