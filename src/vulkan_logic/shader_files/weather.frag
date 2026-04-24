// src/vulkan_logic/shader_files/weather.frag
#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 1) in vec4 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    // Circular particle shape
    vec2 centerUV = fragUV * 2.0 - 1.0;
    float dist = dot(centerUV, centerUV);
    if (dist > 1.0) discard;
    
    float alpha = (1.0 - dist) * fragColor.a;
    outColor = vec4(fragColor.rgb, alpha);
}