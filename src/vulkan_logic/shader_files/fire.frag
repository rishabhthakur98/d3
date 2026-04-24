// src/vulkan_logic/shader_files/fire.frag
#version 450
layout(location = 0) in vec2 fragUV;
layout(location = 1) in vec4 fragColor;
layout(location = 0) out vec4 outColor;

void main() {
    vec2 centerUV = fragUV * 2.0 - 1.0;
    float dist = length(centerUV);
    if (dist > 1.0) discard;
    
    // Core is brighter, edges fade out
    float intensity = pow(1.0 - dist, 1.5);
    outColor = vec4(fragColor.rgb * intensity * 2.0, fragColor.a * intensity);
}